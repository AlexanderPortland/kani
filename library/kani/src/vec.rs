// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::{Arbitrary, any, any_where};

/// Generates an arbitrary vector whose length is at most MAX_LENGTH.
pub fn any_vec<T, const MAX_LENGTH: usize>() -> Vec<T>
where
    T: Arbitrary,
{
    let real_length: usize = any_where(|sz| *sz <= MAX_LENGTH);
    match real_length {
        0 => vec![],
        exact if exact == MAX_LENGTH => exact_vec::<T, MAX_LENGTH>(),
        _ => {
            let mut any_vec = exact_vec::<T, MAX_LENGTH>();
            any_vec.truncate(real_length);
            any_vec.shrink_to_fit();
            assert!(any_vec.capacity() == any_vec.len());
            any_vec
        }
    }
}

/// Generates an arbitrary vector that is exactly EXACT_LENGTH long.
pub fn exact_vec<T, const EXACT_LENGTH: usize>() -> Vec<T>
where
    T: Arbitrary,
{
    let boxed_array: Box<[T; EXACT_LENGTH]> = Box::new(any());
    <[T]>::into_vec(boxed_array)
}

#[inline(always)]
fn force_partition_well_formedness<T: Arbitrary, F>(partition_conditions: impl IntoIterator<Item = F>, fn_to_verify: impl FnOnce(T))  where F: FnOnce(&T) {
    // doesn't do anything, but just makes sure input is well formed, or this won't type-check
}

// well-formedness:
//   1. conditions should be of type FnOnce(&T) -> bool
//   2. where the closure to verify is of type impl FnOnce(T)

// proof harness 1


// proof harness 2

#[inline(always)]
pub fn partition<T: Arbitrary, F>(partition_conditions: impl IntoIterator<Item = F>, closure_to_verify: impl FnOnce(T)) where F: FnOnce(&T) -> bool {
    let input = T::any();
    let missing_full_coverage = partition_conditions.into_iter().any(|condition: F| condition(&input));

    assert!(missing_full_coverage, "kani::partition conditions don't have full coverage of input type {:?}", std::any::type_name::<T>());

    // closure_to_verify()
}