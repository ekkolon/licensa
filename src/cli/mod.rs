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

mod apply;
mod verify;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use self::{apply::ApplyArgs, verify::VerifyArgs};

/// Licensia is an experimental CLI tool to enforce, verify, apply and modify
/// file license headers for a variaty of programming languages.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(next_line_help = true)]
pub struct Cli {
  /// Optional name to operate on
  // pub name: Option<String>,

  /// Use options from a custom config file
  #[arg(short, long, value_name = "FILE")]
  pub config: Option<PathBuf>,

  #[arg(short, long, default_value_t = false)]
  pub verbose: bool,

  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Verify presence of license headers in one or more files.
  ///
  /// A glob pattern may be used to verify multiple files that recursively.
  Verify(VerifyArgs),

  /// Apply license header to one or more files.
  ///
  /// A glob pattern can be used to run this command on matches recursively.
  Apply(ApplyArgs),
}
