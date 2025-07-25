// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests that use unsafe casts to write to mutable variables through immutable references.
//!
//! Irregardless of the type or how long the chain of references is, writing through the immutable
//! reference constitutes mutating immutable bytes.

use std::mem::transmute;

// TODO: should this just take the bytes that are directly pointed to by `ptr` rather than cloning? bc by cloning
// you're presumably making a new heap allocation for certain collections, so you'll still be overwriting `*ptr`
// with that a pointer ot that new allocation rather than keeping it the same
fn rewrite_existing_to_const_ptr<T: Clone>(ptr: *const T) {
    // Transmute through an intermediate `usize` type so the compiler doens't notice that
    // we're violating mutability and complain
    unsafe {
        let ptr = transmute::<usize, *mut T>(transmute::<*const T, usize>(ptr));
        let old_val = (*ptr).clone();
        *ptr = old_val;
    }
}

#[kani::proof]
pub fn test_write_to_local() {
    write_to_local::<i32>(42);
    write_to_local::<Box<bool>>(Box::new(true));
    write_to_local::<Vec<usize>>(vec![1, 2]);
}

pub fn write_to_local<T: Clone>(initial: T) {
    let local: T = initial;
    rewrite_existing_to_const_ptr(&local);
}

static STATIC_I32: i32 = 42;
static STATIC_ARRAY: [usize; 3] = [1, 2, 3];
static STATIC_STR: &str = "hello";

#[kani::proof]
pub fn test_write_to_static() {
    rewrite_existing_to_const_ptr(&STATIC_I32);
    rewrite_existing_to_const_ptr(&STATIC_ARRAY);
    rewrite_existing_to_const_ptr(&STATIC_STR);
}

#[kani::proof]
pub fn write_element_to_immut_array() {
    let local = [1, 2, 3];

    rewrite_existing_to_const_ptr(local.as_ptr());
}

#[kani::proof]
pub fn write_to_immut_local_array_offset_within() {
    let local = [1, 2, 3];

    let offset_ptr = unsafe { local.as_ptr().offset(1) };
    rewrite_existing_to_const_ptr::<i32>(offset_ptr);
}

#[kani::proof]
pub fn write_to_immut_local_array_offset_past() {
    let local: [i32; 3] = [1, 2, 3];

    // Make up a new pointer to write past array bounds.
    let hallucinated_ptr = unsafe { transmute::<*const [i32; 3], *const [i32; 4]>(&local) };
    rewrite_existing_to_const_ptr::<[i32; 4]>(hallucinated_ptr);
}
