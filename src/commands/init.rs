// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::config::Config;
use crate::error::exit_io_error;
use crate::license::LicensesManifest;
use crate::ops::workspace::{save_workspace_config, throw_workspace_config_exists};
use crate::schema::{LicenseHeaderFormat, LicenseId};
use crate::workspace::LicensaWorkspace;

use anyhow::Result;
use clap::Args;
use inquire::{Select, Text};

use std::env::current_dir;
use std::fs;

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    #[command(flatten)]
    config: Config,
}

impl InitArgs {
    pub fn to_config(&self) -> Result<Config> {
        Ok(Config::default())
    }
}

pub fn run(args: &InitArgs) -> Result<()> {
    let workspace_root = current_dir()?.join("tmp");
    fs::create_dir_all(&workspace_root)?;

    if let Err(err) = throw_workspace_config_exists(&workspace_root) {
        exit_io_error(err);
    }

    let mut config = args.config.clone().with_workspace_config(&workspace_root)?;

    println!("{:?}", &config);

    if config.license.is_none() {
        let license_id = prompt_license_selection()?;
        let _ = config.license.insert(license_id);
    }
    if config.owner.is_none() {
        let owner = prompt_copyright_owner()?;
        let _ = config.owner.insert(owner);
    }
    if config.format.is_none() {
        let format = prompt_copyright_notice_format()?;
        if format == LicenseHeaderFormat::Compact {
            if config.compact_format_args.location.is_none() {
                let location = prompt_license_location()?;
                let _ = config.compact_format_args.location.insert(location);
            }

            if config.compact_format_args.determiner.is_none() {
                let determiner = prompt_license_location_determiner()?;
                let _ = config.compact_format_args.determiner.insert(determiner);
            }
        }

        let _ = config.format.insert(format);
    }

    // FIXME: Invalid range error when using format YYYY-present
    // TODO: check year

    let workspace_config = serde_json::to_value(config)?;
    let workspace_config: LicensaWorkspace = serde_json::from_value(workspace_config)?;
    save_workspace_config(workspace_root, workspace_config)?;

    Ok(())
}

fn prompt_license_selection() -> Result<LicenseId> {
    let license_ids = LicensesManifest::spdx_ids();
    let license_id: String = Select::new("Choose a License", license_ids).prompt()?;
    let license_id = LicenseId::from(license_id);
    Ok(license_id)
}

fn prompt_copyright_owner() -> Result<String> {
    let owner = Text::new("Copyright owner").prompt()?;
    Ok(owner)
}

fn prompt_copyright_notice_format() -> Result<LicenseHeaderFormat> {
    let options = ["Compact", "Full", "Spdx"];
    let format = Select::new(
        "The format of the copyright notice to render",
        options.to_vec(),
    )
    // .with_starting_cursor(2)
    .prompt()?;

    Ok(LicenseHeaderFormat::from(format))
}

fn prompt_license_location() -> Result<String> {
    let location = Text::new("Where can users find your license")
        .with_placeholder("e.g. \"the root of this project\"")
        .prompt()?;
    Ok(location)
}

fn prompt_license_location_determiner() -> Result<String> {
    let delimiter = Text::new("License location text delimiter")
        .with_placeholder("e.g. \"in\" or \"at\"")
        .prompt()?;

    Ok(delimiter)
}
