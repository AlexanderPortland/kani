// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
//! The actual `Irep` structure, and associated constructors, getters, and setters.

use super::super::MachineModel;
use super::super::goto_program::{Location, Type};
use super::{IrepId, ToIrep};
use crate::cbmc_string::InternedString;
use crate::linear_map;
use linear_map::LinearMap;
use num::BigInt;
use std::fmt::Debug;

/// The CBMC serialization format for goto-programs.
/// CBMC implementation code is at:
/// <https://github.com/diffblue/cbmc/blob/develop/src/util/irep.h>
#[derive(Clone, Debug, PartialEq)]
pub struct Irep<'i> {
    pub id: IrepId,
    pub sub: Vec<Irep<'i>, &'i bumpalo::Bump>,
    pub named_sub: LinearMap<IrepId, Irep<'i>>,
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
impl<'i> Irep<'i> {
    pub fn with_location(
        self,
        l: &'i Location,
        mm: &MachineModel,
        arena: &'i bumpalo::Bump,
    ) -> Self {
        if !l.is_none() {
            self.with_named_sub(IrepId::CSourceLocation, l.to_irep(mm, arena))
        } else {
            self
        }
    }

    /// Adds a `comment` sub to the irep.
    /// Note that there might be comments both on the irep itself and
    /// inside the location sub of the irep.
    pub fn with_comment<T: Into<InternedString>>(self, c: T) -> Self {
        let a = Irep::just_string_id(c, self.sub.allocator());
        self.with_named_sub(IrepId::Comment, a)
    }

    pub fn with_named_sub(mut self, key: IrepId, value: Irep<'i>) -> Self {
        if !value.is_nil() {
            self.named_sub.insert(key, value);
        }
        self
    }

    pub fn with_named_sub_option(self, key: IrepId, value: Option<Irep<'i>>) -> Self {
        match value {
            Some(value) => self.with_named_sub(key, value),
            _ => self,
        }
    }

    pub fn with_type(self, t: &'i Type, mm: &MachineModel, arena: &'i bumpalo::Bump) -> Self {
        self.with_named_sub(IrepId::Type, t.to_irep(mm, arena))
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
impl<'i> Irep<'i> {
    /// `__attribute__(constructor)`. Only valid as a function return type.
    /// <https://gcc.gnu.org/onlinedocs/gcc-4.7.0/gcc/Function-Attributes.html>
    pub fn constructor(arena: &'i bumpalo::Bump) -> Irep<'i> {
        Irep::just_id(IrepId::Constructor, arena)
    }

    pub fn empty(arena: &'i bumpalo::Bump) -> Irep<'i> {
        Irep::just_id(IrepId::Empty, arena)
    }

    pub fn just_bitpattern_id<T>(i: T, width: u64, signed: bool, arena: &'i bumpalo::Bump) -> Irep
    where
        T: Into<BigInt>,
    {
        Irep::just_id(IrepId::bitpattern_from_int(i, width, signed), arena)
    }

    pub fn just_id(id: IrepId, arena: &'i bumpalo::Bump) -> Irep {
        Irep { id, sub: Vec::new_in(arena), named_sub: LinearMap::new() }
    }

    pub fn just_int_id<T>(i: T, arena: &'i bumpalo::Bump) -> Irep
    where
        T: Into<BigInt>,
    {
        Irep::just_id(IrepId::from_int(i), arena)
    }
    pub fn just_named_sub(
        named_sub: LinearMap<IrepId, Irep<'i>>,
        arena: &'i bumpalo::Bump,
    ) -> Irep<'i> {
        Irep { id: IrepId::EmptyString, sub: Vec::new_in(arena), named_sub }
    }

    pub fn just_string_id<T: Into<InternedString>>(s: T, arena: &'i bumpalo::Bump) -> Irep {
        Irep::just_id(IrepId::from_string(s), arena)
    }

    pub fn just_sub(sub: Vec<Irep<'i>, &'i bumpalo::Bump>) -> Irep<'i> {
        Irep { id: IrepId::EmptyString, sub, named_sub: LinearMap::new() }
    }

    pub fn nil(arena: &'i bumpalo::Bump) -> Irep {
        Irep::just_id(IrepId::Nil, arena)
    }

    pub fn one(arena: &'i bumpalo::Bump) -> Irep {
        Irep::just_id(IrepId::Id1, arena)
    }

    pub fn zero(arena: &'i bumpalo::Bump) -> Irep {
        Irep::just_id(IrepId::Id0, arena)
    }

    pub fn tuple(sub: Vec<Irep<'i>, &'i bumpalo::Bump>) -> Self {
        Irep {
            id: IrepId::Tuple,
            named_sub: linear_map![(IrepId::Type, Irep::just_id(IrepId::Tuple, sub.allocator()))],
            sub,
        }
    }
}
