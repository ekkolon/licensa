// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use licensa::cli::{Cli, Command};
use licensa::commands;

use anyhow::Result;
use clap::Parser;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> Result<()> {
    run()
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Command::Init(args) => {
            commands::init::run(args)?;
        }

        Command::Apply(args) => {
            commands::apply::run(args)?;
        }

        Command::Verify(args) => {
            commands::verify::run(args)?;
        }

        Command::List(args) => {
            commands::list::run(args);
        }
    };

    Ok(())
}