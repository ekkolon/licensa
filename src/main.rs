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
mod env;
mod license;
mod logger;
mod spdx;
mod store;
mod utils;
mod validator;

use clap::{error::ErrorKind, CommandFactory, Parser};
use cli::Commands;
use mimalloc::MiMalloc;
use spdx::license::SpdxLicenseStore;
use store::LicenseRef;

use crate::cli::Cli;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Configure logging
  logger::init();

  // let item = spdx::license::find_license_item("mit");
  // println!("LICENSE LIST ITEM: {:#?}", item);

  let http_client = reqwest::Client::new();
  // let license_store = LicenseStore::new(&http_client);
  let license_store = SpdxLicenseStore::new(&http_client);

  let cli = Cli::parse();

  match &cli.command {
    Commands::Init(args) => {
      // Fetch provided license from local or remote store.
      let config = config::Config::parse()?;
      println!("{:#?}", &config);
      //
    }

    Commands::Apply(args) => {
      // Fetch provided license from local or remote store.
      let license_ref = LicenseRef::new(&args.license);
      let license = license_store.fetch_license_details(&args.license).await;
      if let Err(fetch_error) = license {
        Cli::command()
          .error(ErrorKind::Io, fetch_error.to_string())
          .exit()
      } else {
        println!("{:#?} License details: {:#?}", &args.license, &license);
        return Ok(());
      };
    }

    Commands::Verify(args) => {}
  };

  Ok(())
}
