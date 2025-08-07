use std::hash::Hash;

use std::collections::hash_map::Entry as HashMapEntry;

use cbmc::goto_program::{Location, Type};
use fxhash::FxHashMap;

use crate::codegen_cprover_gotoc::context::SpanWrapper;

// #[derive(Default)]
// pub struct CodegenCache {

//     types: FxHashMap<rustc_public::ty::Ty, Type>,
//     spans: FxHashMap<SpanWrapper, Location>,
// }

type HashImpl<K, V> = FxHashMap<K, V>;

pub trait CodegenCacheEl
where
    Self: Sized,
{
    type Key: Hash + Eq;
    fn get_individual_cache(cache: &mut CodegenCache) -> &mut HashImpl<Self::Key, Self>;
}

// impl CodegenCacheEl for Type {
//     type Key = rustc_public::ty::Ty;
//     fn get_individual_cache(cache: &mut CodegenCache) -> &mut HashImpl<Self::Key, Self> {
//         &mut cache.types
//     }
// }

// impl CodegenCacheEl for Location {
//     type Key = SpanWrapper;
//     fn get_individual_cache(cache: &mut CodegenCache) -> &mut HashImpl<Self::Key, Self> {
//         &mut cache.spans
//     }
// }

macro_rules! generate_cache {
    ($name:tt -- $([$field_name:tt] $key:path => $el:path),+) => {
        #[derive(Default)]
        pub struct $name {
            $($field_name: HashImpl<$key, $el>,)*
        }

        $(
            impl CodegenCacheEl for $el {
                type Key = $key;
                fn get_individual_cache(cache: &mut CodegenCache) -> &mut HashImpl<Self::Key, Self> {
                    &mut cache.$field_name
                }
            }
        )*
    };
}

generate_cache!(CodegenCache --
    [types] rustc_public::ty::Ty => cbmc::goto_program::Type,
    [spans] SpanWrapper => cbmc::goto_program::Location
);

// you give me a key i tell you if it's there

// if it's there, you may want to modify before taking it

// if it's not, you want to insert it by running some code

// they you want a ref to the newly inserted

impl CodegenCache {
    pub fn entry<E: CodegenCacheEl + Clone>(&mut self, key: E::Key) -> Entry<'_, E::Key, E> {
        Entry(E::get_individual_cache(self).entry(key))
    }

    pub fn entry_ref<E: CodegenCacheEl>(&mut self, key: E::Key) -> EntryRef<'_, E::Key, E> {
        EntryRef(E::get_individual_cache(self).entry(key))
    }
}

// an entry that we inevitably want to clone
struct Entry<'a, K, V: Clone>(HashMapEntry<'a, K, V>);
struct EntryRef<'a, K, V>(HashMapEntry<'a, K, V>);

// an [Entry] that will have a function applied
struct TweakedEntry<'a, K, V: Clone>(Option<V>, HashMapEntry<'a, K, V>);

impl<'a, K, V: Clone> EntryRef<'a, K, V> {
    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a V {
        match self.0 {
            HashMapEntry::Occupied(existing) => existing.into_mut(),
            HashMapEntry::Vacant(vacant) => vacant.insert(f()),
        }
    }
}

impl<'a, K, V: Clone> Entry<'a, K, V> {
    pub fn tweak_from_cache<F: FnOnce(&mut V)>(self, f: F) -> TweakedEntry<'a, K, V> {
        TweakedEntry(
            match &self.0 {
                HashMapEntry::Occupied(o) => {
                    let mut new_val = o.get().clone();
                    f(&mut new_val);
                    Some(new_val)
                }
                HashMapEntry::Vacant(_) => None,
            },
            self.0,
        )
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> V {
        match self.0 {
            HashMapEntry::Occupied(existing) => existing.get().clone(),
            HashMapEntry::Vacant(vacant) => vacant.insert(f()).clone(),
        }
    }
}

impl<'a, K, V: Clone> TweakedEntry<'a, K, V> {
    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> V {
        match (self.0, self.1) {
            (Some(tweaked), HashMapEntry::Occupied(_)) => tweaked,
            (None, HashMapEntry::Vacant(vacant)) => vacant.insert(f()).clone(),
            _ => unreachable!(),
        }
    }
}
