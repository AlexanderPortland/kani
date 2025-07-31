// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    codegen_cprover_gotoc::HarnessWithReachable, kani_middle::transform::BodyTransformation,
};

pub struct MostReachableItems;

impl CodegenHeuristic for MostReachableItems {
    fn evaluate_harness(harness: &HarnessWithReachable) -> usize {
        harness.1.len()
    }
}

pub trait CodegenHeuristic {
    fn evaluate_harness(harness: &HarnessWithReachable) -> usize;
}

fn reorder_harnesses<'a, H: CodegenHeuristic>(
    harnesses: &mut Vec<(HarnessWithReachable<'a>, BodyTransformation)>,
) {
    let start = std::time::Instant::now();
    harnesses.sort_unstable_by_key(|(harness, _)| usize::MAX - H::evaluate_harness(harness));
    println!("sorted elements in {:?}", start.elapsed());
}

pub trait HeuristicOrderable: Iterator {
    fn apply_ordering_heuristic<T: CodegenHeuristic>(self) -> impl Iterator<Item = Self::Item>;
}

pub fn print_harness_heuristic(harness: HarnessWithReachable) -> HarnessWithReachable {
    println!(
        "\t[!] harness {:?} -- heuristic is {:?}",
        harness.0.trimmed_name(),
        MostReachableItems::evaluate_harness(&harness)
    );
    harness
}

impl<'a, I> HeuristicOrderable for I
where
    I: Iterator<Item = Vec<(HarnessWithReachable<'a>, BodyTransformation)>>,
{
    fn apply_ordering_heuristic<H: CodegenHeuristic>(self) -> impl Iterator<Item = I::Item> {
        // Sort each codegen unit according to `T`.
        self.map(|mut harnesses| {
            reorder_harnesses::<H>(&mut harnesses);
            harnesses
        })
    }
}
