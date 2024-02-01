// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use crate::config::args::Config;
use crate::config::{throw_if_config_file_exists, LicensaConfig};
use crate::error::exit_io_error;
use crate::license::LicensesManifest;
use crate::schema::{LicenseId, LicenseNoticeFormat};

use anyhow::Result;
use clap::Args;
use inquire::{Select, Text};

use std::env::current_dir;

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
    let workspace_root = current_dir()?;

    if let Err(err) = throw_if_config_file_exists(false, &workspace_root) {
        // Current directory is already a licensa workspace
        exit_io_error(err);
    }

    let mut config = args.config.clone().with_workspace_config(&workspace_root)?;
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
        if format == LicenseNoticeFormat::Compact {
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
    let workspace_config: LicensaConfig = serde_json::from_value(workspace_config)?;
    workspace_config.generate_config_file(workspace_root, true)?;


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

fn prompt_copyright_notice_format() -> Result<LicenseNoticeFormat> {
    let options = ["Compact", "Full", "Spdx"];
    let format = Select::new(
        "The format of the copyright notice to render",
        options.to_vec(),
    )
    // .with_starting_cursor(2)
    .prompt()?;

    Ok(LicenseNoticeFormat::from(format))
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
