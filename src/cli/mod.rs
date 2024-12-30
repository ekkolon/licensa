// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

mod command;

pub use command::*;

use clap::Parser;

/// Licensa is a powerful CLI tool designed for seamless source code license management.
///
/// Developers can effortlessly verify, apply, modify, and enforce SPDX license headers
/// across their source code.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(
    next_line_help = true,
    //flatten_help = true,
    //override_usage = "licensa add -X [-a] [-b] <file>\n       \
    //     myapp -Y [-c] <file1> <file2>\n       \
    //     myapp -Z [-d|-e]"
)]
pub struct Cli {
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}
