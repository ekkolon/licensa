use clap::Subcommand;
use serde::Serialize;
use uninstall::UninstallStep;
use update::UpdateStep;

pub mod uninstall;
pub mod update;

const HELP_LONG_COMMAND_UPDATE: &str = color_print::cstr!(
    r#"Update Licensa to the latest available version.

The `update` command checks for a newer version of Licensa and installs it if found.
It ensures you have the latest features, performance improvements, and security patches.

This command supports both interactive and non-interactive modes. 
In non-interactive environments (e.g., CI), use the `--force` or `--yes` flag to skip confirmation.

This command fails if:
    - The update server cannot be reached.
    - The binary cannot be replaced due to permission issues.
    - The downloaded version is invalid or corrupted."#
);

const HELP_LONG_COMMAND_UNINSTALL: &str = color_print::cstr!(
    r#"Uninstall Licensa from your system.

The `uninstall` command removes the Licensa binary and any associated configuration data.
You will be asked for confirmation unless `--force` or `--yes` is provided.

Use this command if you want to completely remove Licensa from your environment.

This command fails if:
    - Required permissions are missing to remove the binary or configuration files.
    - The process is aborted by the user.
    - File operations fail due to locks or corrupted paths."#
);

#[derive(Debug, Subcommand, Serialize)]
pub enum Command {
    /// Update Licensa to the latest available version.
    #[command(long_about = HELP_LONG_COMMAND_UPDATE)]
    Update(UpdateStep),

    /// Uninstall Licensa from your system.
    #[command(long_about = HELP_LONG_COMMAND_UNINSTALL)]
    Uninstall(UninstallStep),
}
