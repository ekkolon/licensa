// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use crate::ops::scan::{Scan, ScanConfig};
use crate::ops::stats::{WorkTreeRunnerStatistics, WorkTreeRunnerStatus};
use crate::ops::work_tree::{FileTaskResponse, WorkTree};
use crate::template::has_copyright_notice;

use anyhow::Result;
use clap::Args;
use rayon::prelude::*;

use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub fn run(args: &VerifyArgs) -> anyhow::Result<()> {
    let mut runner_stats = WorkTreeRunnerStatistics::new("verify", "found");

    let workspace_root = current_dir()?;
    let workspace_config = args.config.clone().with_workspace_config(&workspace_root)?;

    // ========================================================
    // Scanning process
    // ========================================================
    let candidates = scan_workspace(&workspace_root)?;

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

    let candidates: Vec<PathBuf> = scan
        .run()
        .into_iter()
        .par_bridge()
        .map(|entry| entry.abspath)
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
