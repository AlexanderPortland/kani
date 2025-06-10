// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
use super::Symbol;
use crate::InternedString;
use std::collections::BTreeMap;

/// A direct implementation of the CBMC serilization format for symbol tables implemented in
/// <https://github.com/diffblue/cbmc/blob/develop/src/util/symbol_table.h>
#[derive(Debug, PartialEq)]
pub struct SymbolTable<'i> {
    pub symbol_table: BTreeMap<InternedString, Symbol<'i>>,
}

/// Constructors
impl<'i> Default for SymbolTable<'i> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'i> SymbolTable<'i> {
    pub fn new() -> SymbolTable<'i> {
        SymbolTable { symbol_table: BTreeMap::new() }
    }
}

/// Setters
impl<'i> SymbolTable<'i> {
    pub fn insert(&mut self, symbol: Symbol<'i>) {
        self.symbol_table.insert(symbol.name, symbol);
    }
}
