use bumpalo::Bump;
use serde::Serialize;

// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
use super::Symbol;
use crate::InternedString;
use std::collections::BTreeMap;

/// A direct implementation of the CBMC serilization format for symbol tables implemented in
/// <https://github.com/diffblue/cbmc/blob/develop/src/util/symbol_table.h>
#[derive(Debug, PartialEq)]
pub struct SymbolTable<'b> {
    pub symbol_table: BTreeMap<InternedString, Symbol<'b>, &'b bumpalo::Bump>,
}

/// Constructors
// impl Default for SymbolTable {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl<'b> SymbolTable<'b> {
    pub fn new_in(arena: &'b Bump) -> SymbolTable<'b> {
        SymbolTable { symbol_table: BTreeMap::new_in(arena) }
    }
}

/// Setters
impl<'b> SymbolTable<'b> {
    pub fn insert(&mut self, symbol: Symbol<'b>) {
        self.symbol_table.insert(symbol.name, symbol);
    }
}
