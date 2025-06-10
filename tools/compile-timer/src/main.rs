// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![feature(exit_status_error)]

use std::io::Write;
use std::{
    process::{Command, Stdio},
    time::Duration,
};

const WARMUP_RUNS: usize = 2; // should be at least one for initial extern crate business
const TIMED_RUNS: usize = 10;

fn main() {
    // println!("Doing {WARMUP_RUNS} warm up runs");
    let warmup_results = (0..WARMUP_RUNS)
        .map(|i| {
            print!("running warmup {}/{WARMUP_RUNS}", i + 1);
            let _ = std::io::stdout().flush();
            let res = run_command();
            println!(" -- {res:?}");
            res
        })
        .collect::<Vec<_>>();

    let timed_results = (0..TIMED_RUNS)
        .map(|i| {
            print!("running timed run {}/{TIMED_RUNS}", i + 1);
            let _ = std::io::stdout().flush();
            let res = run_command();
            println!(" -- {res:?}");
            res
        })
        .collect::<Vec<_>>();

    let aggr = aggregate_results(&timed_results);
    println!("results are in! {aggr:?}");

    let _sniff = sniff_test(&warmup_results, &timed_results, aggr);
}

type RunResult = Duration;
fn run_command() -> RunResult {
    // `cargo clean` to ensure the compiler is run again
    let _ = Command::new("cargo")
        .arg("clean")
        .stdout(Stdio::null())
        .output()
        .expect("cargo clean should succeed");

    // do the actual run
    let kani_output = Command::new("cargo")
        .arg("kani")
        .arg("--only-codegen")
        .env("TIME_COMPILER", "true")
        .output()
        .expect("cargo kani should succeed");

    // parse the compiler time
    let out_str = String::from_utf8(kani_output.stdout).expect("utf8 conversion should succeed");

    if !kani_output.status.success() {
        println!("outstr is {out_str:?}");
        panic!("cargo kani command failed");
    }

    out_str.split("\n").filter(|line| line.starts_with("BUILT")).map(extract_duration).sum()
}

fn extract_duration(s: &str) -> Duration {
    let micros = s.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok().unwrap();

    Duration::from_micros(micros)
}

#[allow(dead_code)]
#[derive(Debug)]
struct AggrResult {
    pub avg: Duration,
    pub std_dev: Duration,
    pub range: (Duration, Duration),
}

fn aggregate_results(results: &[Duration]) -> AggrResult {
    assert!(results.len() == TIMED_RUNS);

    let avg = results.iter().sum::<Duration>() / results.len().try_into().unwrap();
    let range = (*results.iter().min().unwrap(), *results.iter().max().unwrap());

    let deviations = results.iter().map(|x| x.abs_diff(avg).as_micros().pow(2)).sum::<u128>();
    let std_dev =
        Duration::from_micros((deviations / results.len() as u128).isqrt().try_into().unwrap());

    AggrResult { avg, std_dev, range }
}

#[allow(dead_code)]
enum PotentialIssue {
    ColdBuildCache,
}

fn sniff_test(
    warmup_results: &[Duration],
    timed_results: &[Duration],
    _aggr: AggrResult,
) -> Vec<PotentialIssue> {
    let issues = Vec::new();

    println!("warm ups {warmup_results:?}");
    println!("timed {timed_results:?}");

    issues
}

#[cfg(test)]
mod test {}
