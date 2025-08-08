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

#[allow(private_interfaces)]
pub trait CodegenCacheEl
where
    Self: Sized,
{
    type Key: Hash + Eq;
    fn get_individual_cache(cache: &CodegenCache) -> &HashImpl<Self::Key, Self>;
    fn get_individual_cache_mut(cache: &mut CodegenCache) -> &mut HashImpl<Self::Key, Self>;
    fn get_individual_cache_stats(cache: &mut CodegenCache) -> &mut CacheStats;
}

impl Drop for CodegenCache {
    fn drop(&mut self) {
        self.calc_size();
        self.print_stats();
    }
}

#[derive(Default, Copy, Clone)]
struct CacheStats {
    hits: usize,
    misses: usize,
}

impl std::fmt::Debug for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total = self.hits + self.misses;
        let hit_rate = self.hits as f64 / total as f64 * 100_f64;
        write!(f, " {} hits / {total} queries ({:.2?}%)", self.hits, hit_rate)
    }
}

impl std::ops::AddAssign for CacheStats {
    fn add_assign(&mut self, rhs: Self) {
        self.hits += rhs.hits;
        self.misses += rhs.misses;
    }
}

macro_rules! generate_cache {
    ($name:tt -- $($(@$global:tt)? [$field_name:tt] $key:path => $el:path),+) => {
        #[derive(Default)]
        pub struct $name {
            $($field_name: (HashImpl<$key, $el>, CacheStats),)*
        }

        impl $name {
            #[allow(dead_code)]
            #[cfg(debug_assertions)]
            fn print_stats(&self) {
                let mut total = CacheStats::default();
                println!("\n***CACHE STATS***");
                $(
                    let name = type_name::<$el>();
                    let stats = self.$field_name.1;
                    println!("{name}: {:?}", stats);
                    total += stats;
                )*
                println!("\nTOTAL: {:?}\n", total);
            }

            #[allow(dead_code)]
            #[cfg(not(debug_assertions))]
            fn print_stats(&self) {
                println!("[cache stats are turned off in release mode]");
            }

            #[allow(dead_code)]
            fn calc_size(&self) {
                let mut total = 0;
                $(
                    let key_size = size_of::<$key>();
                    let el_size = size_of::<$el>();
                    let cap = self.$field_name.0.capacity();
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
                    clear_cache!($($global)? cache, $el);
                )*
            })
        }

        $(
            #[allow(private_interfaces)]
            impl CodegenCacheEl for $el {
                type Key = $key;
                fn get_individual_cache(cache: &CodegenCache) -> &HashImpl<Self::Key, Self> {
                    &cache.$field_name.0
                }
                fn get_individual_cache_mut(cache: &mut CodegenCache) -> &mut HashImpl<Self::Key, Self> {
                    &mut cache.$field_name.0
                }
                fn get_individual_cache_stats(cache: &mut CodegenCache) -> &mut CacheStats {
                    &mut cache.$field_name.1
                }
            }
        )*
    };
}

// TODO: add more granularity here...
macro_rules! clear_cache {
    ( $cache:tt, $el:path) => {
        <$el as CodegenCacheEl>::get_individual_cache_mut($cache).clear();
    };
    (global $cache:tt, $el:path) => {
        /* global field, don't clear cache */
    };
}

generate_cache!(CodegenCache --
            [types] rustc_public::ty::Ty              => cbmc::goto_program::Type,
    @global [spans] SpanWrapper                       => cbmc::goto_program::Location,
            [abis ] rustc_public::mir::mono::Instance => rustc_public::abi::FnAbi
);

// TODO: add rvalues for sure...

pub struct FinalEntry<T: CodegenCacheEl>(Option<T>, T::Key);

#[cfg(debug_assertions)]
pub fn update_stats<E: CodegenCacheEl + Clone>(cache_hit: bool) {
    CACHE.with_borrow_mut(|cache| match cache_hit {
        true => E::get_individual_cache_stats(cache).hits += 1,
        false => E::get_individual_cache_stats(cache).misses += 1,
    })
}

#[cfg(not(debug_assertions))]
#[inline(always)]
pub fn update_stats<E: CodegenCacheEl + Clone>(cache_hit: bool) {}

pub fn cache_entry<E: CodegenCacheEl + Clone>(key: E::Key) -> FinalEntry<E> {
    let found_value = CACHE.with_borrow(|cache| E::get_individual_cache(cache).get(&key).cloned());

    update_stats::<E>(found_value.is_some());

    FinalEntry(found_value, key)
}

// pub struct FinalEntryRef<'a, T: CodegenCacheEl>(std::collections::hash_map::Entry<'a, T::Key, T>);

// pub fn cache_ref<E: CodegenCacheEl>(cache: &mut CodegenCache, key: E::Key) -> FinalEntryRef<'_, E> {
//     FinalEntryRef(E::get_individual_cache_mut(cache).entry(key))
// }

// pub fn operate_on_cache_ref<E: CodegenCacheEl, U, I: FnOnce() -> E, F: FnOnce(&E) -> U>(
//     key: E::Key,
//     init: I,
//     and_op: F,
// ) -> U {
//     CACHE.with_borrow_mut(|cache| and_op(cache_ref(cache, key).or_insert_with(init)))
// }

// impl<'a, T: CodegenCacheEl> FinalEntryRef<'a, T> {
//     pub fn or_insert_with<F: FnOnce() -> T>(self, f: F) -> &'a T {
//         self.0.or_insert_with(f)
//     }
// }

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
