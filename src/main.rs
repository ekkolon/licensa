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
mod env;
mod spdx;
mod template;
mod utils;
mod validator;

use clap::Parser;
use spdx::{SpdxTemplateRef, TemplateRef, TemplateStore};

use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let http_client = reqwest::Client::new();
  let template_store = TemplateStore::new(&http_client);
  let cli = Cli::parse();

  // println!("{:?}", cli);

  let template_ref = SpdxTemplateRef {
    spdx_id: "Apache-2.0".into(),
  };

  template_store.fetch(template_ref).await?;

  // spdx::confirm_remote_fetch(test_spdx_id)?;
  // spdx::fetch_remote_template(test_spdx_id).await?;
  // println!("CWD {}", cwd.display());y

  // let patterns = glob("./**/*.rs").expect("Failed to read glob pattern");

  // for entry in patterns {
  //   match entry {
  //     Ok(path) => println!("{:?}", path.display()),
  //     Err(e) => println!("{:?}", e),
  //   }
  // }

  Ok(())
}
