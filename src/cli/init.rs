// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use std::env::current_dir;

use crate::config::resolve_workspace_config;
use crate::config::Config;
use crate::license::LicensesManifest;
use crate::schema::LicenseId;
use crate::schema::LicenseNoticeFormat;
use crate::schema::LicenseYear;

use anyhow::Result;
use clap::Args;
use inquire::{Select, Text};
use serde::Serialize;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// License SPDX ID.
    #[arg(short = 't', long = "type")]
    pub license: Option<LicenseId>,

    /// The copyright owner.
    #[arg(short, long)]
    pub owner: Option<String>,

    /// The copyright year.
    #[arg(short, long)]
    pub year: Option<LicenseYear>,

    /// The copyright header format to apply on each file to be licensed.
    #[arg(
        short,
        long,
        value_enum,
        rename_all = "lower",
        requires_if("compact", "compact_info")
    )]
    pub format: Option<LicenseNoticeFormat>,

    #[command(flatten)]
    compact_template_args: CompactLicenseNoticeArgs,
}

impl InitArgs {
    pub fn to_config(&self) -> Result<Config> {
        let workspace_root = current_dir()?;
        let mut config = resolve_workspace_config(workspace_root)?;

        config.update(Config {
            license: self.license.clone(),
            owner: self.owner.clone(),
            format: self.format.clone(),
            year: self.year.clone(),
            license_location_determiner: self
                .compact_template_args
                .license_location_determiner
                .clone(),
            license_location: self.compact_template_args.license_location.clone(),
            ..Default::default()
        });

        if config.license.is_none() {
            // Prompt user with license selection
        }

        Ok(config)
    }
}

#[derive(Debug, Args, Serialize, Clone)]
#[group(id = "compact_info", required = false, multiple = true)]
pub struct CompactLicenseNoticeArgs {
    /// The location where the LICENSE file can be found.
    ///
    /// Only takes effect in conjunction with 'compact' format.
    #[arg(long = "location")]
    #[serde(rename = "location")]
    pub license_location: Option<String>,

    /// The word that appears before the path to the license in a sentence (e.g. "in").
    ///
    /// Only takes effect in conjunction with 'compact' format.
    #[arg(long = "determiner")]
    #[serde(rename = "determiner")]
    pub license_location_determiner: Option<String>,
}

pub fn run(args: &InitArgs) -> Result<()> {
    let mut config = Config::from_defaults();

    if args.license.is_none() {
        let license_id = prompt_license_selection()?;
        let _ = config.license.insert(license_id);
    }
    if args.owner.is_none() {
        let owner = prompt_copyright_owner()?;
        let _ = config.owner.insert(owner);
    }
    if args.format.is_none() {
        let format = prompt_copyright_notice_format()?;
        if format == LicenseNoticeFormat::Compact {
            if args
                .compact_template_args
                .license_location_determiner
                .is_none()
            {
                let determiner = prompt_license_location_determiner()?;
                let _ = config.license_location_determiner.insert(determiner);
            }

            if args.compact_template_args.license_location.is_none() {
                let location = prompt_license_location()?;
                let _ = config.license_location.insert(location);
            }
        }

        let _ = config.format.insert(format);
    }
    // TODO: check year

    // TODO: Parse config to LicensaConfig

    // TODO: Write LicensaConfig json

    println!("{config:?}");

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
    let options = vec!["Compact", "Full", "Spdx"];
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
