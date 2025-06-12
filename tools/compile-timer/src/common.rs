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

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AggrInfo {
    pub iqr: Stats,
    full: Stats,
}

#[allow(dead_code)]
impl AggrInfo {
    pub fn new(iqr: Stats, full: Stats) -> Self {
        AggrInfo { iqr, full }
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
