// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Args;

use std::{
    env::current_dir,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::config::resolve_workspace_config;
use crate::config::Config;
use crate::copyright_notice::contains_copyright_notice;
use crate::workspace::scan::{Scan, ScanConfig};
use crate::workspace::stats::{WorkTreeRunnerStatistics, WorkTreeRunnerStatus};
use crate::workspace::work_tree::{FileTaskResponse, WorkTree};

use rayon::prelude::*;

pub fn run(args: &VerifyArgs) -> anyhow::Result<()> {
    let mut runner_stats = WorkTreeRunnerStatistics::new("verify", "found");

    let workspace_root = std::env::current_dir()?;
    let workspace_config = args.to_config()?;

    // ========================================================
    // Scanning process
    // ========================================================

    let scan_config = ScanConfig {
        // FIXME: Add limit to workspace config
        limit: 100,
        // FIXME: Use exclude from workspace config
        exclude: None,
        root: workspace_root.clone(),
    };
    let scan = Scan::new(scan_config);

    let candidates: Vec<PathBuf> = scan
        .run()
        .into_iter()
        .par_bridge()
        .map(|entry| entry.abspath)
        .collect();

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
pub struct VerifyArgs {}

impl VerifyArgs {
    // Merge self with config::Config
    fn to_config(&self) -> Result<Config> {
        let workspace_root = current_dir()?;
        let config = resolve_workspace_config(workspace_root)?;
        Ok(config)
    }
}

#[derive(Clone)]
struct ScanContext {
    pub root: PathBuf,
    pub runner_stats: Arc<Mutex<WorkTreeRunnerStatistics>>,
}

fn read_entry(context: &mut ScanContext, response: &FileTaskResponse) {
    let mut runner_stats = context.runner_stats.lock().unwrap();
    if contains_copyright_notice(&response.content) {
        runner_stats.add_action_count();
    } else {
        runner_stats.add_ignore();
    }
}
