// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT
use std::{cmp::max, fs::File, io, path::PathBuf};

use clap::Parser;
mod common;
use serde_json::Deserializer;

use crate::common::AggrResult;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    path_pre: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    path_post: PathBuf,
}

fn main() {
    let c = Cli::parse();

    // println!("have {c:?}");

    let (pre, post) = try_read_files(&c).unwrap();

    // println!("pre is {:?}", pre);

    let (pre_ser, post_ser) = (Deserializer::from_reader(pre), Deserializer::from_reader(post));
    // let iter = post_ser.into_iter::<AggrResult>();
    // println!("first is {:?}", iter.peekable().peek().unwrap());
    let iter = pre_ser
        .into_iter::<AggrResult>()
        .filter_map(Result::ok)
        .zip(post_ser.into_iter::<AggrResult>().filter_map(Result::ok))
        .collect::<Vec<_>>();

    let krate_column_len = iter
        .iter()
        .map(|(a, b)| max(a.krate_mini_path().len(), b.krate_mini_path().len()))
        .max()
        .unwrap();
    println!("printing shit...");
    for (pre_res, post_res) in &iter {
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
            "krate {:krate_column_len$} -- [{:.2?} => {:.2?} ({change_dir}{change_amount:5.2}%)]",
            pre_res.krate_mini_path(),
            pre_time,
            post_time
        );
    }
}

fn try_read_files(c: &Cli) -> io::Result<(File, File)> {
    let pre = File::open(c.path_pre.canonicalize()?)?;
    let post = File::open(c.path_post.canonicalize()?)?;

    io::Result::Ok((pre, post))
}

// fn try_parse_files(files: ()) -> io::Result<(Vec<)> {

// }
