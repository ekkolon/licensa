// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code, unused_variables)]

mod cache;
mod cli;
mod config;
mod copyright_notice;
mod env;
mod header;
mod helpers;
mod interpolation;
mod license;
mod logger;
mod spdx;
mod store;
#[cfg(test)]
mod test_utils;
mod utils;
mod validator;
mod workspace;

use clap::Parser;
use cli::Commands;
use mimalloc::MiMalloc;

use crate::cli::Cli;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init(args) => {
            cli::init::build(args)?;
        }

        Commands::Apply(args) => {
            cli::apply::build(args)?;
        }

        Commands::Verify(args) => {
            cli::verify::build(args)?;
        }

        Commands::List(args) => {
            cli::list::build(args);
        }
        Commands::Add(args) => {
            println!("{args:?}");
        }
    };

    Ok(())
}
