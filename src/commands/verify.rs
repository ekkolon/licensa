// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::config::Config;
use crate::ops::scan::{is_candidate, Scan, ScanConfig};
use crate::ops::stats::{WorkTreeRunnerStatistics, WorkTreeRunnerStatus};
use crate::ops::work_tree::{FileTaskResponse, WorkTree};
use crate::template::has_copyright_notice;
use crate::workspace::walker::WalkBuilder;

use anyhow::Result;
use clap::Args;
use rayon::prelude::*;

use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub fn run(args: &mut VerifyArgs) -> anyhow::Result<()> {
    let mut runner_stats = WorkTreeRunnerStatistics::new("verify", "found");

    let workspace_root = current_dir()?;
    let config = &args.config.with_workspace_config(&workspace_root)?;

    // ========================================================
    // Scanning process
    // ========================================================

    let mut walk_builder = WalkBuilder::new(&workspace_root);
    walk_builder.exclude(config.exclude.clone())?;

    let mut walker = walk_builder.build()?;
    walker.quit_while(|res| res.is_err());
    walker.send_while(|res| is_candidate(res.unwrap()));
    walker.max_capacity(None);

    let candidates = walker
        .run_task()
        .iter()
        .par_bridge()
        .into_par_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<PathBuf>>();

    runner_stats.set_items(candidates.len());

    // ========================================================
    // File processing
    // ========================================================
    let runner_stats = Arc::new(Mutex::new(runner_stats));

    let context = ScanContext {
        root: workspace_root,
        runner_stats: runner_stats.clone(),
    };

    let mut worktree = WorkTree::new();
    worktree.add_task(context, read_entry);
    worktree.run(candidates);

    // ========================================================
    // Print output statistics
    let mut runner_stats = runner_stats.lock().unwrap();
    runner_stats.set_status(WorkTreeRunnerStatus::Ok);
    runner_stats.print(true);

    Ok(())
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    #[command(flatten)]
    config: Config,
}

#[derive(Clone)]
struct ScanContext {
    pub root: PathBuf,
    pub runner_stats: Arc<Mutex<WorkTreeRunnerStatistics>>,
}

// FIXME: Refactor to more generic, re-usable fn
fn scan_workspace<P>(workspace_root: P) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let scan_config = ScanConfig {
        // FIXME: Add limit to workspace config
        limit: 100,
        // FIXME: Use exclude from workspace config
        exclude: None,
        root: workspace_root.as_ref().to_path_buf(),
    };

    let scan = Scan::new(scan_config);
    let candidates = scan
        .find_candidates()
        .par_iter()
        .map(|e| e.path().to_path_buf())
        .collect();
    Ok(candidates)
}

fn read_entry(context: &mut ScanContext, response: &FileTaskResponse) {
    let mut runner_stats = context.runner_stats.lock().unwrap();
    if has_copyright_notice(response.content.as_bytes()) {
        runner_stats.add_action_count();
    } else {
        runner_stats.add_ignore();
    }
}

use std::time::{Duration, Instant};

fn measure_time<F, T>(func: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = func();
    let duration = start.elapsed();
    (result, duration)
}

fn measure_time_result<F, T>(func: F) -> Result<(T, Duration)>
where
    F: FnOnce() -> Result<T>,
{
    let start = Instant::now();
    let result = func()?;
    let duration = start.elapsed();
    Ok((result, duration))
}
