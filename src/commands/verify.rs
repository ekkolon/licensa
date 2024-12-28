use crate::config::Config;
use crate::ops::scan::is_candidate;
use crate::ops::stats::{WorkTreeRunnerStatistics, WorkTreeRunnerStatus};
use crate::template::has_copyright_notice;
use crate::workspace::walker::WalkBuilder;

use anyhow::Result;
use clap::Args;
use ignore::DirEntry;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::env::current_dir;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread}; // Add colored crate for color output

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

    let task = walker.run_task();

    let candidates: Vec<DirEntry> = task
        .iter()
        .par_bridge()
        .into_par_iter()
        .filter_map(Result::ok)
        .collect();

    let count: &Vec<_> = &candidates
        .par_iter()
        .progress_count(candidates.len() as u64)
        .collect();

    let pb = ProgressBar::new(0);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{msg} {spinner}")
            .unwrap(),
    );

    pb.set_message("Verifying license headers...");

    let num_candidates = candidates.len();

    runner_stats.set_items(num_candidates);

    // ========================================================
    // File processing
    // ========================================================
    let runner_stats = Arc::new(Mutex::new(runner_stats));

    // Read file as bytes vector and return its content and the path to it
    let read_file = |entry: &DirEntry| {
        fs::read(entry.path())
            .ok()
            .map(|content| (content, entry.path().to_path_buf()))
    };

    // Check existence of copyright notice and update output statistics
    let check_copyright_notice = |(ref file_contents, ref path): (Vec<u8>, PathBuf)| {
        let mut runner_stats = runner_stats.lock().unwrap();
        if has_copyright_notice(file_contents) {
            runner_stats.add_action_count();
            //println!("{} {}", "✔".green(), path.display());
        } else {
            runner_stats.add_ignore();
            //println!("{} {}", "✘".red(), path.display());
        }
    };

    candidates
        .par_iter()
        .filter_map(read_file)
        .for_each(check_copyright_notice);

    // ========================================================
    // Print output statistics
    // ========================================================

    thread::sleep(Duration::from_secs(5));
    // Finish spinner once processing is complete
    pb.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());

    pb.abandon_with_message(format!(
        "Verifying license headers... {} Done.",
        "✔".green()
    ));

    let mut runner_stats = runner_stats.lock().unwrap();
    runner_stats.set_status(WorkTreeRunnerStatus::Ok);

    println!(
        "Processed {} files in {}.\n",
        num_candidates.to_string().bold(),
        runner_stats.elapsed_time().to_string().bold()
    );

    let output = VerifyOutput {
        failed: runner_stats.count_failed(),
        ok: runner_stats.count_passed(),
        untracked: runner_stats.count_ignored(),
    };

    output.print();

    let num_ignored = runner_stats.count_ignored();
    let num_ignored_formatted = num_ignored.to_string().bold().yellow();
    if num_ignored > 0 {
        println!("\nYou have untracked files. Run `licensa add` to add license headers to them.");
    }

    // Summary message
    //println!(
    //    "\n{} files licensed, {} untracked, {} skipped.",
    //    &runner_stats.count_passed().to_string().green(),
    //    &runner_stats.count_ignored().to_string().yellow(),
    //    &runner_stats.count_ignored().to_string().black()
    //);

    Ok(())
}

struct VerifyOutput {
    failed: usize,
    untracked: usize,
    ok: usize,
}

impl VerifyOutput {
    fn print(&self) {
        let (num_failed, failed_suffix) = match self.failed > 0 {
            true => (&self.failed.to_string().red().bold(), "failed".red()),
            false => (&self.failed.to_string().dimmed().bold(), "failed".dimmed()),
        };
        let (num_untracked, untracked_suffix) = (
            &self.untracked.to_string().yellow().bold(),
            "untracked".yellow(),
        );
        let (num_passed, passed_suffix) = (&self.ok.to_string().green().bold(), "ok".green());

        let middot = "·".dimmed();
        // This would print for example:
        // 27 ok; 1 untracked; 0 failed
        println!(
            "{:>2}{} {} {middot} {} {} {middot} {} {}",
            "",
            num_passed,
            passed_suffix,
            num_untracked,
            untracked_suffix,
            num_failed,
            failed_suffix
        );
    }
}
