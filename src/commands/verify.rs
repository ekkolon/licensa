// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::config::Config;
use crate::ops::scan::is_candidate;
use crate::ops::stats::{WorkTreeRunnerStatistics, WorkTreeRunnerStatus};
use crate::template::has_copyright_notice;
use crate::workspace::walker::WalkBuilder;

use anyhow::Result;
use clap::Args;
use ignore::DirEntry;
use rayon::prelude::*;

use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Args, Debug)]
pub struct VerifyArgs {
    #[command(flatten)]
    config: Config,
}

pub fn run(args: &mut VerifyArgs) -> anyhow::Result<()> {
    let mut runner_stats = WorkTreeRunnerStatistics::new("verify", "found");

    let workspace_root = current_dir()?;
    let config = &args.config.with_workspace_config(&workspace_root)?;

    // ========================================================
    // Scanning process
    // ========================================================

    let mut walk_builder = WalkBuilder::new(&workspace_root);
    walk_builder.exclude(Some(config.exclude.clone()))?;

    let mut walker = walk_builder.build()?;
    walker
        .quit_while(|res| res.is_err())
        .send_while(|res| is_candidate(res.unwrap()))
        .max_capacity(None);

    let candidates: Vec<DirEntry> = walker
        .run_task()
        .iter()
        .par_bridge()
        .into_par_iter()
        .filter_map(Result::ok)
        .collect();

    runner_stats.set_items(candidates.len());

    // ========================================================
    // File processing
    // ========================================================
    let runner_stats = Arc::new(Mutex::new(runner_stats));

    // Read file as bytes vector and return its content and the patht to it
    let read_file = |entry: &DirEntry| {
        fs::read(entry.path())
            .ok()
            .map(|content| (content, entry.path().to_path_buf()))
    };

    // Check existence of copyright notice and update output statistices
    let check_copyright_notice = |(ref file_contents, ref path): (Vec<u8>, PathBuf)| {
        let mut runner_stats = runner_stats.lock().unwrap();
        if has_copyright_notice(file_contents) {
            runner_stats.add_action_count();
        } else {
            runner_stats.add_ignore();
        }
    };

    candidates
        .par_iter()
        .filter_map(read_file)
        .for_each(check_copyright_notice);

    // ========================================================
    // Print output statistics
    let mut runner_stats = runner_stats.lock().unwrap();
    runner_stats.set_status(WorkTreeRunnerStatus::Ok);
    runner_stats.print(true);

    Ok(())
}
