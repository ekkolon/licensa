// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use licensa::cli::{self, Cli, Command};

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init(args) => {
            cli::init::run(&args)?;
        }

        Command::Add(args) => {
            cli::add::run(&args)?;
        }

        Command::Check(mut args) => {
            cli::check::run(&mut args)?;
        }
    };

    Ok(())
}
