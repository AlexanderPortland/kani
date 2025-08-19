// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Utilities for optimizing the order of Kani codegen.
//!
//! When compiling with more than a single thread, the order in which we codegen harnesses can have
//! an non-negligable impact on performance. Specifically, if we handle harness that will generate
//! a lot of code near the end of compilation, the main compiler thread can get stuck waiting for
//! worker threads to export that code, slowing down overall compilation.
//!
//! This module currently provides a simple [MostReachableItems] heuristic to combat that, but more
//! complex heuristics might be able to improve on this or avoid other kinds of pitfalls.
use rustc_public::mir::mono::MonoItem;

use crate::{codegen_cprover_gotoc::HarnessWithReachable, kani_middle::reachability::{CallGraph, CollectedNode, Node, ReachabilityInfo}};

/// Orders harnesses within a [CodegenUnit](crate::kani_middle::codegen_units::CodegenUnit) based on
/// **the raw number of items found during reachability analysis**, putting those with more first.
///
/// The number of reachable items seems to be a good proxy for the amount of code we will generate and
/// thus how long both codegen and the goto file exporting will take. Putting the harnesses that will take
/// the longest first ensures that
pub struct MostReachableItems;

impl CodegenHeuristic for MostReachableItems {
    fn evaluate_harness(harness: &HarnessWithReachable) -> usize {
        harness.1.reachable.len()
    }
}

pub trait CodegenHeuristic {
    /// Evaluate and rate a harness based on the given heuristic (where *higher is better*).
    fn evaluate_harness(harness: &HarnessWithReachable) -> usize;
}

fn reorder_harnesses<'a, H: CodegenHeuristic>(
    harnesses: &mut Vec<HarnessWithReachable<'a>>,
) {
    // Sort is ascending by default, so `usize::MAX - ...` ensures higher rated harnesses come first.
    // We don't care about stability, and for cheap heuristic fns like the one for `MostReachableItems`,
    // caching isn't likely to make a difference.
    // for h in harnesses.iter() {
    //     size_of_reach_info(&h.1);
    // }
    // println!("thread {:?} sorting {:?} harnesses in codegen unit", std::thread::current().id(), harnesses.len());
    harnesses.sort_unstable_by_key(|harness| usize::MAX - H::evaluate_harness(harness));
}

/// Simple trait extender to allow us to call `.apply_...()` on the right kind of iterators.
/// Could also just be implemented as a function, but this matches the iterator style now used
/// for reachability in `codegen_crate`.
pub trait HeuristicOrderable: Iterator {
    fn apply_ordering_heuristic<T: CodegenHeuristic>(self) -> impl Iterator<Item = Self::Item>;
}

impl<'a, I, T> HeuristicOrderable for I
where
    I: Iterator<Item = (Vec<HarnessWithReachable<'a>>, T)>,
{
    /// Apply an codegen ordering heuristic to an iterator over codegen units.
    fn apply_ordering_heuristic<H: CodegenHeuristic>(self) -> impl Iterator<Item = I::Item> {
        // Reorder harnesses within each codegen unit according to `T`.
        
        let v = self.map(|mut harnesses| {
            reorder_harnesses::<H>(&mut harnesses.0);
            harnesses
        })
        .collect::<Vec<_>>();
// println!("thread {:?} applyied heuristic to {:?} units", std::thread::current().id(), v.len());
        v.into_iter()
    }
}

fn size_of_reach_info(r: &ReachabilityInfo) -> usize {
    let ReachabilityInfo { starting, reachable, call_graph } = r;

    let mut total_bytes = 0;

    total_bytes += starting.len() * size_of::<MonoItem>();
    total_bytes += reachable.len() * size_of::<MonoItem>();

    let mut call_graph_bytes = 0;

    let CallGraph {
        nodes, 
        edges, 
        back_edges} = call_graph;

    call_graph_bytes += nodes.len() * size_of::<Node>();
    for (k, v) in edges {
        call_graph_bytes += size_of_val(k) + v.len() * size_of::<CollectedNode>();
    }

    for (k, v) in back_edges {
        call_graph_bytes += size_of_val(k) + v.len() * size_of::<CollectedNode>();
    }

    total_bytes += call_graph_bytes;

    println!("{call_graph_bytes:?} from callgraph, {total_bytes:?} total");

    total_bytes
}
