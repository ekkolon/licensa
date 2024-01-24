// Copyright 2024 Nelson Dominguez
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code, unused_variables)]

mod cli;
mod config;
mod copyright_notice;
mod env;
mod interpolation;
mod license;
mod logger;
mod spdx;
mod store;
mod utils;
mod validator;

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
    }

    Commands::Apply(args) => {
      println!("{:?}", &args);
    }

    Commands::Verify(args) => {
      println!("{:?}", &args);
    }
  };

  Ok(())
}
