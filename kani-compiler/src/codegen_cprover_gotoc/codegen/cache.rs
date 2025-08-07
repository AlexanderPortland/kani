// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{cell::RefCell, hash::Hash};

use fxhash::FxHashMap;
use std::any::type_name;

use crate::codegen_cprover_gotoc::context::SpanWrapper;

thread_local! {
    pub static CACHE: RefCell<CodegenCache> = RefCell::new(Default::default());
}

type HashImpl<K, V> = FxHashMap<K, V>;

pub trait CodegenCacheEl
where
    Self: Sized,
{
    type Key: Hash + Eq;
    fn get_individual_cache(cache: &CodegenCache) -> &HashImpl<Self::Key, Self>;
    fn get_individual_cache_mut(cache: &mut CodegenCache) -> &mut HashImpl<Self::Key, Self>;
}

// impl Drop for CodegenCache {
//     fn drop(&mut self) {
//         self.calc_size();
//     }
// }

macro_rules! generate_cache {
    ($name:tt -- $($(@$global:tt)? [$field_name:tt] $key:path => $el:path),+) => {
        #[derive(Default)]
        pub struct $name {
            $($field_name: HashImpl<$key, $el>,)*
        }

        impl $name {
            #[allow(dead_code)]
            fn calc_size(&self) {
                let mut total = 0;
                $(
                    let key_size = size_of::<$key>();
                    let el_size = size_of::<$el>();
                    let cap = self.$field_name.capacity();
                    let field_total = (key_size + el_size) * cap;
                    println!("for {:?} map -- key: {key_size}, el: {el_size}, cap: {cap} => {field_total}", type_name::<$el>());
                    total += field_total;
                )*
                println!("{total} in total!");
            }
        }

        pub fn clear_codegen_cache() {
            CACHE.with_borrow_mut(|cache|{
                $(
                    clear_cache!($($global)? cache, $field_name);
                )*
            })
        }

        $(
            impl CodegenCacheEl for $el {
                type Key = $key;
                fn get_individual_cache(cache: &CodegenCache) -> &HashImpl<Self::Key, Self> {
                    &cache.$field_name
                }
                fn get_individual_cache_mut(cache: &mut CodegenCache) -> &mut HashImpl<Self::Key, Self> {
                    &mut cache.$field_name
                }
            }
        )*
    };
}

// TODO: add more granularity here...
macro_rules! clear_cache {
    ( $cache:tt, $field_name:tt) => {
        $cache.$field_name.clear();
    };
    (global $cache:tt, $field_name:tt) => {
        /* global field, don't clear cache */
    };
}

generate_cache!(CodegenCache --
            [types] rustc_public::ty::Ty => cbmc::goto_program::Type,
    @global [spans] SpanWrapper => cbmc::goto_program::Location
);

pub struct FinalEntry<T: CodegenCacheEl>(Option<T>, T::Key);

pub fn cache_entry<E: CodegenCacheEl + Clone>(key: E::Key) -> FinalEntry<E> {
    let found_value = CACHE.with_borrow(|cache| E::get_individual_cache(cache).get(&key).cloned());
    FinalEntry(found_value, key)
}

impl<E: CodegenCacheEl> FinalEntry<E> {
    pub fn tweak<F: FnOnce(&mut E)>(mut self, f: F) -> FinalEntry<E> {
        if let Some(found_val) = &mut self.0 {
            f(found_val)
        }

        self
    }
}

impl<E: CodegenCacheEl + Clone> FinalEntry<E> {
    pub fn or_insert_with<F: FnOnce() -> E>(self, f: F) -> E {
        match self.0 {
            Some(cached) => cached,
            None => {
                let calculated = f();
                CACHE.with_borrow_mut(|cache| {
                    E::get_individual_cache_mut(cache).insert(self.1, calculated.clone())
                });
                calculated
            }
        }
    }
}
