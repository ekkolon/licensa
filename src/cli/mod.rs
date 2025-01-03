// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

mod traits;

use serde::Serialize;
pub use traits::*;
pub mod flags;

use clap::Parser;

use crate::commands::Command;

/// Licensa is a powerful CLI tool designed for seamless source code license management.
///
/// Developers can effortlessly verify, apply, modify, and enforce SPDX license headers
/// across their source code.
#[derive(Parser, Debug, Serialize)]
#[command(
    author,
    version,
    about,
    long_about,
    propagate_version = true,
    next_line_help = true
)]
pub struct Cli {
    /// Specifies the command or subcommand to execute.
    #[command(subcommand)]
    pub step: Command,
}
