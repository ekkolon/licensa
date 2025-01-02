// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::flags::Flags;
use crate::cli::{Exit, Step, UnwrapOrExit};
use crate::console::{Line, Logger};
use crate::workspace::LicensaManifest;
use crate::{Error, Result};

use clap::Parser;
use inquire::Confirm;
use licensa_flags::{dry_run, force};
use serde::Serialize;

#[dry_run(false)]
#[force]
#[derive(Parser, Debug, Serialize)]
pub struct UninstallStep {
    #[command(flatten)]
    config: LicensaManifest,

    /// Specifies the command or subcommand to execute.
    #[command(flatten)]
    flags: Flags,
}

impl Step for UninstallStep {
    fn run(&mut self) -> Result<()> {
        let mut logger = Logger::init();

        let bin_name = env!("CARGO_CRATE_NAME");
        logger.write_line(Line::new("{cmd} CLI").bind("cmd", bin_name))?;
        confirm_uninstall(&mut logger, self.dry_run)?;
        Ok(())
    }
}

fn confirm_uninstall(logger: &mut Logger<'_>, should_prompt: bool) -> Result<()> {
    if !should_prompt {
        return Ok(());
    }
    let action = Confirm::new("Are you sure you want to continue?")
        .with_default(false)
        .with_help_message("This data is stored for good reasons")
        .prompt()
        .map_err(Error::Inquire);

    match action {
        Ok(true) => logger.display("User confirmed."),
        Ok(false) => logger.display("User did not confirm."),
        Err(err) => err.exit(),
    }
    .unwrap_or_exit();

    Ok(())
}
