use crate::license::template::has_copyright_notice;
use crate::ops::scan::is_candidate;
use crate::terminal::{self, Step};
use crate::workspace::walker::WalkBuilder;
use crate::workspace::Config;
use crate::Result;

use clap::Args;
use colored::*;
use ignore::DirEntry;
use rayon::prelude::*;
use std::env::current_dir;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Args, Debug)]
pub struct CheckArgs {
    #[command(flatten)]
    config: Config,
}

pub fn run(args: &mut CheckArgs) -> Result<()> {
    let mut task = terminal::Task::lazy("Verify SPDX License headers");
    task.start()?;

    let workspace_root = current_dir()?;
    let config = &args.config.with_workspace_config(&workspace_root)?;

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
        .filter_map(|e| e.ok())
        .collect();

    let stats = check_license_headers(&candidates);

    task.finish_ok()?;

    task.logln(format!(
        "Checked {} files in {}",
        candidates.len().to_string().bold(),
        task.duration_in_secs()?.bold()
    ));

    task.line_break();

    let outstats = stats.to_single_line(None);
    task.logln(outstats);
    task.line_break();

    if stats.untracked > 0 {
        task.logln("You have untracked files. Run `licensa add` to add license headers to them.");
    }

    Ok(())
}

fn check_license_headers(candidates: &Vec<DirEntry>) -> CommandStats {
    let num_verfied = AtomicUsize::new(0);
    let num_untracked = AtomicUsize::new(0);
    let num_failed = AtomicUsize::new(0);

    candidates
        .par_iter()
        .for_each(|entry: &DirEntry| match fs::read(entry.path()) {
            Ok(content) => {
                if has_copyright_notice(&content) {
                    num_verfied.fetch_add(1, Ordering::Relaxed);
                } else {
                    num_untracked.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(_) => {
                num_failed.fetch_add(1, Ordering::Relaxed);
            }
        });

    CommandStats {
        failed: num_failed.load(Ordering::Relaxed),
        passed: num_verfied.load(Ordering::Relaxed),
        untracked: num_untracked.load(Ordering::Relaxed),
    }
}

struct CommandStats {
    failed: usize,
    untracked: usize,
    passed: usize,
}

const DEFAULT_STAT_FRAGMENT_SEP: &str = "; ";

impl CommandStats {
    fn to_single_line(&self, sep: Option<String>) -> String {
        let (passed_count, passed_suffix) = (&self.passed.to_string().green().bold(), "passed");

        let (untracked_count, untracked_suffix) =
            (&self.untracked.to_string().yellow().bold(), "untracked");

        // Highlight failed fragments if there is at least 1 failed task.
        let (failed_count, failed_suffix) = match self.failed > 0 {
            true => (&self.failed.to_string().red().bold(), "failed"),
            false => (&self.failed.to_string().dimmed().bold(), "failed"),
        };

        let seperator = sep.as_deref().unwrap_or(DEFAULT_STAT_FRAGMENT_SEP);

        let passed = format!("{passed_count} {passed_suffix}");
        let untracked = format!("{untracked_count} {untracked_suffix}");
        let failed = format!("{failed_count} {failed_suffix}");

        // This would print for example: "27 ok; 1 untracked; 0 failed"
        format!("{passed}{seperator}{untracked}{seperator}{failed}",)
    }
}
