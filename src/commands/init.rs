// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::config::Config;
use crate::error::exit_io_error;
use crate::ops::workspace::{
    save_workspace_config, save_workspace_ignore, throw_workspace_config_exists,
};
use crate::schema::LicenseId;

use anyhow::Result;
use clap::Args;
use inquire::{Select, Text};

use std::env::current_dir;
use std::str::FromStr;

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    #[command(flatten)]
    config: Config,
}

impl InitArgs {
    pub fn into_config(&self) -> Result<Config> {
        let mut config = Config::default();
        config.update(self.config.clone());
        Ok(config)
    }
}

pub fn run(args: &InitArgs) -> Result<()> {
    let workspace_root = current_dir()?;

    if let Err(err) = throw_workspace_config_exists(&workspace_root) {
        exit_io_error(err);
    }

    let mut config = args.into_config()?;

    if config.license.is_none() {
        let license_id = prompt_license_selection()?;
        let _ = config.license.insert(license_id);
    }
    if config.owner.is_none() {
        let owner = prompt_copyright_owner()?;
        let _ = config.owner.insert(owner);
    }

    save_workspace_config(&workspace_root, config)?;
    save_workspace_ignore(workspace_root)?;

    println!("Successfully initialized Licensa workspace");

    Ok(())
}

fn prompt_license_selection() -> Result<LicenseId> {
    let license_ids = crate::spdx::list_spdx_license_names();
    let license_id: String = Select::new("Choose a License", license_ids).prompt()?;
    let license_id = crate::spdx::id_from_license_fullname(&license_id)?;
    let license_id = LicenseId::from_str(&license_id)?;
    Ok(license_id)
}

fn prompt_copyright_owner() -> Result<String> {
    let owner = Text::new("Copyright owner").prompt()?;
    Ok(owner)
}
