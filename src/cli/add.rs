// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use std::env::current_dir;

use crate::config::{resolve_workspace_config, Config};
use crate::error;
use crate::schema::{LicenseId, LicenseNoticeFormat, LicenseYear};

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};

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
            error::missing_required_arg_error("-t, --type <LICENSE>")
        }
        if config.owner.is_none() {
            error::missing_required_arg_error("-o, --owner <OWNER>")
        }
        if config.format.is_none() {
            error::missing_required_arg_error("-f, --format <FORMAT>")
        }

        let args = AddArgs {
            format: config.format,
            license: config.license,
            owner: config.owner,
            year: config.year,
            determiner: self.determiner.clone(),
            location: self.location.clone(),
        };

        let args = serde_json::to_value(args);
        if let Err(err) = args.as_ref() {
            error::serialize_args_error("add", err)
        }

        let config = serde_json::from_value::<AddCommandConfig>(args.unwrap());
        if let Err(err) = config.as_ref() {
            error::deserialize_args_error("add", err)
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
