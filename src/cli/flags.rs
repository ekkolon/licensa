use std::path::PathBuf;

use clap::{ArgAction, Parser};
use serde::Serialize;

#[derive(Parser, Debug, Serialize)]
pub struct Flags {
    /// Specifies the path to a custom configuration file.
    /// If not provided, the tool will use the default configuration or attempt to locate it automatically.
    #[arg(
        short = 'c',
        long = "config",
        value_name = "PATH",
        env = "LICENSA_CONFIG",
        global = true
    )]
    config_path: Option<PathBuf>,

    /// Indicates if the command is being run in a continuous integration (CI) environment.
    /// Enables CI-specific behavior, such as suppressing certain prompts and outputs.
    #[arg(global = true, env = "CI", long, default_value_t = false)]
    ci: bool,

    /// Display output in JSON format.
    /// This is useful for integrating with other tools or for further processing the results programmatically.
    #[arg(global = true, long, default_value_t = false)]
    json: bool,

    /// Disables colored output in the terminal.
    /// This option is typically used in environments where color codes may interfere with output parsing, such as in CI/CD pipelines.
    #[arg(name = "color", global = true, long = "no-color", action = ArgAction::SetFalse)]
    colored: bool,

    /// Disables colored output in the terminal.
    /// This option is typically used in environments where color codes may interfere with output parsing, such as in CI/CD pipelines.
    #[clap(long = "color", overrides_with = "color", hide = true)]
    #[serde(skip)]
    _no_color: bool,

    /// Enables interactive mode, where the tool will prompt the user for input when needed.
    /// If not explicitly provided, defaults to `true`. Can be overridden with `--no-interactive`.
    #[arg(global = true, long = "no-interactive", action = ArgAction::SetFalse)]
    interactive: bool,

    /// Foo all the bars [default]
    #[clap(long = "interactive", overrides_with = "interactive", hide = true)]
    #[serde(skip)]
    _no_interactive: bool,

    /// Suppresses all output except for errors.
    /// This is particularly useful when automating tasks or running scripts where only critical messages are needed.
    #[arg(global = true, short, long, default_value_t = false)]
    quiet: bool,

    /// Automatically answers "yes" to all prompts, effectively disabling any interactive behavior.
    /// Useful for scripting and automated workflows where user input is not possible or desired.
    #[arg(global = true, short, long, default_value_t = false)]
    yes: bool,

    /// Enables verbose output, providing detailed information about the execution.
    /// Useful for debugging or gaining more insights into the tool's actions.
    #[arg(global = true, short, long, default_value_t = false)]
    verbose: bool,
}

impl Flags {
    /// Adjusts flag values based on the `ci` environment.
    /// When `ci` is `true`, it sets specific flags to appropriate values.
    pub fn adjust_for_ci(&mut self) {
        if self.ci {
            self.colored = false;
            self.interactive = false;
            self.yes = true;
        }
    }
}
