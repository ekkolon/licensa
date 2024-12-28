// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use licensa::cli::{Cli, Command};
use licensa::commands;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init(args) => {
            commands::init::run(&args)?;
        }

        Command::Add(args) => {
            commands::apply::run(&args)?;
        }

        Command::Check(mut args) => {
            commands::check::run(&mut args)?;
        }
    };

    Ok(())
}
