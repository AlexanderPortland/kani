// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
use std::{cmp::max, fs::File, io, path::PathBuf, time::Duration};

use clap::Parser;
mod common;
use serde_json::Deserializer;

use crate::common::{AggrResult, fraction_of_duration};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct AnalyzerArgs {
    #[arg(short, long, value_name = "FILE")]
    path_pre: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    path_post: PathBuf,

    /// Output results in markdown format
    #[arg(short, long)]
    only_markdown: bool,
}

fn main() {
    let c = AnalyzerArgs::parse();

    let (pre, post) = try_read_files(&c).unwrap();

    let (pre_ser, post_ser) = (Deserializer::from_reader(pre), Deserializer::from_reader(post));

    let pre_results = pre_ser.into_iter::<AggrResult>().collect::<Vec<_>>();
    let post_results = post_ser.into_iter::<AggrResult>().collect::<Vec<_>>();

    let mut results = pre_results.into_iter().filter_map(Result::ok).zip(post_results.into_iter().filter_map(Result::ok)).collect::<Vec<_>>();
    results.sort_by_key(|a| (signed_percent_diff(&a.0.iqr_stats.avg, &a.1.iqr_stats.avg).abs() * 1000_f64) as i64);

    if c.only_markdown {
        print_markdown(results.as_slice());
    } else {
        print_to_terminal(results.as_slice());
    }
}

// Print results for a terminal output.
fn print_to_terminal(results: &[(AggrResult, AggrResult)]) {
    let krate_column_len = results
        .iter()
        .map(|(a, b)| max(a.krate_trimmed_path.len(), b.krate_trimmed_path.len()))
        .max()
        .unwrap();

    for (pre_res, post_res) in results {
        assert!(pre_res.krate == post_res.krate);
        let pre_time = pre_res.iqr_stats.avg;
        let post_time = post_res.iqr_stats.avg;

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
            pre_res.krate_trimmed_path,
            pre_time,
            post_time,
            Verdict::from_results(pre_res, post_res)
        );
    }
}

// Print results in a markdown format (for GitHub actions).
fn print_markdown(results: &[(AggrResult, AggrResult)]) {
    println!("# Compiletime Results");
    let total_pre = results.iter().map(|i|i.0.iqr_stats.avg).sum();
    let total_post = results.iter().map(|i|i.1.iqr_stats.avg).sum();
    println!("### *on the whole: {:.2?} => {:.2?} -- {}*", total_pre, total_post, diff_string(total_pre, total_post));
    println!("| test crate | old compile time | new compile time | diff | verdict |");
    println!("| - | - | - | - | - |");
    for (pre_res, post_res) in results {
        assert!(pre_res.krate_trimmed_path == post_res.krate_trimmed_path);
        let pre_time = pre_res.iqr_stats.avg;
        let post_time = post_res.iqr_stats.avg;

        println!(
            "| {} | {:.2?} | {:.2?} | {} | {:?} |",
            pre_res.krate_trimmed_path,
            pre_time,
            post_time,
            diff_string(pre_time, post_time),
            Verdict::from_results(pre_res, post_res)
        );
    }
}

fn signed_percent_diff(pre: &Duration, post: &Duration) -> f64 {
    let change_amount = (pre.abs_diff(*post).as_micros() as f64
            / post.as_micros() as f64)
            * 100_f64;
    if (post < pre) {
        -change_amount
    } else { change_amount }
}

fn diff_string(pre: Duration, post: Duration) -> String {
    let change_dir = if post > pre {
            "$\\color{red}\textsf{↑"
        } else if post == pre {
            "$\\color{black}\textsf{-"
        } else {
            "$\\color{green}\textsf{↓"
        };
    let change_amount = signed_percent_diff(&pre, &post).abs();
    format!("{change_dir} {:.2?} ({change_amount:.2}%)}}$", pre.abs_diff(post))
}

#[derive(Debug)]
#[allow(dead_code)]
enum Verdict {
    /// This crate now compiles faster!
    Improved,
    /// This crate compiled slower, but likely because of OS noise.
    ProbablyNoise(NoiseExplanation),
    /// This crate compiled slower, potentially indicating a true performance regression.
    PotentialRegression(std::time::Duration, std::time::Duration),
}

#[derive(Debug)]
#[allow(dead_code)]
/// The reason a regression was flagged as likely noise rather than a true performance regression.
enum NoiseExplanation {
    /// The increase in compile time is so small compared to the
    /// sample's standard deviation that it is probably just sampling noise.
    SmallComparedToStdDev(std::time::Duration),
    /// The percentage increase in compile time is so small, 
    /// the difference is likely insignificant.
    SmallPercentageChange,
}

impl Verdict {
    fn from_results(pre: &AggrResult, post: &AggrResult) -> Self {
        let (pre_time, post_time) = (pre.iqr_stats.avg, post.iqr_stats.avg);

        if pre.iqr_stats.avg > post.iqr_stats.avg {
            return Self::Improved;
        }

        let avg_std_dev = (pre.full_std_dev() + post.full_std_dev()) / 2;
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

fn try_read_files(c: &AnalyzerArgs) -> io::Result<(File, File)> {
    io::Result::Ok((
        File::open(c.path_pre.canonicalize()?)?, 
        File::open(c.path_post.canonicalize()?)?
    ))
}