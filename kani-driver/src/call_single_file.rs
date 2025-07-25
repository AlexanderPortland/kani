// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::Result;
use kani_metadata::UnstableFeature;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::session::{KaniSession, lib_folder};
use crate::util::args::{CommandWrapper, KaniArg, PassTo, RustcArg, encode_as_rustc_arg};

pub struct LibConfig {
    args: Vec<RustcArg>,
}

impl LibConfig {
    pub fn new(path: PathBuf) -> LibConfig {
        let sysroot = &path.parent().unwrap();
        let kani_std_rlib = path.join("libstd.rlib");
        let kani_std_wrapper = format!("noprelude:std={}", kani_std_rlib.to_str().unwrap());
        let args = [
            "--sysroot",
            sysroot.to_str().unwrap(),
            "-L",
            path.to_str().unwrap(),
            "--extern",
            "kani",
            "--extern",
            kani_std_wrapper.as_str(),
        ]
        .map(RustcArg::from)
        .to_vec();
        LibConfig { args }
    }

    pub fn new_no_core(path: PathBuf) -> LibConfig {
        LibConfig {
            args: ["-L", path.to_str().unwrap(), "--extern", "kani_core"]
                .map(RustcArg::from)
                .to_vec(),
        }
    }
}

impl KaniSession {
    /// Used by `kani` and not `cargo-kani` to process a single Rust file into a `.symtab.json`
    // TODO: Move these functions to be part of the builder.
    pub fn compile_single_rust_file(
        &self,
        file: &Path,
        crate_name: &String,
        outdir: &Path,
    ) -> Result<()> {
        let mut kani_args = self.kani_compiler_local_flags();
        kani_args.push(format!("--reachability={}", self.reachability_mode()).into());

        let lib_path = lib_folder().unwrap();
        let mut rustc_args = self.kani_rustc_flags(LibConfig::new(lib_path));
        rustc_args.push(file.into());
        rustc_args.push("--out-dir".into());
        rustc_args.push(RustcArg::from(outdir.as_os_str()));
        rustc_args.push("--crate-name".into());
        rustc_args.push(crate_name.into());

        if self.args.tests {
            // e.g. `tests/kani/Options/check_tests.rs` will fail because it already has it
            // so this is a hacky workaround
            let t = "--test".into();
            if !rustc_args.contains(&t) {
                rustc_args.push(t);
            }
        } else {
            // We only run against proof harnesses, so always compile as a library.
            // This ensures compilation passes if the crate does not have a `main` function.
            rustc_args.push("--crate-type".into());
            rustc_args.push("lib".into());
        }

        // Note that the order of arguments is important. Kani specific flags should precede
        // rustc ones.
        let mut cmd = Command::new(&self.kani_compiler);

        cmd.pass_rustc_arg(encode_as_rustc_arg(&kani_args), PassTo::OnlyLocalCrate)
            .pass_rustc_args(&rustc_args, PassTo::OnlyLocalCrate);

        // This is only required for stable but is a no-op for nightly channels
        cmd.env("RUSTC_BOOTSTRAP", "1");

        if self.args.common_args.quiet {
            self.run_suppress(cmd)?;
        } else {
            self.run_terminal(cmd)?;
        }
        Ok(())
    }

    /// Create a compiler option that represents the reachability mode.
    pub fn reachability_arg(&self) -> KaniArg {
        format!("--reachability={}", self.reachability_mode()).into()
    }

    /// The `kani-compiler`-specific arguments that should be passed when building all crates,
    /// including dependencies.
    pub fn kani_compiler_dependency_flags(&self) -> Vec<KaniArg> {
        let mut flags = vec![check_version()];

        if self.args.ignore_global_asm {
            flags.push("--ignore-global-asm".into());
        }

        flags
    }

