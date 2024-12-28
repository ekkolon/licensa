use crate::config::Config;
use crate::ops::scan::is_candidate;
use crate::template::has_copyright_notice;
use crate::utils::format::elapsed_time_in_secs;
use crate::workspace::walker::WalkBuilder;

use anyhow::Result;
use clap::Args;
use ignore::DirEntry;
use rayon::prelude::*;

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::env::current_dir;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::{fs, thread}; // Add colored crate for color output

#[derive(Args, Debug)]
pub struct CheckArgs {
    #[command(flatten)]
    config: Config,
}

pub fn run(args: &mut CheckArgs) -> anyhow::Result<()> {
    let start_time = Instant::now();

    let pb = ProgressBar::new(0);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{msg} {spinner}")
            .unwrap(),
    );
    pb.set_message("Verifying license headers...");

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

    let num_candidates = candidates.len();

    // ========================================================
    // File processing
    // ========================================================
    let stats = check_license_headers(&candidates);

    // ========================================================
    // Print output statistics
    // ========================================================
    // Finish spinner once processing is complete
    pb.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());
    pb.abandon_with_message(format!(
        "Verifying license headers... {} Done.",
        "✔".green()
    ));

    let end_time = elapsed_time_in_secs(start_time);

    println!(
        "Processed {} files in {}.\n",
        num_candidates.to_string().bold(),
        end_time.to_string().bold()
    );

    stats.print_multi_line();

    let num_ignored_formatted = stats.untracked.to_string().bold().yellow();
    if stats.untracked > 0 {
        println!("\nYou have untracked files. Run `licensa add` to add license headers to them.");
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
            Ok(contnet) => {
                if has_copyright_notice(&contnet) {
                    num_verfied.fetch_add(1, Ordering::Relaxed);
                } else {
                    num_untracked.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(err) => {
                num_failed.fetch_add(1, Ordering::Relaxed);
            }
        });

    CommandStats {
        failed: num_failed.load(Ordering::Relaxed),
        verified: num_verfied.load(Ordering::Relaxed),
        untracked: num_untracked.load(Ordering::Relaxed),
    }
}

struct CommandStats {
    failed: usize,
    untracked: usize,
    verified: usize,
}

impl CommandStats {
    fn print_single_line(&self) {
        let (num_failed, failed_suffix) = match self.failed > 0 {
            true => (&self.failed.to_string().red().bold(), "failed".red()),
            false => (&self.failed.to_string().dimmed().bold(), "failed".dimmed()),
        };
        let (num_untracked, untracked_suffix) = (
            &self.untracked.to_string().yellow().bold(),
            "untracked".yellow(),
        );
        let (num_passed, passed_suffix) = (&self.verified.to_string().green().bold(), "ok".green());

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

    fn print_multi_line(&self) {
        let (num_failed, failed_suffix) = match self.failed > 0 {
            true => (&self.failed.to_string().red().bold(), "failed".red()),
            false => (&self.failed.to_string().dimmed().bold(), "failed".dimmed()),
        };
        let (num_untracked, untracked_suffix) = (
            &self.untracked.to_string().yellow().bold(),
            "untracked".yellow(),
        );
        let (num_passed, passed_suffix) = (&self.verified.to_string().green().bold(), "ok".green());

        // This would print for example:
        // 27 ok; 1 untracked; 0 failed
        println!("{:>2} {} {}", "✅", num_passed, passed_suffix,);
        println!("{:>2} {} {}", "🔔", num_untracked, untracked_suffix,);
        println!("{:>2} {} {}", "❌", num_failed, failed_suffix,);
    }
}
