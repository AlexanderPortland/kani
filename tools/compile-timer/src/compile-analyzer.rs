// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
use std::{cmp::max, fs::File, io, path::PathBuf, time::Duration};

use clap::Parser;
mod common;
use serde_json::Deserializer;

use crate::common::AggrResult;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct AnalyzerArgs {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    path_pre: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    path_post: PathBuf,

    #[arg(short, long)]
    only_markdown: bool,

    #[arg(short, long)]
    ignore: Vec<PathBuf>,
}

fn main() {
    let c = AnalyzerArgs::parse();

    let (pre, post) = try_read_files(&c).unwrap();

    let (pre_ser, post_ser) = (Deserializer::from_reader(pre), Deserializer::from_reader(post));

    let results = pre_ser
        .into_iter::<AggrResult>()
        .filter_map(Result::ok)
        .zip(post_ser.into_iter::<AggrResult>().filter_map(Result::ok))
        .collect::<Vec<_>>();

    if c.only_markdown {
        print_markdown(results.as_slice());
    } else {
        print_terminal(results.as_slice());
    }
}

fn print_terminal(results: &[(AggrResult, AggrResult)]) {
    let krate_column_len = results
        .iter()
        .map(|(a, b)| max(a.krate_mini_path().len(), b.krate_mini_path().len()))
        .max()
        .unwrap();

    for (pre_res, post_res) in results {
        assert!(pre_res.krate == post_res.krate);
        let pre_time = pre_res.info.iqr.avg;
        let post_time = post_res.info.iqr.avg;

        let change_dir = if post_time > pre_time {
            "↑"
        } else if post_time == pre_time {
            "-"
        } else {
            "↓"
        };
        let change_amount = (pre_time.abs_diff(post_time).as_micros() as f64
            / post_time.as_micros() as f64)
            * 100_f64;

        println!(
            "krate {:krate_column_len$} -- [{:.2?} => {:.2?} ({change_dir}{change_amount:5.2}%)] {:?}",
            pre_res.krate_mini_path(),
            pre_time,
            post_time,
            Verdict::from_results(pre_res, post_res)
        );
    }
}

fn print_markdown(results: &[(AggrResult, AggrResult)]) {
    println!("# Compiletime Results");
    println!("| test crate | old compile time | new compile time | diff | verdict |");
    println!("| - | - | - | - | - |");
    for (pre_res, post_res) in results {
        assert!(pre_res.krate == post_res.krate);
        let pre_time = pre_res.info.iqr.avg;
        let post_time = post_res.info.iqr.avg;

        let change_dir = if post_time > pre_time {
            "↑"
        } else if post_time == pre_time {
            "-"
        } else {
            "↓"
        };

        let change_amount = (pre_time.abs_diff(post_time).as_micros() as f64
            / post_time.as_micros() as f64)
            * 100_f64;

        println!(
            "| {} | {:.2?} | {:.2?} | {change_dir} {:.2?} ({change_amount:.2}%) | {:?} |",
            pre_res.krate_mini_path(),
            pre_time,
            post_time,
            pre_time.abs_diff(post_time),
            Verdict::from_results(pre_res, post_res)
        );
    }
}

#[derive(Debug)]
#[allow(dead_code)]
enum Verdict {
    Improved,
    ProbablyNoise(NoiseExplanation),
    PotentialRegression(std::time::Duration, std::time::Duration),
}

#[derive(Debug)]
#[allow(dead_code)]
enum NoiseExplanation {
    SmallComparedToStdDev(std::time::Duration),
    SmallPercentageChange,
}

impl Verdict {
    fn from_results(pre: &AggrResult, post: &AggrResult) -> Self {
        let pre_time = pre.info.iqr.avg;
        let post_time = post.info.iqr.avg;
        if pre.info.iqr.avg > post.info.iqr.avg {
            return Self::Improved;
        }

        let avg_std_dev = (pre.info.iqr() + post.info.iqr()) / 2;
        let std_dev_thresh = fraction_of_duration(avg_std_dev, 1.5);
        if post_time.abs_diff(pre_time) < std_dev_thresh {
            return Self::ProbablyNoise(NoiseExplanation::SmallComparedToStdDev(std_dev_thresh));
        }

        if post_time.abs_diff(pre_time) < fraction_of_duration(pre_time, 0.01) {
            return Self::ProbablyNoise(NoiseExplanation::SmallPercentageChange);
        }

        Self::PotentialRegression(std_dev_thresh, post_time.abs_diff(pre_time))
    }
}

fn fraction_of_duration(dur: Duration, frac: f64) -> Duration {
    Duration::from_nanos(((dur.as_nanos() as f64) * frac) as u64)
}

fn try_read_files(c: &AnalyzerArgs) -> io::Result<(File, File)> {
    let pre = File::open(c.path_pre.canonicalize()?)?;
    let post = File::open(c.path_post.canonicalize()?)?;

    io::Result::Ok((pre, post))
}

// fn try_parse_files(files: ()) -> io::Result<(Vec<)> {

// }
