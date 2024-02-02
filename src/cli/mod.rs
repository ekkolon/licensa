// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

mod commands;
pub use commands::Command;

use clap::Parser;

/// Licensa is a powerful CLI tool designed for seamless source code license management.
///
/// Developers can effortlessly verify, apply, modify, and enforce SPDX license headers
/// across their source code.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(next_line_help = true)]
pub struct Cli {
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}
