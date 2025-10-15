// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use licensa::cli::{Cli, Step};
use licensa::commands::add::AddStep;
use licensa::commands::check::CheckStep;
use licensa::commands::init::InitStep;
use licensa::commands::{installer, Command};
use licensa::Result;

use clap::Parser;

fn main() -> Result<()> {
    let mut cli = Cli::parse();

    // println!("{}", serde_json::to_string_pretty(&cli)?);
    match cli.step {
        Command::Init(ref mut args) => InitStep::run(args)?,
        Command::Add(ref mut args) => AddStep::run(args)?,
        Command::Check(ref mut args) => CheckStep::run(args)?,
        Command::Installer(ref mut args) => match args {
            installer::Command::Update(ref mut step) => installer::update::UpdateStep::run(step)?,
            installer::Command::Uninstall(ref mut step) => {
                installer::uninstall::UninstallStep::run(step)?
            }
        },
    };

    Ok(())
}
