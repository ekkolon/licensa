// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use crate::cli::flags::Flags;
use crate::cli::Step;
use crate::cli::UnwrapOrExit;
use crate::console::Line;
use crate::console::Logger;
use crate::io::tree::{DocumentSnapshot, DocumentState, TreeBuilder, TreeSnapshot};
use crate::workspace::LicensaManifest;
use crate::Result;

use clap::Args;
use colored::*;
use serde::Serialize;
use std::env::{self};
use std::time::Instant;

#[derive(Args, Debug, Serialize)]
pub struct CheckStep {
    #[command(flatten)]
    config: LicensaManifest,

    /// Specifies the command or subcommand to execute.
    #[command(flatten)]
    flags: Flags,
}

impl Step for CheckStep {
    fn run(&mut self) -> Result<()> {
        let mut logger = Logger::init();
        let time = Instant::now();

        let src_root = env::current_dir()?;
        let config = &self
            .config
            .merge_into_existing_at_path(&src_root)
            .unwrap_or_exit();

        let tree = TreeBuilder::new(&src_root)
            .exclude(config.exclude.to_vec())?
            .build()?;

        let info = tree.read_license_info()?;
        let time_end = logger.human_duration(time);
        logger.write_line(
            Line::new("Checked license headers on {count} files. Done in {duration}")
                .bind("count", info.count())
                .bind("duration", time_end),
        )?;

        let outstats = info.to_single_line(None);
        logger.write_line(Line::new(outstats).indent(2))?;
        logger.line_break()?;

        if info.count_failed() > 0 {
            let failed = &info.get_state(DocumentState::Failed);
            log_failed(&mut logger, failed)?;
        }

        if info.count_unlicensed() > 0 {
            let unlicensed = &info.get_state(DocumentState::Unlicensed);
            log_unlicensed(&mut logger, unlicensed)?;
        }

        Ok(())
    }
}

const DEFAULT_STAT_FRAGMENT_SEP: &str = "; ";

impl TreeSnapshot {
    fn to_single_line(&self, sep: Option<String>) -> String {
        let (passed_count, passed_suffix) =
            (&self.count_licensed().to_string().green().bold(), "passed");

        let (untracked_count, untracked_suffix) = (
            &self.count_unlicensed().to_string().yellow().bold(),
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

fn log_unlicensed(logger: &mut Logger<'_>, snapshots: &[DocumentSnapshot]) -> Result<()> {
    logger.display("You workspace contains unlicensed files:")?;
    logger.write_line(
        Line::new("(use \"licensa add <glob...>\" to apply license headers)").indent(2),
    )?;

    let lines: Vec<Line> = snapshots
        .iter()
        .map(|snapshot| {
            Line::new("unlicensed:   {path}")
                .bind("path", snapshot.path().display())
                .colored(Color::Red)
                .indent(8)
        })
        .collect();

    logger.write_lines(lines)?;
    Ok(())
}

fn log_failed(logger: &mut Logger<'_>, snapshots: &[DocumentSnapshot]) -> Result<()> {
    logger.write_line(Line::new(
        "The following {count} files could not be checked:",
    ))?;

    logger.write_line(
        Line::new("(use \"licensa add <glob...>\" to apply license headers)").indent(2),
    )?;

    let lines: Vec<Line> = snapshots
        .iter()
        .flat_map(|failed| match failed {
            DocumentSnapshot::Failed { path, reason } => {
                let main_line = Line::new("failed:   {path}")
                    .bind("path", path.display())
                    .colored(Color::Red)
                    .indent(8);

                let reason_line = Line::new("reason: {reason}")
                    .bind("reason", reason)
                    .indent(10);

                vec![main_line, reason_line]
            }
            _ => vec![],
        })
        .collect();

    logger.write_lines(lines)?;

    Ok(())
}
