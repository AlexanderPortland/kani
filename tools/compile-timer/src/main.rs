// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![feature(exit_status_error)]

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{
    process::{Command, Stdio},
    time::Duration,
};

use serde::{Deserialize, Serialize};

/// We need at least one warm-up run to make sure crates are fetched & cached in
/// the local `.cargo/registry` folder. Otherwise the first run will be unduly slower.
const WARMUP_RUNS: usize = 1;
const TIMED_RUNS: usize = 10;

fn main() {
    let current = std::env::current_dir().expect("should be run in a directory");
    let mut to_visit = vec![current];
    let mut res = Vec::new();
    let run_start = std::time::Instant::now();
    let mut out_file = File::create("compile-timer.json").unwrap();

    while let Some(next) = to_visit.pop() {
        let _p = next.canonicalize().unwrap();
        let toml = next.canonicalize().unwrap().join("Cargo.toml");
        // dbg!(&toml);
        // println!("p is {p:?} next is {next:?}");

        if toml.exists() && toml.is_file() {
            // in rust directory so we profile that jawn
            println!("[!] profiling in {next:?}");
            let new_res = profile_on_crate(&next).into_result(next);
            out_file.write_all(serde_json::to_string_pretty(&new_res).unwrap().as_bytes()).unwrap();
            out_file.write_all("\n".as_bytes()).unwrap();
            res.push(new_res);
        } else {
            let a = std::fs::read_dir(next)
                .unwrap()
                .filter_map(|entry| match entry {
                    Err(_) => None,
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() { Some(path) } else { None }
                    }
                })
                .collect::<Vec<_>>();
            // println!("recurr in {:?}", a);
            to_visit.extend(a);
        }
    }

    println!("[!] total info is {:?}", aggregate_aggregates(&res));
    print!("\t [*] run took {:?}", run_start.elapsed());
}

#[derive(Debug, Serialize, Deserialize)]
struct AggrResult {
    pub krate: String,
    info: AggrInfo,
}

fn _print_to_file(aggr: &AggrInfo) {
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("out.txt")
        .unwrap();

    let s = serde_json::to_string_pretty(&aggr).unwrap();
    f.write_all(s.as_bytes()).unwrap();
}

fn profile_on_crate(absolute_path: &std::path::PathBuf) -> AggrInfo {
    let _warmup_results = (0..WARMUP_RUNS)
        .map(|i| {
            print!("\t[*] running warmup {}/{WARMUP_RUNS}", i + 1);
            let _ = std::io::stdout().flush();
            let res = run_command_in(absolute_path);
            println!(" -- {res:?}");
            res
        })
        .collect::<Vec<_>>();

    let timed_results = (0..TIMED_RUNS)
        .map(|i| {
            print!("\t[*] running timed run {}/{TIMED_RUNS}", i + 1);
            let _ = std::io::stdout().flush();
            let res = run_command_in(absolute_path);
            println!(" -- {res:?}");
            res
        })
        .collect::<Vec<_>>();

    let aggr = aggregate_results(&timed_results);
    println!("\t[!] results for {absolute_path:?} are in! {aggr:?}");

    aggr
}

type RunResult = Duration;
fn run_command_in(absolute_path: &PathBuf) -> RunResult {
    // println!("running in {:?}", absolute_path);
    // `cargo clean` to ensure the compiler is run again
    let _ = Command::new("cargo")
        .current_dir(absolute_path)
        .arg("clean")
        .stdout(Stdio::null())
        .output()
        .expect("cargo clean should succeed");

    // do the actual run
    let kani_output = Command::new("cargo")
        .current_dir(absolute_path)
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

fn aggregate_aggregates(info: &[AggrResult]) -> (Duration, Duration) {
    for i in info {
        println!("krate {} -- {:?}", i.krate, i.info.iqr.avg);
    }

    (info.iter().map(|i| i.info.iqr.avg).sum(), info.iter().map(|i| i.info.iqr.std_dev).sum())
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct AggrInfo {
    pub iqr: Stats,
    full: Stats,
}

#[derive(Debug, Serialize, Deserialize)]
struct Stats {
    pub avg: Duration,
    pub std_dev: Duration,
    pub range: (Duration, Duration),
}

// enum RunQuality {
//     VeryGood,
//     Good,
//     Passable,
//     Noisy,
// }

impl AggrInfo {
    // fn classify_quality(&self) -> RunQuality {
    //     match (self.full.std_dev.as_micros() as f64) / (self.full.std_dev.as_micros() as f64) {
    //         d if d < 0.01 => RunQuality::VeryGood,
    //         d if d < 0.05 => RunQuality::Good,
    //         d if d < 0.1 => RunQuality::Passable,
    //         _ => RunQuality::Noisy,
    //     }
    // }

    fn into_result(self, krate: PathBuf) -> AggrResult {
        AggrResult { krate: format!("{krate:?}"), info: self }
    }

    // fn to_string(&self) -> String {
    //     format!("")
    // }
}

fn aggregate_results(results: &[Duration]) -> AggrInfo {
    assert!(results.len() == TIMED_RUNS);

    let mut sorted = results.to_vec();
    sorted.sort();
    let iqr_bounds = (0.25 * results.len() as f64, 0.75 * results.len() as f64);
    let iqr_durations = sorted
        .into_iter()
        .enumerate()
        .filter_map(|(i, v)| {
            if i >= iqr_bounds.0 as usize && i <= iqr_bounds.1 as usize { Some(v) } else { None }
        })
        .collect::<Vec<Duration>>();

    println!("iqr durations are {iqr_durations:?}");

    AggrInfo { iqr: result_stats(&iqr_durations), full: result_stats(results) }
}

fn result_stats(results: &[Duration]) -> Stats {
    let avg = results.iter().sum::<Duration>() / results.len().try_into().unwrap();
    let range = (*results.iter().min().unwrap(), *results.iter().max().unwrap());

    let deviations = results.iter().map(|x| x.abs_diff(avg).as_micros().pow(2)).sum::<u128>();
    let std_dev =
        Duration::from_micros((deviations / results.len() as u128).isqrt().try_into().unwrap());

    Stats { avg, std_dev, range }
}

#[allow(dead_code)]
enum PotentialIssue {
    ColdBuildCache,
}

// fn sniff_test(
//     warmup_results: &[Duration],
//     timed_results: &[Duration],
//     _aggr: &AggrInfo,
// ) -> Vec<PotentialIssue> {
//     let issues = Vec::new();

//     println!("warm ups {warmup_results:?}");
//     println!("timed {timed_results:?}");

//     issues
// }

#[cfg(test)]
mod test {}
