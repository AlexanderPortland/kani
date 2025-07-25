// Copyright rustc Contributors
// Adapted from rustc: https://github.com/rust-lang/rust/tree/5f98537eb7b5f42c246a52c550813c3cff336069/src/test/ui/coroutine/smoke-resume-args.rs
//
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Modifications Copyright Kani Contributors
// See GitHub history for details.

// run-pass

// revisions: default nomiropt
//[nomiropt]compile-flags: -Z mir-opt-level=0

#![feature(coroutines, coroutine_trait)]
#![feature(stmt_expr_attributes)]

use std::fmt::Debug;
use std::marker::Unpin;
use std::ops::{
    Coroutine,
    CoroutineState::{self, *},
};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

fn drain<G: Coroutine<R, Yield = Y> + Unpin, R, Y>(
    g: &mut G,
    inout: Vec<(R, CoroutineState<Y, G::Return>)>,
) where
    Y: Debug + PartialEq,
    G::Return: Debug + PartialEq,
{
    let mut g = Pin::new(g);

    for (input, out) in inout {
        assert_eq!(g.as_mut().resume(input), out);
    }
}

static DROPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, PartialEq)]
struct DropMe;

impl Drop for DropMe {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

fn expect_drops<T>(expected_drops: usize, f: impl FnOnce() -> T) -> T {
    DROPS.store(0, Ordering::SeqCst);

    let res = f();

    let actual_drops = DROPS.load(Ordering::SeqCst);
    assert_eq!(actual_drops, expected_drops);
    res
}

#[kani::proof]
#[kani::unwind(8)]
fn main() {
    drain(
        &mut #[coroutine]
        |mut b| {
            while b != 0 {
                b = yield (b + 1);
            }
            -1
        },
        vec![(1, Yielded(2)), (-45, Yielded(-44)), (500, Yielded(501)), (0, Complete(-1))],
    );

    expect_drops(2, || {
        drain(
            &mut #[coroutine]
            |a| yield a,
            vec![(DropMe, Yielded(DropMe))],
        )
    });

    expect_drops(6, || {
        drain(
            &mut #[coroutine]
            |a| yield yield a,
            vec![(DropMe, Yielded(DropMe)), (DropMe, Yielded(DropMe)), (DropMe, Complete(DropMe))],
        )
    });

    #[allow(unreachable_code)]
    expect_drops(2, || {
        drain(
            &mut #[coroutine]
            |a| yield return a,
            vec![(DropMe, Complete(DropMe))],
        )
    });

    expect_drops(2, || {
        drain(
            &mut #[coroutine]
            |a: DropMe| {
                if false { yield () } else { a }
            },
            vec![(DropMe, Complete(DropMe))],
        )
    });

    expect_drops(4, || {
        drain(
            #[allow(unused_assignments, unused_variables)]
            &mut #[coroutine]
            |mut a: DropMe| {
                a = yield;
                a = yield;
                a = yield;
            },
            vec![
                (DropMe, Yielded(())),
                (DropMe, Yielded(())),
                (DropMe, Yielded(())),
                (DropMe, Complete(())),
            ],
        )
    });
}
