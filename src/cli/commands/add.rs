// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::flags::Flags;
use crate::cli::Exit;
use crate::cli::Step;
use crate::cli::UnwrapOrExit;
use crate::cli::UnwrapOrExitWith;
use crate::io::Tree;
use crate::io::{DocumentSnapshot, DocumentState};
use crate::utils::console::{Line, Logger};
use crate::workspace::LicensaManifest;
use crate::workspace::LicenseConfig;
use crate::Error;
use crate::Result;

use clap::Parser;
use colored::Color;
use licensa_flags::dry_run;
use serde::Serialize;

#[dry_run(false)]
#[derive(Parser, Debug, Serialize)]
pub struct AddStep {
    #[command(flatten)]
    manifest: LicensaManifest,

    #[command(flatten)]
    flags: Flags,
}

impl Step for AddStep {
    fn run(&mut self) -> Result<()> {
        let mut logger = Logger::init();

        if self.dry_run {
            logger.write_line("Running in dry-run mode (no files will be modified).")?;
            logger.line_break()?;
        }

        let mut config = self.manifest.clone();

        let src_root = std::env::current_dir()?;
        let config = config
            .merge_into_existing_at_path(&src_root)
            .unwrap_or_exit();

        // Verify required fields such es `license`, `owner` and `format` are set.
        let _license = config
            .license()
            .unwrap_or_exit_with(Error::MissingRequiredArgument("-t, --type <LICENSE>"));

        let _license_owner = config
            .holder()
            .unwrap_or_exit_with(Error::MissingRequiredArgument("-o, --owner <OWNER>"));

        let config = serde_json::to_value(config)
            .map_err(Error::Json)
            .unwrap_or_exit();

        let config: LicenseConfig = serde_json::from_value(config)
            .map_err(Error::Json)
            .unwrap_or_exit();

        let tree_snapshot = Tree::builder(&src_root)
            .set_dry_run(self.dry_run)
            .exclude(config.exclude.to_vec())?
            .build()
            .unwrap_or_exit()
            .update_license_info(&config)
            .unwrap_or_exit();

        let modified_docs = tree_snapshot.get_state(DocumentState::Modified);

        log_modified(&mut logger, &modified_docs, self.dry_run).unwrap_or_else(|err| err.exit());

        Ok(())
    }
}

fn log_modified(
    logger: &mut Logger<'_>,
    snapshots: &[DocumentSnapshot],
    dry_run: bool,
) -> Result<()> {
    let num_modified = snapshots.len();

    match dry_run {
        true => {
            if num_modified == 0 {
                logger.write_line("No files require license updates.")?;
            } else {
                logger.write_line(
                    Line::new("Detected {count} file(s) needing license headers:")
                        .bind("count", num_modified),
                )?;
                logger.write_line(
                    Line::new("Run without --dry-run to apply the changes.").indent(2),
                )?;
            }
        }
        false => {
            if num_modified == 0 {
                logger.write_line("All files already contain valid license headers.")?;
            } else {
                logger.write_line(
                    Line::new("Added license headers to {count} file(s):")
                        .bind("count", num_modified),
                )?;
            }
        }
    }

    if num_modified > 0 {
        let log_line = |snapshot: &DocumentSnapshot| {
            Line::new("modified:   {path}")
                .bind("path", snapshot.path().display())
                .colored(Color::Green)
                .indent(8)
        };

        logger.write_lines(snapshots.iter().map(log_line))?;
    }

    if dry_run && num_modified > 0 {
        logger.line_break()?;
        logger.write_line(Line::new("No changes written (dry run)."))?;
    }

    Ok(())
}
