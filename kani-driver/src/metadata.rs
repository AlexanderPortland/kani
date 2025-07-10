// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::{Result, bail};
use std::path::Path;
// use tracing::{debug, trace};

use kani_metadata::{HarnessMetadata, InternedString, TraitDefinedMethod, VtableCtxResults};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::session::KaniSession;
use serde::Deserialize;

/// From either a file or a path with multiple files, output the CBMC restrictions file we should use.
pub fn collect_and_link_function_pointer_restrictions(
    path: &Path,
    output_filename: &Path,
) -> Result<()> {
    let md = std::fs::metadata(path)?;

    // Fill with data from all files in that path with the expected suffix
    let mut per_crate_restrictions = Vec::new();

    if md.is_dir() {
        for element in path.read_dir()? {
            let path = element?.path();
            if path.as_os_str().to_str().unwrap().ends_with(".restrictions.json") {
                let restrictions = from_json(&path)?;
                per_crate_restrictions.push(restrictions);
            }
        }
    } else if md.is_file() {
        assert!(path.as_os_str().to_str().unwrap().ends_with(".restrictions.json"));
        let restrictions = from_json(path)?;
        per_crate_restrictions.push(restrictions);
    } else {
        unreachable!("Path must be restrcitions file or directory containing restrictions files")
    }

    link_function_pointer_restrictions(per_crate_restrictions, output_filename)
}

/// Collect all vtable restriction metadata together, and write one combined output in CBMC's format
fn link_function_pointer_restrictions(
    data_per_crate: Vec<VtableCtxResults>,
    output_filename: &Path,
) -> Result<()> {
    // Combine all method possibilities into one global mapping
    let mut combined_possible_methods: HashMap<TraitDefinedMethod, Vec<InternedString>> =
        HashMap::new();
    for crate_data in &data_per_crate {
        for entry in &crate_data.possible_methods {
            combined_possible_methods
                .insert(entry.trait_method.clone(), entry.possibilities.clone());
        }
    }

    // Emit a restriction for every call site
    let mut output = HashMap::new();
    for crate_data in data_per_crate {
        for call_site in crate_data.call_sites {
            // CBMC Now supports referencing callsites by label:
            // https://github.com/diffblue/cbmc/pull/6508
            let cbmc_call_site_name = format!("{}.{}", call_site.function_name, call_site.label);
            let trait_def = call_site.trait_method;

            // Look up all possibilities, defaulting to the empty set
            let possibilities =
                combined_possible_methods.get(&trait_def).unwrap_or(&vec![]).clone();
            output.insert(cbmc_call_site_name, possibilities);
        }
    }

    let f = File::create(output_filename)?;
    let f = BufWriter::new(f);
    serde_json::to_writer(f, &output)?;
    Ok(())
}

/// Deserialize a json file into a given structure
pub fn from_json<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let obj = serde_json::from_reader(reader)?;
    Ok(obj)
}

impl KaniSession {
    /// Determine which function to use as entry point, based on command-line arguments and kani-metadata.
    pub fn determine_targets<'a>(
        &self,
        all_harnesses: Vec<&'a HarnessMetadata>,
    ) -> Result<Vec<&'a HarnessMetadata>> {
        let harnesses = BTreeSet::from_iter(self.args.harnesses.iter());
        let all_targets = &harnesses;

        // For dev builds, re-filter the harnesses to ensure we're doing the minimal harness codegen possible.
        if cfg!(debug_assertions) {
            let filtered_harnesses = find_proof_harnesses();
        }

        // If any of the `--harness` filters failed to find a harness, report that to the user.
        if self.args.exact && (all_harnesses.len() < self.args.harnesses.len()) {
            let harness_found_names: BTreeSet<&String> =
                all_harnesses.iter().map(|&h| &h.pretty_name).collect();

            // Check which harnesses are missing from the difference of targets and harnesses_found
            let harnesses_missing: Vec<&String> =
                all_targets.difference(&harness_found_names).cloned().collect();
            let joined_string = harnesses_missing
                .iter()
                .map(|&s| (*s).clone())
                .collect::<Vec<String>>()
                .join("`, `");

            bail!(
                "Failed to match the following harness(es):\n`{joined_string}`\nPlease specify the fully-qualified name of a harness.",
            );
        }

        Ok(all_harnesses)
    }
}

/// Sort harnesses such that for two harnesses in the same file, it is guaranteed that later
/// appearing harnesses get processed earlier.
/// This is necessary for the concrete playback feature (with in-place unit test modification)
/// because it guarantees that injected unit tests will not change the location of to-be-processed harnesses.
pub fn sort_harnesses_by_loc<'a>(harnesses: &[&'a HarnessMetadata]) -> Vec<&'a HarnessMetadata> {
    let mut harnesses_clone = harnesses.to_vec();
    harnesses_clone.sort_unstable_by(|harness1, harness2| {
        harness1
            .original_file
            .cmp(&harness2.original_file)
            .then(harness1.original_start_line.cmp(&harness2.original_start_line).reverse())
    });
    harnesses_clone
}
