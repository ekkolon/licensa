// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::config::{
    Config, {LICENSA_CONFIG_FILENAME, LICENSA_IGNORE_FILENAME},
};
use crate::schema::LicenseId;
use crate::workspace::ops::{ensure_config_missing, save_config, save_ignore_file};

use anyhow::Result;
use clap::Args;
use inquire::{Select, Text};
use lazy_static::lazy_static;

use std::env::current_dir;
use std::str::FromStr;

lazy_static! {
    static ref LICENSA_IGNORE: &'static str = std::include_str!("../../.licensaignore");
}

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    #[command(flatten)]
    config: Config,
}

impl InitArgs {
    pub fn into_config(&self) -> Result<Config> {
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
    let workspace_root = current_dir()?;
    ensure_config_missing(&workspace_root, LICENSA_CONFIG_FILENAME)?;
    let config = args.into_config()?;
    save_config(&workspace_root, LICENSA_CONFIG_FILENAME, config)?;
    save_ignore_file(
        workspace_root,
        LICENSA_IGNORE_FILENAME,
        LICENSA_IGNORE.as_bytes(),
    )?;

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