    /// The `kani-compiler`-specific arguments that should be passed only to the local crate
    /// being compiled.
    pub fn kani_compiler_local_flags(&self) -> Vec<KaniArg> {
        let mut flags: Vec<KaniArg> = vec![];

        if self.args.common_args.debug {
            flags.push("--log-level=debug".into());
        } else if self.args.common_args.verbose {
            // Print the symtab command being invoked.
            flags.push("--log-level=info".into());
        } else {
            flags.push("--log-level=warn".into());
        }

        if self.args.restrict_vtable() {
            flags.push("--restrict-vtable-fn-ptrs".into());
        }
        if self.args.assertion_reach_checks() {
            flags.push("--assertion-reach-checks".into());
        }

        if self.args.is_stubbing_enabled() {
            flags.push("--enable-stubbing".into());
        }

        if self.args.coverage {
            flags.push("--coverage-checks".into());
        }

        if self.args.common_args.unstable_features.contains(UnstableFeature::ValidValueChecks) {
            flags.push("--ub-check=validity".into())
        }

        if self.args.common_args.unstable_features.contains(UnstableFeature::UninitChecks) {
            // Automatically enable shadow memory, since the version of uninitialized memory checks
            // without non-determinism depends on it.
            flags.push("-Z ghost-state".into());
            flags.push("--ub-check=uninit".into());
        }

        if self.args.common_args.unstable_features.contains(UnstableFeature::Lean) {
            flags.push("--backend=llbc".into());
        }

        if self.args.print_llbc {
            flags.push("--print-llbc".into());
        }

        if self.args.no_assert_contracts {
            flags.push("--no-assert-contracts".into());
        }

        for harness in &self.args.harnesses {
            flags.push(format!("--harness {harness}").into());
        }

        if self.args.exact {
            flags.push("--exact".into());
        }

        if let Some(args) = self.autoharness_compiler_flags.clone() {
            flags.extend(args.into_iter().map(KaniArg::from));
        }

        flags.extend(self.args.common_args.unstable_features.as_arguments().map(KaniArg::from));

        flags
    }

    /// This function generates all rustc configurations required by our goto-c codegen.
    pub fn kani_rustc_flags(&self, lib_config: LibConfig) -> Vec<RustcArg> {
        let mut flags: Vec<_> = base_rustc_flags(lib_config);
        // We only use panic abort strategy for verification since we cannot handle unwind logic.
        if self.args.coverage {
            flags.extend_from_slice(
                &["-C", "instrument-coverage", "-Z", "no-profiler-runtime"].map(RustcArg::from),
            );
        }
        flags.extend_from_slice(
            &[
                "-C",
                "panic=abort",
                "-C",
                "symbol-mangling-version=v0",
                "-Z",
                "panic_abort_tests=yes",
                "-Z",
                "mir-enable-passes=-RemoveStorageMarkers",
                "--check-cfg=cfg(kani)",
                // Do not invoke the linker since the compiler will not generate real object files
                "-Clinker=echo",
            ]
            .map(RustcArg::from),
        );

        if self.args.no_codegen {
            flags.push("-Z".into());
            flags.push("no-codegen".into());
        }

        if let Some(seed_opt) = self.args.randomize_layout {
            flags.push("-Z".into());
            flags.push("randomize-layout".into());
            if let Some(seed) = seed_opt {
                flags.push("-Z".into());
                flags.push(format!("layout-seed={seed}").into());
            }
        }

        if self.args.coverage {
            flags.push("-Zmir-enable-passes=-SingleUseConsts".into());
        }

        // This argument will select the Kani flavour of the compiler. It will be removed before
        // rustc driver is invoked.
        flags.push("--kani-compiler".into());

        flags
    }
}

/// Common flags used for compiling user code for verification and playback flow.
pub fn base_rustc_flags(lib_config: LibConfig) -> Vec<RustcArg> {
    let mut flags = [
        "-C",
        "overflow-checks=on",
        "-Z",
        "unstable-options",
        "-Z",
        "trim-diagnostic-paths=no",
        "-Z",
        "human_readable_cgu_names",
        "-Z",
        "always-encode-mir",
        "--cfg=kani",
        "-Z",
        "crate-attr=feature(register_tool)",
        "-Z",
        "crate-attr=register_tool(kanitool)",
    ]
    .map(RustcArg::from)
    .to_vec();

    flags.extend(lib_config.args);

    // e.g. compiletest will set 'compile-flags' here and we should pass those down to rustc
    // and we fail in `tests/kani/Match/match_bool.rs`
    if let Ok(str) = std::env::var("RUSTFLAGS") {
        flags.extend(str.split(' ').map(RustcArg::from));
    }

    flags
}

/// Function that returns a `--check-version` argument to be added to the compiler flags.
/// This is really just used to force the compiler to recompile everything from scratch when a user
/// upgrades Kani. Cargo currently ignores the codegen backend version.
/// See <https://github.com/model-checking/kani/issues/2140> for more context.
fn check_version() -> KaniArg {
    format!("--check-version={}", env!("CARGO_PKG_VERSION")).into()
}
