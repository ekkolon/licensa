// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

pub mod apply;
mod init;
pub mod list;
pub mod verify;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use apply::ApplyArgs;
use init::InitArgs;
use list::ListArgs;
use verify::VerifyArgs;

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
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Verify presence of license headers in one or more files.
    ///
    /// A glob pattern may be used to verify multiple files that recursively.
    #[command(name = "init")]
    Init(InitArgs),

    /// Verify presence of license headers in one or more files.
    ///
    /// A glob pattern may be used to verify multiple files that recursively.
    #[command(name = "verify")]
    Verify(VerifyArgs),

    /// Apply license header to one or more files.
    ///
    /// A glob pattern can be used to run this command on matches recursively.
    #[command(name = "apply")]
    Apply(ApplyArgs),

    /// Get a list of available licenses
    #[command(name = "list")]
    List(ListArgs),
}
