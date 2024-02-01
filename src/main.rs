// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code, unused_variables)]

mod cache;
mod cli;
mod config;
mod copyright_notice;
mod env;
mod error;
mod header;
mod interpolation;
mod license;
mod logger;
mod schema;
mod spdx;
mod store;
#[cfg(test)]
mod test_utils;
mod utils;
mod validator;
mod workspace;

use clap::Parser;
use cli::SubCommands;
use mimalloc::MiMalloc;

use crate::cli::Cli;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.subcommands {
        SubCommands::Init(args) => {
            cli::init::run(args)?;
        }

        SubCommands::Apply(args) => {
            cli::apply::run(args)?;
        }

        SubCommands::Verify(args) => {
            cli::verify::run(args)?;
        }

        SubCommands::List(args) => {
            cli::list::run(args);
        }
        SubCommands::Add(args) => {
            cli::add::run(args)?;
        }
    };

    Ok(())
}
