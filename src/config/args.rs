// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use super::read_config_file;
use crate::schema::{LicenseId, LicenseNoticeFormat, LicenseYear};

use anyhow::{anyhow, Result};
use clap::{Args, Parser};
use serde::{Deserialize, Serialize};

use std::path::Path;

/// Represents the container for a Licensa config file that may be
/// included in root directory of a software project.
///
/// A Licensa config file contains workspace-wide config presets.
/// If a config file is present in the same directory a Licensa command
/// is executed in, the provided config fields will be merged into
/// the command arguments, replacing the specific command's default
/// argument settings.
///
/// CLI arguments **always** take precedence over options provided
/// in the config file. An exeception to that rule is when a command
/// accepts a `--config` flag, which, when present, explicitly requests
/// the usage of a specific Licensa config file.
///
/// It is assumed the file is in valid JSON format and is named after one
/// of the following filenames:
///
///   - `.licensarc`
///   - `.licensarc.json`
#[derive(Debug, Serialize, Deserialize, Default, Parser, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// The SPDX identifier of the license (case-insensitive).
    ///
    /// SPDX (Software Package Data Exchange) is a standard format for communicating the components,
    /// licenses, and copyrights associated with software.
    ///
    /// See https://spdx.org/licenses/
    #[arg(short = 't', long = "type")]
    pub license: Option<LicenseId>,

    /// The copyright owner.
    #[arg(short, long, value_name = "NAME")]
    pub owner: Option<String>,

    /// The E-Mail of the copyright owner.
    #[arg(long)]
    pub email: Option<String>,

    /// The name of the project to be licensed.
    ///
    /// Note that most licenses don't require this field, however,
    /// there are a few that do:
    ///
    /// - **BSD-4-Clause**
    /// - **MulanPSL-2.0**
    /// - **NCSA**
    /// - **Vim**
    ///
    /// An interpolation error will occur if this field is missing in
    /// an attempt to apply a copyright notice to a license requiring
    /// this field.
    #[arg(long)]
    pub project: Option<String>,

    /// URL of the project.
    ///
    /// Note that most licenses don't require this field, however,
    /// there are a few that do:
    ///
    /// - **BSD-4-Clause**
    /// - **MulanPSL-2.0**
    /// - **NCSA**
    /// - **Vim**
    ///
    /// An interpolation error will occur if this field is missing in
    /// an attempt to apply a copyright notice to a license requiring
    /// this field.
    #[arg(long, value_name = "URL")]
    pub project_url: Option<url::Url>,

    /// Represents the copyright year or a range of years.
    ///
    /// The `year` field accepts various formats, allowing flexibility in specifying the copyright period:
    ///
    /// - A single year, e.g., `2022`.
    ///
    /// - A range of years, e.g., `2022-2024`.
    ///
    /// - The special keyword `present`, indicating the current year, e.g., `2022-present`.
    ///
    /// This field is used to define the copyright duration when applying license headers.
    /// When providing a range, it signifies the inclusive span of years.
    #[arg(long, value_name = "YYYY | YYYY-YYYY | YYYY-present")]
    pub year: Option<LicenseYear>,

    /// The copyright header format to apply on each file to be licensed.
    #[arg(
        short,
        long,
        value_enum,
        rename_all = "lower",
        requires_if("compact", "compact_format")
    )]
    pub format: Option<LicenseNoticeFormat>,

    #[command(flatten)]
    #[serde(flatten)]
    pub compact_format_args: CompactLicenseNoticeArgs,

    /// A list of glob patterns to exclude from the licensing process.
    ///
    /// Defining patterns here is synonymous to adding them either to
    /// the `.gitignore` or `.licensaignore` file.
    #[arg(long)]
    pub exclude: Option<Vec<String>>,
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_defaults() -> Self {
        let empty = Config::new();
        Config {
            email: empty.email().map(|s| s.to_owned()),
            exclude: Some(empty.exclude().to_vec()),
            format: Some(empty.format().to_owned()),
            owner: empty.holder().map(|s| s.to_owned()),
            license: empty.license().map(|s| s.into()),
            project: empty.project().map(|s| s.to_owned()),
            project_url: empty.project_url().map(|s| s.to_owned()),
            year: empty.year().map(|s| s.to_owned()),
            compact_format_args: CompactLicenseNoticeArgs {
                location: empty.location().map(|s| s.to_owned()),
                determiner: empty.determiner().map(|s| s.to_owned()),
            },
        }
    }

    pub fn update(&mut self, source: Config) {
        if let Some(email) = source.email.as_deref() {
            self.email = Some(email.to_owned())
        }
        if let Some(exclude) = source.exclude.as_deref() {
            self.exclude = Some(exclude.to_owned())
        }
        if let Some(format) = source.format.as_ref() {
            self.format = Some(format.to_owned())
        }
        if let Some(holder) = source.owner.as_deref() {
            self.owner = Some(holder.to_owned())
        }
        if let Some(license) = source.license.as_deref() {
            self.license = Some(LicenseId(license.to_string()))
        }
        if let Some(project) = source.project.as_deref() {
            self.project = Some(project.to_owned())
        }
        if let Some(project_url) = source.project_url.as_ref() {
            self.project_url = Some(project_url.to_owned())
        }
        if let Some(year) = source.year.as_ref() {
            self.year = Some(year.to_owned())
        }
        if let Some(location) = source.compact_format_args.location.as_ref() {
            self.compact_format_args.location = Some(location.to_owned())
        }
        if let Some(determiner) = source.compact_format_args.determiner.as_ref() {
            self.compact_format_args.determiner = Some(determiner.to_owned())
        }
    }

    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    pub fn exclude(&self) -> &[String] {
        self.exclude.as_ref().map(|v| v.as_ref()).unwrap_or(&[])
    }

    pub fn format(&self) -> &LicenseNoticeFormat {
        self.format.as_ref().unwrap_or(&LicenseNoticeFormat::Spdx)
    }

    pub fn holder(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    pub fn license(&self) -> Option<&str> {
        self.license.as_deref()
    }

    pub fn project(&self) -> Option<&str> {
        self.project.as_deref()
    }

    pub fn project_url(&self) -> Option<&url::Url> {
        self.project_url.as_ref()
    }

    pub fn year(&self) -> Option<&LicenseYear> {
        self.year.as_ref()
    }

    pub fn location(&self) -> Option<&str> {
        self.compact_format_args.location.as_deref()
    }

    pub fn determiner(&self) -> Option<&str> {
        self.compact_format_args.determiner.as_deref()
    }

    /// Try to resolve workspace configuration and merge those with defaults.
    pub fn with_workspace_config<T>(&mut self, workspace_root: T) -> Result<Config>
    where
        T: AsRef<Path>,
    {
        let mut merge_config = Config::from_defaults();
        merge_config.update(self.clone());
        Self::from_workspace_config(workspace_root, Some(merge_config))
    }

    /// Try to resolve workspace configuration and merge those with defaults.
    pub fn from_workspace_config<T>(workspace_root: T, initial: Option<Config>) -> Result<Config>
    where
        T: AsRef<Path>,
    {
        let mut config = initial.unwrap_or(Config::from_defaults());

        // Read config file, if it exists and update config
        let config_file = read_config_file(workspace_root.as_ref());
        if let Ok(parsed_config) = config_file {
            let parsed_config = serde_json::from_str::<Config>(&parsed_config);
            if let Err(err) = parsed_config {
                // Config file found but failed parsing.
                return Err(anyhow!("Failed to parse Licensa config file.\n {}", err));
            }
            config.update(parsed_config.unwrap());
        }

        Ok(config)
    }
}

