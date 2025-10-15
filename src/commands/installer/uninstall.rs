// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::flags::Flags;
use crate::cli::{Exit, Step};
use crate::console::{Line, Logger};
use crate::{Error, Result};

use clap::Parser;
use inquire::Confirm;
use licensa_flags::{dry_run, force};
use serde::Serialize;

#[dry_run(false)]
#[force]
#[derive(Parser, Debug, Serialize)]
pub struct UninstallStep {
    /// Specifies the command or subcommand to execute.
    #[command(flatten)]
    flags: Flags,
}

impl Step for UninstallStep {
    fn run(&mut self) -> Result<()> {
        let mut logger = Logger::init();

        let bin = env!("CARGO_CRATE_NAME");
        confirm(&mut logger, bin, self.force)?;

        Ok(())
    }
}

fn confirm(logger: &mut Logger<'_>, cmd: &str, force: bool) -> Result<()> {
    if force {
        logger.write_line(Line::new(format!(
            "Force mode enabled. Skipping confirmation for `{cmd}`."
        )))?;
        return Ok(());
    }

    let message = format!("Uninstall `{cmd}`?");
    let help = format!(
        "All configuration and data for `{cmd}` will be permanently removed. This action cannot be undone."
    );

    let response = Confirm::new(&message)
        .with_default(false)
        .with_help_message(&help)
        .prompt()
        .map_err(Error::Inquire);

    match response {
        Ok(true) => {
            logger.write_line(Line::new("Proceeding with uninstall..."))?;
            // TODO: perform uninstall logic here
            Ok(())
        }
        Ok(false) => {
            logger.write_line(Line::new("Uninstall cancelled."))?;
            Ok(())
        }
        Err(err) => err.exit(),
    }
}
