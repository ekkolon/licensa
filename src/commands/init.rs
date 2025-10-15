// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::{Step, UnwrapOrExit};
use crate::console::Logger;
use crate::license::LicenseId;
use crate::workspace::ops::{ensure_config_missing, save_config, save_ignore_file};
use crate::workspace::{
    IntoWorkspaceConfig, LicensaManifest, LICENSA_CONFIG_FILENAME, LICENSA_IGNORE,
    LICENSA_IGNORE_FILENAME,
};
use crate::Result;
use clap::Parser;
use inquire::{Select, Text};
use serde::Serialize;
use std::fmt::Debug;
use std::{env::current_dir, str::FromStr};

#[derive(Parser, Debug, Clone, Serialize)]
pub struct InitStep {
    #[command(flatten)]
    config: LicensaManifest,
}

impl IntoWorkspaceConfig for InitStep {
    fn into_workspace_config(self) -> Result<LicensaManifest> {
        let mut config = LicensaManifest::default();
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

impl Step for InitStep {
    fn run(&mut self) -> Result<()> {
        let mut logger = Logger::init();

        let src_root = current_dir()?;
        ensure_config_missing(&src_root, LICENSA_CONFIG_FILENAME).unwrap_or_exit();

        let config = self.clone().into_workspace_config()?;
        save_config(&src_root, LICENSA_CONFIG_FILENAME, config).unwrap_or_exit();

        save_ignore_file(src_root, LICENSA_IGNORE_FILENAME, LICENSA_IGNORE.as_bytes())
            .unwrap_or_exit();

        logger.display("Workspace initialized successfully.")?;
        logger.display("Created configuration and ignore files.")?;
        logger.line_break()?;
        logger.display("Next step: run `licensa add` to apply license headers.")?;

        Ok(())
    }
}

fn prompt_license_selection() -> Result<LicenseId> {
    let license_ids = crate::license::list_spdx_license_names();
    let choice: String = Select::new("Select a license:", license_ids).prompt()?;
    let id = crate::license::id_from_license_fullname(&choice)?;
    Ok(LicenseId::from_str(&id)?)
}

fn prompt_copyright_owner() -> Result<String> {
    let owner = Text::new("Enter copyright owner:").prompt()?;
    Ok(owner)
}
