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
    // Configure logging
    logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init(args) => {
            println!("{:?}", &args);
            workspace::_examples::example_scan_parallel()?;
        }

        Commands::Apply(args) => {
            println!("{:?}", &args);
        }

        Commands::Verify(args) => {
            cli::verify::build(args)?;
        }

        Commands::List(args) => {
            cli::list::build(args);
        }
    };

    Ok(())
}
