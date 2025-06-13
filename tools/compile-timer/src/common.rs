// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AggrResult {
    pub krate: PathBuf,
    pub info: AggrInfo,
}

#[allow(dead_code)]
// allowing dead code bc neither of the binaries are named `main.rs`, so the lints
// don't seem to care that we use them there...
impl AggrResult {
    pub fn new(krate: PathBuf, info: AggrInfo) -> Self {
        AggrResult { krate, info }
    }

    pub fn krate_mini_path(&self) -> String {
        format!(
            "{:?}",
            self.krate.strip_prefix(std::env::current_dir().unwrap().parent().unwrap()).unwrap()
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggrInfo {
    pub iqr: Stats, // the stats for only the 25th-75th percentile of runs on this crate
    full: Stats,    // the stats for all runs on this crate
}

#[allow(dead_code)]
// allowing dead code bc
impl AggrInfo {
    pub fn new(iqr: Stats, full: Stats) -> Self {
        AggrInfo { iqr, full }
    }

    pub fn std_dev(&self) -> Duration {
        self.full.std_dev
    }

    pub fn iqr(&self) -> Duration {
        self.iqr.range.1 - self.iqr.range.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub avg: Duration,
    pub std_dev: Duration,
    pub range: (Duration, Duration),
}

#[allow(dead_code)]
pub fn aggregate_aggregates(info: &[AggrResult]) -> (Duration, Duration) {
    for i in info {
        println!("krate {:?} -- {:?}", i.krate, i.info.iqr.avg);
    }

    (info.iter().map(|i| i.info.iqr.avg).sum(), info.iter().map(|i| i.info.iqr.std_dev).sum())
}

#[allow(dead_code)]
pub fn fraction_of_duration(dur: Duration, frac: f64) -> Duration {
    Duration::from_nanos(((dur.as_nanos() as f64) * frac) as u64)
}
