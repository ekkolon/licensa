// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;

pub mod add;
pub mod check;
pub mod init;
pub mod installer;

use add::AddStep;
use check::CheckStep;
use init::InitStep;
use serde::Serialize;

const HELP_LONG_COMMAND_INIT: &str = color_print::cstr!(
    r#"Initialize the Licensa configuration for the current workspace.
    
The `init` command simplifies the configuration process by generating a `.licensarc`
file in the current directory, which contains workspace-wide Licensa settings. This
eliminates the need to repeatedly specify settings for subsequent commands like `add`.
    
Actions performed by `init`:
    
    1. Creates a `.licensarc` file with the provided arguments.
    2. Generates a `.licensaignore` file containing glob patterns for excluded files or directories.
    
This command fails if:
    
    - A `.licensarc` file already exists in the current directory.
    - Invalid arguments are provided."#
);

const HELP_LONG_COMMAND_CHECK: &str = color_print::cstr!(
    r#"Verify the presence of license headers in one or more files.
The `check` command inspects files to ensure they contain the required license headers.

It supports glob patterns, making it easy to validate multiple files recursively.
Use this command to confirm compliance with licensing requirements across your codebase.

This command fails if:
    - Invalid glob patterns are provided.
    - Specified files or directories are unreadable."#
);

const HELP_LONG_COMMAND_ADD: &str = color_print::cstr!(
    r#"Apply license headers to source code files.
The `add` command scans specified directories and adds license headers to source files
that are missing them. Existing headers are preserved, and no previously applied licenses
will be overwritten.

You can customize file inclusion and exclusion using `.gitignore` or `.licensaignore` patterns,
with `.licensaignore` taking precedence. If a `.licensarc` config file exists, its settings
will be merged with command-line arguments, with the latter taking precedence.

This command fails if:
    - The `.licensarc` configuration file has an invalid format (if used).
    - Invalid argument values are supplied.
    - Required arguments are missing.
    - Insufficient read/write permissions for the specified files or directories."#
);

#[derive(Debug, Parser, Serialize)]
pub enum Command {
    /// Initialize the Licensa configuration for the current workspace.
    #[command(long_about = HELP_LONG_COMMAND_INIT)]
    Init(InitStep),

    /// Apply license headers to source code files.
    #[command(long_about = HELP_LONG_COMMAND_ADD, visible_alias = "apply")]
    Add(AddStep),

    /// Verify the presence of license headers in one or more files.
    #[command(long_about = HELP_LONG_COMMAND_CHECK, visible_alias = "verify")]
    Check(CheckStep),

    /// Verify the presence of license headers in one or more files.
    #[command(subcommand, name = "self", long_about = HELP_LONG_COMMAND_CHECK)]
    Installer(installer::Command),
}