#[derive(Debug, Args, Serialize, Deserialize, Clone, Default)]
#[group(id = "compact_format", required = false, multiple = true)]
pub struct CompactLicenseNoticeArgs {
    /// Specifies the location where the full LICENSE text can be accessed.
    ///
    /// This argument is utilized in tandem with `--determiner` and is only applicable
    /// when applying license headers in `compact` format.
    ///
    /// It accepts either a brief text description, such as "the root of this project,"
    /// or a URL, for example, "https://www.apache.org/licenses/LICENSE-2.0."
    ///
    /// For instance, when the full license text is stored on your project's homepage,
    /// set the `--location` and `--determiner` arguments as follows:
    ///
    /// `licensa [COMMAND] --format compact --determiner "at" --location "https://myproject.com/license"`.
    ///
    /// Similarly, if the license text resides in your project repository:
    ///
    /// `licensa [COMMAND] --format compact --determiner in --location "the root of this project"`.
    #[arg(long = "location", value_name = "TEXT | URL")]
    #[serde(rename = "location")]
    pub location: Option<String>,

    /// Specifies the preposition that precedes the path to the license in a sentence (e.g., "in").
    ///
    /// This argument is only applicable when using the 'compact' format.
    ///
    /// It allows you to customize the language used in the license header placement.
    /// For example, if you want the header to read "License information is located in the root of this project,"
    /// you would use `--determiner "in"`.
    #[arg(long = "determiner", value_name = "PREPOSIION")]
    #[serde(rename = "determiner")]
    pub determiner: Option<String>,
}
