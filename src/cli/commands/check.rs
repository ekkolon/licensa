// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use crate::cli::flags::Flags;
use crate::cli::Step;
use crate::cli::UnwrapOrExit;
use crate::utils::console::{Logger, Line};
use crate::io::{DocumentSnapshot, DocumentState, TreeBuilder, TreeSnapshot};
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
            Line::new("Checked {count} source file(s) in {duration}")
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

impl TreeSnapshot {
    fn to_single_line(&self, sep: Option<String>) -> String {
        let sep = sep.as_deref().unwrap_or("; ");

        let passed = format!(
            "{} passed",
            self.count_licensed().to_string().green().bold()
        );
        let untracked = format!(
            "{} untracked",
            self.count_unlicensed().to_string().yellow().bold()
        );
        let failed_color = if self.count_failed() > 0 {
            Color::Red
        } else {
            Color::BrightBlack
        };
        let failed = format!(
            "{} failed",
            self.count_failed().to_string().color(failed_color).bold()
        );

        // This would print for example: "27 ok; 1 untracked; 0 failed"
        format!("{passed}{sep}{untracked}{sep}{failed}",)
    }
}

fn log_unlicensed(logger: &mut Logger<'_>, snapshots: &[DocumentSnapshot]) -> Result<()> {
    logger.write_line("Unlicensed files detected:")?;
    logger.write_line(Line::new("(use \"licensa add <file...>\" to insert headers)").indent(2))?;

    let lines = snapshots.iter().map(|s| {
        Line::new("unlicensed:   {path}")
            .bind("path", s.path().display())
            .colored(Color::Yellow)
            .indent(6)
    });

    logger.write_lines(lines)?;
    Ok(())
}

fn log_failed(logger: &mut Logger<'_>, snapshots: &[DocumentSnapshot]) -> Result<()> {
    logger.write_line(
        Line::new("Some files could not be analyzed ({count}):").bind("count", snapshots.len()),
    )?;
    logger.write_line(
        Line::new("(use \"licensa add <file...>\" after fixing the issues)").indent(2),
    )?;

    let lines = snapshots.iter().flat_map(|s| match s {
        DocumentSnapshot::Failed { path, reason } => vec![
            Line::new("failed:   {path}")
                .bind("path", path.display())
                .colored(Color::Red)
                .indent(6),
            Line::new("reason:   {reason}")
                .bind("reason", reason)
                .indent(8),
        ],
        _ => vec![],
    });

    logger.write_lines(lines)?;
    Ok(())
}
