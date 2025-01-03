use clap::Subcommand;
use serde::Serialize;
use uninstall::UninstallStep;
use update::UpdateStep;

pub mod uninstall;
pub mod update;

const HELP_LONG_COMMAND_UPDATE: &str = color_print::cstr!(
    r#"Verify the presence of license headers in one or more files.
The `check` command inspects files to ensure they contain the required license headers.

It supports glob patterns, making it easy to validate multiple files recursively.
Use this command to confirm compliance with licensing requirements across your codebase.

This command fails if:
    - Invalid glob patterns are provided.
    - Specified files or directories are unreadable."#
);

const HELP_LONG_COMMAND_UNINSTALL: &str = color_print::cstr!(
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

#[derive(Debug, Subcommand, Serialize)]
pub enum Command {
    /// Initialize the Licensa configuration for the current workspace.
    #[command(long_about = HELP_LONG_COMMAND_UPDATE)]
    Update(UpdateStep),

    /// Apply license headers to source code files.
    #[command(long_about = HELP_LONG_COMMAND_UNINSTALL)]
    Uninstall(UninstallStep),
}
