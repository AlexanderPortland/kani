// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
//! This module contains a MIR pass that replaces some intrinsics by rust intrinsics models as
//! well as validation logic that can only be added during monomorphization.
//!
//! TODO: Move this code to `[crate::kani_middle::transform::RustcIntrinsicsPass]` since we can do
//! proper error handling after monomorphization.
use rustc_index::IndexVec;
use rustc_middle::mir::{Body, Const as mirConst, ConstValue, Operand, TerminatorKind};
use rustc_middle::mir::{Local, LocalDecl};
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_middle::ty::{Const, GenericArgsRef, IntrinsicDef};
use rustc_span::source_map::Spanned;
use rustc_span::symbol::{Symbol, sym};
use tracing::{debug, trace};

pub struct ModelIntrinsics<'tcx> {
    tcx: TyCtxt<'tcx>,
    /// Local declarations of the function being transformed.
    local_decls: IndexVec<Local, LocalDecl<'tcx>>,
}

impl<'tcx> ModelIntrinsics<'tcx> {
    /// Function that replace calls to some intrinsics that have a high level model in our library.
    ///
    /// For now, we only look at intrinsic calls, which are modelled by a terminator.
    ///
    /// However, this pass runs after lowering intrinsics, which may replace the terminator by
    /// an intrinsic statement (non-diverging intrinsic).
    pub fn run_pass(tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        ModelIntrinsics { tcx, local_decls: body.local_decls.clone() }.transform(body)
    }

    pub fn transform(&self, body: &mut Body<'tcx>) {
        for block in body.basic_blocks.as_mut() {
            let terminator = block.terminator_mut();
            if let TerminatorKind::Call { func, args, .. } = &mut terminator.kind {
                let func_ty = func.ty(&self.local_decls, self.tcx);
                if let Some((intrinsic, generics)) = resolve_rust_intrinsic(self.tcx, func_ty) {
                    let intrinsic_name = intrinsic.name;
                    trace!(?func, ?intrinsic_name, "run_pass");
                    if intrinsic_name == sym::simd_bitmask {
                        self.replace_simd_bitmask(func, args, generics)
                    }
                }
            }
        }
    }

    /// Change the function call to use the stubbed version.
    /// We only replace calls if we can ensure the input has simd representation.
    fn replace_simd_bitmask(
        &self,
        func: &mut Operand<'tcx>,
        args: &[Spanned<Operand<'tcx>>],
        gen_args: GenericArgsRef<'tcx>,
    ) {
        assert_eq!(args.len(), 1);
        let tcx = self.tcx;
        let arg_ty = args[0].node.ty(&self.local_decls, tcx);
        if arg_ty.is_simd() {
            // Get the stub definition.
            let Some(stub_id) = tcx.get_diagnostic_item(Symbol::intern("KaniModelSimdBitmask"))
            else {
                // This should only happen when verifying the standard library.
                // We don't need to warn here, since the backend will print unsupported constructs.
                return;
            };
            debug!(?func, ?stub_id, "replace_simd_bitmask");

            // Get SIMD information from the type.
            let (len, elem_ty) = simd_len_and_type(tcx, arg_ty);
            debug!(?len, ?elem_ty, "replace_simd_bitmask Ok");

            // Increment the list of generic arguments since our stub also takes element type and len.
            let mut new_gen_args = Vec::from_iter(gen_args.iter());
            new_gen_args.push(elem_ty.into());
            new_gen_args.push(len.into());

            let Operand::Constant(fn_def) = func else { unreachable!() };
            fn_def.const_ = mirConst::from_value(
                ConstValue::ZeroSized,
                tcx.type_of(stub_id).instantiate(tcx, &*new_gen_args),
            );
        } else {
            debug!(?arg_ty, "replace_simd_bitmask failed");
        }
    }
}

fn simd_len_and_type<'tcx>(tcx: TyCtxt<'tcx>, simd_ty: Ty<'tcx>) -> (Const<'tcx>, Ty<'tcx>) {
    match simd_ty.kind() {
        ty::Adt(def, args) => {
            assert!(def.repr().simd(), "`simd_size_and_type` called on non-SIMD type");
            let variant = def.non_enum_variant();
            let f0_ty = variant.fields[rustc_abi::FieldIdx::from_usize(0)].ty(tcx, args);

            match f0_ty.kind() {
                ty::Array(elem_ty, len) => (*len, *elem_ty),
                _ => (Const::from_target_usize(tcx, variant.fields.len() as u64), f0_ty),
            }
        }
        _ => unreachable!("unexpected layout for simd type {simd_ty}"),
    }
}

fn resolve_rust_intrinsic<'tcx>(
    tcx: TyCtxt<'tcx>,
    func_ty: Ty<'tcx>,
) -> Option<(IntrinsicDef, GenericArgsRef<'tcx>)> {
    if let ty::FnDef(def_id, args) = *func_ty.kind()
        && let Some(symbol) = tcx.intrinsic(def_id)
    {
        return Some((symbol, args));
    }
    None
}
