// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::Cli;
use crate::license::LicenseId;
use crate::terminal::{self, Step};
use crate::workspace::ops::{ensure_config_missing, save_config, save_ignore_file};
use crate::workspace::{
    Config, IntoWorkspaceConfig, LICENSA_CONFIG_FILENAME, LICENSA_IGNORE, LICENSA_IGNORE_FILENAME,
};
use crate::Result;

use clap::error::ErrorKind;
use clap::{Args, CommandFactory};
use inquire::{Select, Text};
use std::fmt::Debug;
use std::{env::current_dir, str::FromStr};

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    #[command(flatten)]
    config: Config,
}

impl IntoWorkspaceConfig for InitArgs {
    fn into_workspace_config(self) -> Result<Config> {
        let mut config = Config::default();
        config.update(self.config.clone());

        if config.license.is_none() {
            let license_id = prompt_license_selection()?;
            let _ = config.license.insert(license_id);
        }
        if config.owner.is_none() {
            let owner = prompt_copyright_owner()?;
            let _ = config.owner.insert(owner);
        }

        Ok(config)
    }
}

pub fn run(args: &InitArgs) -> Result<()> {
    let mut task = terminal::Task::new("Initialize Licensa workspace");
    task.start()?;

    let workspace_root = current_dir()?;
    if let Err(err) = ensure_config_missing(&workspace_root, LICENSA_CONFIG_FILENAME) {
        task.finish_err()?;
        Cli::command().error(ErrorKind::Io, err).exit();
    }

    let config = args.clone().into_workspace_config()?;
    if let Err(err) = save_config(&workspace_root, LICENSA_CONFIG_FILENAME, config) {
        task.finish_err()?;
        Cli::command().error(ErrorKind::Io, err).exit();
    };

    if let Err(err) = save_ignore_file(
        workspace_root,
        LICENSA_IGNORE_FILENAME,
        LICENSA_IGNORE.as_bytes(),
    ) {
        task.finish_err()?;
        Cli::command().error(ErrorKind::Io, err).exit();
    };

    task.finish_ok()?;
    task.logln("Successfully initialized Licensa workspace");

    task.line_break();
    task.logln("Use `licensa add` to apply license headers to files.");

    Ok(())
}

fn prompt_license_selection() -> Result<LicenseId> {
    let license_ids = crate::license::list_spdx_license_names();
    let license_id: String = Select::new("Choose a License", license_ids).prompt()?;
    let license_id = crate::license::id_from_license_fullname(&license_id)?;
    let license_id = LicenseId::from_str(&license_id)?;
    Ok(license_id)
}

fn prompt_copyright_owner() -> Result<String> {
    let owner = Text::new("Copyright owner").prompt()?;
    Ok(owner)
}
