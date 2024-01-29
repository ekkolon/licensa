// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use clap::Args;

#[derive(Args, Debug)]
pub struct VerifyArgs {}

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::copyright_notice::contains_copyright_notice;
use crate::workspace::scan::{Scan, ScanConfig};
use crate::workspace::stats::ScanStats;
use crate::workspace::work_tree::{FileTaskResponse, WorkTree};
use crate::{helpers::channel_duration::ChannelDuration, logger::notice};
use colored::Colorize;

use rayon::prelude::*;

#[derive(Clone)]
struct ScanContext {
    pub root: PathBuf,
    pub stats: Arc<Mutex<ScanStats>>,
}

pub fn build(args: &VerifyArgs) -> anyhow::Result<()> {
    // only: DEBUG
    let mut channel_duration = ChannelDuration::new();

    let root = std::env::current_dir()?;

    // ========================================================
    // Scanning process
    // ========================================================

    let scan_config = ScanConfig {
        limit: 100,
        exclude: None,
        root: root.clone(),
    };

    let scan = Scan::new(scan_config);

    let candidates: Vec<PathBuf> = scan
        .run()
        .into_iter()
        .par_bridge()
        .map(|entry| entry.abspath)
        .collect();

    let num_candidates = candidates.len();

    // ========================================================

    // ========================================================
    // File processing
    // ========================================================

    let stats = Arc::new(Mutex::new(ScanStats::new()));

    let context = ScanContext {
        root,
        stats: stats.clone(),
    };

    let mut worktree = WorkTree::new();
    worktree.add_task(context, read_entry);
    worktree.run(candidates);

    // ========================================================

    // only: DEBUG
    channel_duration.drop_channel();

    let task_duration = &channel_duration.as_secs_rounded();
    notice!(format!(
        "Verifying license headers took {}secs for {:?} files",
        task_duration, num_candidates
    ));

    let num_candidates_skipped = &stats.lock().unwrap().skipped;
    let num_candidates_without_license = num_candidates - num_candidates_skipped;
    notice!(format!(
        "Missing: {}   Skipped: {}",
        num_candidates_without_license, num_candidates_skipped
    ));

    Ok(())
}

fn read_entry(context: &mut ScanContext, response: &FileTaskResponse) {
    if contains_copyright_notice(&response.content) {
        context.stats.lock().unwrap().skip();
    }
}
