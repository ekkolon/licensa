// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

pub mod add;
pub mod apply;
pub mod init;
pub mod list;
pub mod verify;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use apply::ApplyArgs;
use init::InitArgs;
use list::ListArgs;
use verify::VerifyArgs;

use add::AddArgs;

/// Licensa is a powerful CLI tool designed for seamless source code license management.
///
/// Developers can effortlessly verify, apply, modify, and enforce SPDX license headers
/// across their source code.
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
    pub subcommands: SubCommands,
}

#[derive(Subcommand, Debug)]
pub enum SubCommands {
    /// Initialize Licensa configuration for the current workspace.
    ///
    /// The `init` command simplifies the configuration process by creating a `.licensarc` file
    /// in the current working directory. This file contains workspace-wide Licensa configurations,
    /// eliminating the need to pass them as arguments for subsequent subcommands like `apply`.
    ///
    /// When you run the `init` command, the following steps are performed:
    ///
    /// 1. Creation of a `.licensarc` file based on the provided command arguments.
    ///
    /// 2. Generation of a `.licensaignore` file containing glob patterns.
    ///
    /// # Errors
    ///
    /// The `init` workflow fails in the following scenarios:
    ///
    /// - The current working directory already contains a `.licensarc` file.
    ///
    /// - Invalid arguments are provided.
    #[command(name = "init")]
    Init(InitArgs),

    /// Verify presence of license headers in one or more files.
    ///
    /// A glob pattern may be used to verify multiple files that recursively.
    #[command(name = "verify")]
    Verify(VerifyArgs),

    /// Apply copyright license headers to source code files.
    ///
    /// The `apply` command recursively scans specified directory patterns and seamlessly adds
    /// license headers to source files that don't already contain them. Existing headers are
    /// left untouched, ensuring that files are modified in place without overwriting existing
    /// licensing information.
    ///
    /// You can customize which files and directories are considered for license header application
    /// by using patterns in the `.gitignore` or `.licensaignore` file. Patterns provided in the
    /// `.licensaignore` file take precedence over those in the `.gitignore` file.
    ///
    /// If a `.licensarc` config file exists in the current working directory, its configuration
    /// fields are merged with the supplied command arguments. However, command arguments take
    /// precedence over config field values.
    ///
    /// # Errors
    ///
    /// The `apply` command may fail in the following scenarios:
    ///
    /// - Invalid `.licensarc` config file format (if present)
    ///
    /// - Invalid argument value
    ///
    /// - Missing required argument
    ///
    /// - Insufficient read/write permissions for source files
    #[command(name = "apply")]
    Apply(ApplyArgs),

    /// Get a list of available licenses
    #[command(name = "list")]
    List(ListArgs),

    /// Add license header to one or more files
    #[command(name = "add")]
    Add(AddArgs),
}
