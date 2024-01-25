// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code, unused_variables)]

mod cli;
mod config;
mod copyright_notice;
mod env;
mod interpolation;
mod license;
mod logger;
mod scanner;
mod source;
mod spdx;
mod store;
mod utils;
mod validator;

use clap::Parser;
use cli::Commands;
use license::LicensesManifest;
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
      scanner::_examples::example_scan_op()?;
    }

    Commands::Apply(args) => {
      println!("{:?}", &args);
    }

    Commands::Verify(args) => {
      println!("{:?}", &args);
    }

    Commands::List => {
      LicensesManifest::print_license_table();
    }
  };

  Ok(())
}
