// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use crate::io::tree::{ReadTreeStatistics, Tree};
use crate::terminal::{self, Step};
use crate::workspace::Config;
use crate::Result;

use clap::Args;
use colored::*;
use std::env::{self};

#[derive(Args, Debug)]
pub struct CheckArgs {
    #[command(flatten)]
    config: Config,
}

pub fn run(args: &mut CheckArgs) -> Result<()> {
    let mut terminal = terminal::Task::new("Verify SPDX License headers");
    terminal.start()?;

    let src_root = env::current_dir()?;
    let config = &args.config.merge_into_existing_at_path(&src_root)?;

    let exclude = Some(config.exclude.to_vec());
    let tree = Tree::new(&src_root);
    let entries = tree.find_license_candidates(exclude)?;
    let stats = tree.read_license_info(&entries)?;

    terminal.finish_ok()?;

    terminal.print(format!(
        "Checked license headers on {} files. Done in {}",
        stats.count_checked(),
        terminal.human_duration()?
    ));
    terminal.line_break();

    let outstats = stats.to_single_line(None);
    terminal.with_indent(2);
    terminal.logln(outstats);
    terminal.line_break();

    if stats.count_untracked() > 0 {
        terminal.with_indent(0);
        terminal.logln("You workspace contains unlicensed files:");

        terminal.with_indent(2);
        terminal.logln("(use \"licensa add <glob...>\" to apply license headers)");

        let mut untracked_files = stats.untracked().to_vec();

        untracked_files.sort_by(|a, b| a.to_str().unwrap_or("").cmp(b.to_str().unwrap_or("")));

        terminal.with_indent(8);
        untracked_files.iter().for_each(|path| {
            let rel_path = path.strip_prefix(&src_root).unwrap();
            let msg = format!("unlicensed:   {}", rel_path.display());
            terminal.logln(msg.red().to_string());
        });
    }

    Ok(())
}

const DEFAULT_STAT_FRAGMENT_SEP: &str = "; ";

impl ReadTreeStatistics {
    fn to_single_line(&self, sep: Option<String>) -> String {
        let (passed_count, passed_suffix) =
            (&self.count_passed().to_string().green().bold(), "passed");

        let (untracked_count, untracked_suffix) = (
            &self.count_untracked().to_string().yellow().bold(),
            "untracked",
        );

        // Highlight failed fragments if there is at least 1 failed task.
        let (failed_count, failed_suffix) = match self.count_failed() > 0 {
            true => (&self.count_failed().to_string().red().bold(), "failed"),
            false => (&self.count_failed().to_string().dimmed().bold(), "failed"),
        };

        let seperator = sep.as_deref().unwrap_or(DEFAULT_STAT_FRAGMENT_SEP);

        let passed = format!("{passed_count} {passed_suffix}");
        let untracked = format!("{untracked_count} {untracked_suffix}");
        let failed = format!("{failed_count} {failed_suffix}");

        // This would print for example: "27 ok; 1 untracked; 0 failed"
        format!("{passed}{seperator}{untracked}{seperator}{failed}",)
    }
}
