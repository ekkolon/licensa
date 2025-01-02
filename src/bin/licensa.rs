// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use licensa::cli::add::AddStep;
use licensa::cli::check::CheckStep;
use licensa::cli::init::InitStep;
use licensa::cli::{installer, Cli, Command, Step};
use licensa::Result;

use clap::{CommandFactory, Parser};

fn main() -> Result<()> {
    let cmd = Cli::command();
    let version = cmd.get_version().unwrap_or_default();
    let name = cmd.get_name();
    println!("{name} v{version}");

    let mut cli = Cli::parse();

    //println!("{}", serde_json::to_string_pretty(&cli)?);
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
