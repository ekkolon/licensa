// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::flags::Flags;
use crate::cli::Step;
use crate::console::Logger;
use crate::workspace::LicensaManifest;
use crate::Result;

use clap::Parser;
use licensa_flags::dry_run;
use serde::Serialize;

#[dry_run(false)]
#[derive(Parser, Debug, Serialize)]
pub struct UpdateStep {
    #[command(flatten)]
    config: LicensaManifest,

    /// Specifies the command or subcommand to execute.
    #[command(flatten)]
    flags: Flags,
}

impl Step for UpdateStep {
    fn run(&mut self) -> Result<()> {
        let logger = Logger::init();
        Ok(())
    }
}
