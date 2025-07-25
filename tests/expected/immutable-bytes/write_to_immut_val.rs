// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests that use unsafe casts to write to mutable variables through immutable references.
//!
//! Irregardless of the type or how long the chain of references is, writing through the immutable
//! reference constitutes mutating immutable bytes.

use std::mem::transmute;

fn write_to_const_ptr<T>(ptr: *const T, val: T) {
    // Transmute through an intermediate `usize` type so the compiler doens't notice that
    // we're violating mutability and complain
    unsafe {
        let ptr = transmute::<usize, *mut T>(transmute::<*const T, usize>(ptr));
        *ptr = val;
    }
}

#[kani::proof]
pub fn test_write_to_local() {
    write_to_local::<i32>(42, 10);
    write_to_local::<Box<bool>>(Box::new(true), Box::new(true));
    write_to_local::<Vec<usize>>(vec![1, 2], vec![1, 2, 3]);
}

pub fn write_to_local<T>(old: T, new: T) {
    let local: T = old;
    write_to_const_ptr(&local, new);
}

static STATIC_I32: i32 = 42;
static STATIC_ARRAY: [usize; 3] = [1, 2, 3];
static STATIC_STR: &str = "hello";

#[kani::proof]
pub fn test_write_to_static() {
    write_to_const_ptr(&STATIC_I32, 10);
    write_to_const_ptr(&STATIC_ARRAY, [3, 2, 1]);
    write_to_const_ptr(&STATIC_STR, "hi!");
}

#[kani::proof]
pub fn write_element_to_immut_array() {
    let local = [1, 2, 3];

    write_to_const_ptr(local.as_ptr(), 10);
}

#[kani::proof]
pub fn write_to_immut_local_array_offset_within() {
    let local = [1, 2, 3];

    let offset_ptr = unsafe { local.as_ptr().offset(1) };
    write_to_const_ptr::<i32>(offset_ptr, 10);
}

#[kani::proof]
pub fn write_to_immut_local_array_offset_past() {
    let local: [i32; 3] = [1, 2, 3];
    let local2: [i32; 4] = [1, 2, 3, 4];

    // Make up a new pointer to match with the type of local2.
    let hallucinated_ptr = unsafe { transmute::<*const [i32; 3], *const [i32; 4]>(&local) };
    write_to_const_ptr::<[i32; 4]>(hallucinated_ptr, local2);
}
