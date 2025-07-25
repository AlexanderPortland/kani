// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests that use unsafe casts to write to mutable variables through immutable references.
//!
//! Irregardless of the type or how long the chain of references is, writing through the immutable
//! reference constitutes mutating immutable bytes.

use std::mem::transmute;

fn ref_to_mut_ref<T>(r: &T) -> &mut T {
    unsafe { transmute::<usize, &mut T>(transmute::<&T, usize>(r)) }
}

fn simple<T>(old: T, new: T) {
    let mut local: T = old;

    let ref1: &mut T = &mut local;

    let immut_ref: &T = &*ref1; // reborrow as immutable
    *ref_to_mut_ref(immut_ref) = new;
}

#[kani::proof]
pub fn test_simple() {
    simple::<i32>(42, 10);
    simple::<Box<bool>>(Box::new(true), Box::new(true));
    simple::<Vec<usize>>(vec![1, 2], vec![1, 2, 3]);
}

fn nested<T>(old: T, new: T) {
    let mut local = old;

    let ref1 = &mut local;
    let ref2 = &ref1;

    let nested_ref: Box<&&mut T> = Box::new(ref2);
    **ref_to_mut_ref(*nested_ref) = new;
}

#[kani::proof]
pub fn test_nested() {
    nested::<i32>(42, 10);
    nested::<Box<bool>>(Box::new(true), Box::new(true));
    nested::<Vec<usize>>(vec![1, 2], vec![1, 2, 3]);
}

fn very_nested<T>(old: T, new: T) {
    let mut local = old;

    let ref1 = &mut local;
    let mut ref2 = ContainsImmutRef(&ref1);
    let ref3 = &mut ref2;
    let ref4 = Box::new(ref3);

    let crazy_nested_ref: &Box<&mut ContainsImmutRef<'_, &mut T>> = &ref4;

    **ref_to_mut_ref(ref_to_mut_ref(crazy_nested_ref).0) = new;
}

struct ContainsImmutRef<'a, T>(&'a T);

#[kani::proof]
pub fn test_very_nested() {
    very_nested::<i32>(42, 10);
    very_nested::<Box<bool>>(Box::new(true), Box::new(true));
    very_nested::<Vec<usize>>(vec![1, 2], vec![1, 2, 3]);
}
