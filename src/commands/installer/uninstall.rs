// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::flags::Flags;
use crate::cli::{Exit, Step};
use crate::console::Logger;
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
        logger.display("Force flag detected. Skipping confirmation.")?;
        return Ok(());
    }

    let message = format!("Are you sure you want to uninstall {cmd}?");
    let help_message = format!(
        "This will remove all data associated with {cmd} from your system and cannot be undone."
    );

    let action = Confirm::new(&message)
        .with_default(false)
        .with_help_message(&help_message)
        .prompt()
        .map_err(Error::Inquire);

    match action {
        Ok(true) => {
            logger.display("User confirmed.")?; // (todo): remove when implementation done
            return Ok(());
        }
        Ok(false) => {
            logger.display("Operation aborted by user.")
            // (todo): Exit here
        }
        Err(err) => err.exit(),
    }?;

    Ok(())
}
