// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use std::env::current_dir;

use crate::config::{resolve_workspace_config, Config};
use crate::schema::{LicenseId, LicenseNoticeFormat, LicenseYear};
// use crate::validator;
use anyhow::Result;
use clap::{CommandFactory, Parser};
use serde::{Deserialize, Serialize};

use super::Cli;

pub fn run(args: &AddArgs) -> Result<()> {
    let config = args.to_config()?;
    println!("{config:#?}");
    Ok(())
}

#[derive(Parser, Debug, Serialize, Clone)]
pub struct AddArgs {
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

    /// The word that appears before the path to the license in a sentence (e.g. "in").
    ///
    /// Only takes effect in conjunction with 'compact' format.
    #[arg(long, required = false, group = "compact_info")]
    pub determiner: Option<String>,

    /// The location where the LICENSE file can be found.
    ///
    /// Only takes effect in conjunction with 'compact' format.
    #[arg(long, required = false, group = "compact_info")]
    pub location: Option<String>,
}

impl AddArgs {
    // Merge self with config::Config
    fn to_config(&self) -> Result<AddCommandConfig> {
        let workspace_root = current_dir()?;
        let mut config = resolve_workspace_config(workspace_root)?;

        config.update(Config {
            license: self.license.clone(),
            owner: self.owner.clone(),
            format: self.format.clone(),
            year: self.year.clone(),
            ..Default::default()
        });

        if config.license.is_none() {
            missing_required_arg_error("-t, --type <LICENSE>")
        }
        if config.owner.is_none() {
            missing_required_arg_error("-o, --owner <OWNER>")
        }
        if config.format.is_none() {
            missing_required_arg_error("-f, --format <FORMAT>")
        }
        if config.format.is_none() {
            missing_required_arg_error("-y, --year <YEAR>")
        }

        let opts = serde_json::to_value(AddArgs {
            format: config.format,
            license: config.license,
            owner: config.owner,
            year: config.year,
            determiner: self.determiner.clone(),
            location: self.location.clone(),
        });

        if let Err(err) = opts.as_ref() {
            let err_msg = format!("Failed to serialize `add` command arguemnts.\n {}", err);
            Cli::command()
                .error(clap::error::ErrorKind::ValueValidation, err_msg)
                .exit();
        }

        let config = serde_json::from_value::<AddCommandConfig>(opts.unwrap());

        if let Err(err) = config.as_ref() {
            let err_msg = format!("Failed to deserialize `add` command arguemnts.\n {}", err);
            Cli::command()
                .error(clap::error::ErrorKind::ValueValidation, err_msg)
                .exit();
        }

        Ok(config.unwrap())
    }
}

#[derive(Debug, Deserialize)]
pub struct AddCommandConfig {
    pub license: LicenseId,
    pub owner: String,
    pub year: LicenseYear,
    pub format: LicenseNoticeFormat,
    pub determiner: Option<String>,
    pub location: Option<String>,
}

fn missing_required_arg_error<T>(arg: T)
where
    T: AsRef<str>,
{
    Cli::command()
        .error(
            clap::error::ErrorKind::MissingRequiredArgument,
            format!("Missing required argument {}", arg.as_ref()),
        )
        .exit()
}
