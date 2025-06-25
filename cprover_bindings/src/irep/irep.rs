// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
//! The actual `Irep` structure, and associated constructors, getters, and setters.

use super::super::MachineModel;
use super::super::goto_program::{Location, Type};
use super::{IrepId, ToIrep};
use crate::cbmc_string::InternedString;
use crate::linear_map;
use bumpalo::Bump;
use linear_map::LinearMap;
use num::BigInt;
use std::fmt::Debug;

/// The CBMC serialization format for goto-programs.
/// CBMC implementation code is at:
/// <https://github.com/diffblue/cbmc/blob/develop/src/util/irep.h>
#[derive(Clone, Debug, PartialEq)]
pub struct Irep<'b> {
    pub id: IrepId,
    pub sub: Vec<Irep<'b>, &'b Bump>,
    pub named_sub: LinearMap<IrepId, Irep<'b>>,
}

/// Getters
impl Irep<'_> {
    pub fn lookup(&self, key: IrepId) -> Option<&Irep> {
        self.named_sub.get(&key)
    }

    pub fn lookup_as_string(&self, id: IrepId) -> Option<String> {
        self.lookup(id).and_then(|x| {
            let s = x.id.to_string();
            if s.is_empty() { None } else { Some(s) }
        })
    }
}

/// Fluent Builders
impl<'b> Irep<'b> {
    pub fn with_location(self, l: &'b Location, mm: &MachineModel) -> Self {
        let arena = *self.sub.allocator();
        if !l.is_none() {
            self.with_named_sub(IrepId::CSourceLocation, l.to_irep(arena, mm))
        } else {
            self
        }
    }

    /// Adds a `comment` sub to the irep.
    /// Note that there might be comments both on the irep itself and
    /// inside the location sub of the irep.
    pub fn with_comment<T: Into<InternedString>>(self, arena: &'b Bump, c: T) -> Irep<'b> {
        self.with_named_sub(IrepId::Comment, Irep::just_string_id(arena, c))
    }

    pub fn with_named_sub(mut self, key: IrepId, value: Irep<'b>) -> Self {
        if !value.is_nil() {
            self.named_sub.insert(key, value);
        }
        self
    }

    pub fn with_named_sub_option(self, key: IrepId, value: Option<Irep<'b>>) -> Self {
        match value {
            Some(value) => self.with_named_sub(key, value),
            _ => self,
        }
    }

    pub fn with_type(self, t: &'b Type, mm: &MachineModel) -> Self {
        let arena = *self.sub.allocator();
        self.with_named_sub(IrepId::Type, t.to_irep(arena, mm))
    }
}

/// Predicates
impl Irep<'_> {
    pub fn is_just_id(&self) -> bool {
        self.sub.is_empty() && self.named_sub.is_empty()
    }

    pub fn is_just_named_sub(&self) -> bool {
        self.id == IrepId::EmptyString && self.sub.is_empty()
    }

    pub fn is_just_sub(&self) -> bool {
        self.id == IrepId::EmptyString && self.named_sub.is_empty()
    }

    pub fn is_nil(&self) -> bool {
        self.id == IrepId::Nil
    }
}

/// Constructors
impl<'b> Irep<'b> {
    /// `__attribute__(constructor)`. Only valid as a function return type.
    /// <https://gcc.gnu.org/onlinedocs/gcc-4.7.0/gcc/Function-Attributes.html>
    pub fn constructor(arena: &'b Bump) -> Irep {
        Irep::just_id(arena, IrepId::Constructor)
    }

    pub fn empty(arena: &'b Bump) -> Irep {
        Irep::just_id(arena, IrepId::Empty)
    }

    pub fn just_bitpattern_id<T>(arena: &'b Bump, i: T, width: u64, signed: bool) -> Irep
    where
        T: Into<BigInt>,
    {
        Irep::just_id(arena, IrepId::bitpattern_from_int(i, width, signed))
    }

    pub fn just_id(arena: &'b Bump, id: IrepId) -> Irep {
        Irep { id, sub: Vec::new_in(arena), named_sub: LinearMap::new() }
    }

    pub fn just_int_id<T>(arena: &'b Bump, i: T) -> Irep
    where
        T: Into<BigInt>,
    {
        Irep::just_id(arena, IrepId::from_int(i))
    }
    pub fn just_named_sub(arena: &'b Bump, named_sub: LinearMap<IrepId, Irep<'b>>) -> Irep<'b> {
        Irep { id: IrepId::EmptyString, sub: Vec::new_in(arena), named_sub }
    }

    pub fn just_string_id<T: Into<InternedString>>(arena: &'b Bump, s: T) -> Irep {
        Irep::just_id(arena, IrepId::from_string(s))
    }

    pub fn just_sub(sub: Vec<Irep<'b>, &'b Bump>) -> Irep {
        Irep { id: IrepId::EmptyString, sub, named_sub: LinearMap::new() }
    }

    pub fn nil(arena: &'b Bump) -> Irep {
        Irep::just_id(arena, IrepId::Nil)
    }

    pub fn one(arena: &'b Bump) -> Irep {
        Irep::just_id(arena, IrepId::Id1)
    }

    pub fn zero(arena: &'b Bump) -> Irep<'b> {
        Irep::just_id(arena, IrepId::Id0)
    }

    pub fn tuple(sub: Vec<Irep<'b>, &'b Bump>) -> Self {
        Irep {
            id: IrepId::Tuple,
            named_sub: linear_map![(IrepId::Type, Irep::just_id(sub.allocator(), IrepId::Tuple))],
            sub,
        }
    }
}
