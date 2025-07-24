// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use lazy_static::lazy_static;
use std::cell::RefCell;
use std::sync::Mutex;
use string_interner::backend::StringBackend;
use string_interner::symbol::SymbolU32;
use string_interner::{StringInterner, Symbol};

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

// TODO: DONT use a `Mutex` to make this thread safe.
thread_local! {
    static INTERNER: RefCell<InternerWrapper> =
        RefCell::new(InternerWrapper::default());
}

/// [InternedString] is defined based on the thread local [INTERNER] and so cannot be safely
/// sent between threads.
impl !Send for InternedString {}

/// A type that is only [!Send] because it contains [InternedString]s. This forces users to annotate that
/// the types they want to wrap in [WithInterner] are `!Send` just for that specific reason rather than
/// using it to make arbitrary types `Send`.
pub unsafe trait ContainsInternedString {}

/// Since [WithInterner<T>] guarantees that the inner `T` cannot be accessed without updating the
/// thread local [INTERNER] to a copy of what was used to generate `T`, it is safe to send between threads,
/// even if the inner `T` contains [InternedString]s which are not [Send] on their own.
unsafe impl<T: ContainsInternedString> Send for WithInterner<T> {}

/// A type [T] bundled with the [StringInterner] that was used to generate it.
///
/// The only way to access the inner `T` is by calling `into_iter()`, which will automatically
/// update the current thread's interner to the interner used the generate `T`,
/// ensuring interner coherence between the sending & receiving threads.
pub struct WithInterner<T> {
    interner: StringInterner<StringBackend>,
    inner: T,
}

impl<T> WithInterner<T> {
    /// Create a new wrapper with a given `interner` and `inner`.
    pub fn new(interner: StringInterner<StringBackend>, inner: T) -> Self {
        WithInterner { interner, inner }
    }

    /// Create a new wrapper of `inner` with a clone of the current thread local [INTERNER].
    pub fn new_with_current(inner: T) -> Self {
        let interner = INTERNER.with_borrow(|i| i.0.clone());
        WithInterner { interner, inner }
    }

    /// Get the inner wrapped `T` and implicitly update the current thread local [INTERNER] with a
    /// copy of the one used to generate `T`.
    pub fn into_inner(self) -> T {
        INTERNER.with_borrow_mut(|i| i.0 = self.interner);
        self.inner
    }
}

impl InternedString {
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
        INTERNER.with_borrow(|i| f(i.resolve_infalliable(self.0)))
    }

    pub fn starts_with(&self, pattern: &str) -> bool {
        self.map(|s| s.starts_with(pattern))
    }
}

impl std::fmt::Display for InternedString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        INTERNER.with_borrow(|i| write!(fmt, "{}", i.resolve_infalliable(self.0)))
    }
}
/// Custom-implement Debug, so our debug logging contains meaningful strings, not numbers
impl std::fmt::Debug for InternedString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        INTERNER.with_borrow(|i| write!(fmt, "{:?}", i.resolve_infalliable(self.0)))
    }
}

impl<T> From<T> for InternedString
where
    T: AsRef<str>,
{
    fn from(s: T) -> InternedString {
        InternedString(INTERNER.with_borrow_mut(|i| i.get_or_intern(s)))
    }
}

impl<T> PartialEq<T> for InternedString
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        INTERNER.with_borrow(|i| i.resolve_infalliable(self.0) == other.as_ref())
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
