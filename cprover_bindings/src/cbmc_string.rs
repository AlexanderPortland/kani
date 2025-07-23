// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use lazy_static::lazy_static;
use std::cell::RefCell;
use std::sync::Mutex;
use string_interner::{StringInterner, Symbol};
use string_interner::backend::StringBackend;
use string_interner::symbol::SymbolU32;

/// This class implements an interner for Strings.
/// CBMC objects to have a large number of strings which refer to names: symbols, files, etc.
/// These tend to be reused many times, which causes signifcant memory usage.
/// If we intern the strings, each unique string is only allocated once, saving memory.
/// On the stdlib test, interning led to a 15% savings in peak memory usage.
/// Since they're referred to by index, InternedStrings become `Copy`, which simplifies APIs.
/// The downside is that interned strings live the lifetime of the execution.
/// So you should only intern strings that will be used in long-lived data-structures, not temps.
///
/// We use a single global string interner, which is protected by a Mutex (i.e. threadsafe).
/// To create an interned string, either do
/// `let i : InternedString = s.into();` or
/// `let i = s.intern();`
#[derive(Clone, Hash, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InternedString(SymbolU32);

#[derive(Debug)]
struct InternStats {
    resolve_hits: usize,
    resolve_miss: usize,
    get_hits: usize,
    get_miss: usize,
}
impl Default for InternStats {
    fn default() -> Self {
        InternStats { resolve_hits: 0, resolve_miss: 0, get_hits: 0, get_miss: 0 }
    }
}

#[derive(Default)]
struct InternerWrapper(StringInterner<StringBackend>, pub RefCell<InternStats>);

impl InternerWrapper {
    fn resolve_infalliable(&self, symbol: SymbolU32) -> &str {
        self.0.resolve(symbol).expect("dont use this if it can fail")
    }

    fn get_or_intern<T: AsRef<str>>(&mut self, string: T) -> SymbolU32 {
        if let Some(symbol) = self.0.get(&string) {
            self.1.borrow_mut().get_hits += 1;
            symbol
        } else {
            self.1.borrow_mut().get_miss += 1;
            // println!("STATS NOW {:?} on interner of len {:?}", self.1, self.0.len());
            self.0.get_or_intern(string)
        }
    }
}

// Use a `Mutex` to make this thread safe.
lazy_static! {
    static ref INTERNER: Mutex<InternerWrapper> =
        Mutex::new(InternerWrapper::default());
}

impl InternedString {
    // No resets:
    // STATS NOW RefCell { value: InternStats { resolve_hits: 0, resolve_miss: 0, get_hits: 33945043, get_miss: 58624 } } on interner of len 58623
    pub fn time_clone() {
        let a = &mut INTERNER.lock().unwrap().0;
        let start = std::time::Instant::now();
        let b = std::hint::black_box(a.clone());
        println!("took {:?} to clone interner", start.elapsed());

        // INTERNER.lock().unwrap().0 = StringInterner::default();
    }

    pub fn snapshot_interner() -> StringInterner<StringBackend> {
        INTERNER.lock().unwrap().0.clone()
    }

    pub fn override_interner(interner: StringInterner<StringBackend>) {
        let real = &mut INTERNER.lock().unwrap().0;
        let old_len = real.len();
        let new_len = interner.len();
        println!("overwrite from len {old_len} -> {new_len}");
        *real = interner;
    }

    pub fn is_empty(&self) -> bool {
        self.map(|s| s.is_empty())
    }

    pub fn len(&self) -> usize {
        self.map(|s| s.len())
    }

    /// Apply the function `f` to the interned string, represented as an &str.
    /// Needed because exporting the &str backing the InternedString is blocked by lifetime rules.
    /// Instead, this allows users to operate on the &str when needed.
    pub fn map<T, F: FnOnce(&str) -> T>(&self, f: F) -> T {
        f(INTERNER.lock().unwrap().resolve_infalliable(self.0))
    }

    pub fn starts_with(&self, pattern: &str) -> bool {
        self.map(|s| s.starts_with(pattern))
    }
}

impl std::fmt::Display for InternedString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", INTERNER.lock().unwrap().resolve_infalliable(self.0))
    }
}
/// Custom-implement Debug, so our debug logging contains meaningful strings, not numbers
impl std::fmt::Debug for InternedString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", INTERNER.lock().unwrap().resolve_infalliable(self.0))
    }
}

impl<T> From<T> for InternedString
where
    T: AsRef<str>,
{
    fn from(s: T) -> InternedString {
        InternedString(INTERNER.lock().unwrap().get_or_intern(s))
    }
}

impl<T> PartialEq<T> for InternedString
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        INTERNER.lock().unwrap().resolve_infalliable(self.0) == other.as_ref()
    }
}

pub trait InternString {
    fn intern(self) -> InternedString;
}

impl<T> InternString for T
where
    T: Into<InternedString>,
{
    fn intern(self) -> InternedString {
        self.into()
    }
}
pub trait InternStringOption {
    fn intern(self) -> Option<InternedString>;
}

impl<T> InternStringOption for Option<T>
where
    T: Into<InternedString>,
{
    fn intern(self) -> Option<InternedString> {
        self.map(|s| s.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::cbmc_string::InternedString;

    #[test]
    fn test_string_interner() {
        let a: InternedString = "A".into();
        let b: InternedString = "B".into();
        let aa: InternedString = "A".into();

        assert_eq!(a, aa);
        assert_ne!(a, b);
        assert_ne!(aa, b);

        assert_eq!(a, "A");
        assert_eq!(b, "B");
        assert_eq!(aa, "A");
    }
}
