use std::{borrow::Borrow, collections::HashMap};

use rustc_public::mir::mono::MonoItem;

use crate::kani_middle::{codegen_units::{CodegenUnit, Harness}, reachability::CallGraph};

pub trait OrderHeuristic<T> {
    fn eval(&self, val: &T) -> usize;
}

pub struct ReachableItems(HashMap<Harness, (Vec<MonoItem>, CallGraph)>);

impl ReachableItems {
    pub fn empty() -> Self {
        ReachableItems(HashMap::new())
    }

    pub fn insert(&mut self, harness: Harness, reachability_info: (Vec<MonoItem>, CallGraph)) {
        self.0.insert(harness, reachability_info).expect("shouldn't already be in there...");
    }
}

impl OrderHeuristic<Harness> for ReachableItems {
    fn eval(&self, val: &Harness) -> usize {
        self.0.get(val).expect("not FOUND!").0.len()
    }
}

impl OrderHeuristic<CodegenUnit> for ReachableItems {
    fn eval(&self, val: &CodegenUnit) -> usize {
        val.harnesses.iter().map(|harness| self.eval(harness)).sum()
    }
}