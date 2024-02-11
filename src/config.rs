// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::ops::workspace::find_workspace_config;
use crate::schema::{LicenseId, LicenseYear};

use anyhow::{anyhow, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The filename used for Licensa's ignore file, which contains patterns
/// for files or directories to be excluded from license scanning or other
/// workspace operations.
pub const LICENSA_IGNORE_FILENAME: &str = ".licensaignore";

/// The filename used for Licensa's configuration file, which stores
/// workspace-specific settings and preferences.F
pub const LICENSA_CONFIG_FILENAME: &str = ".licensarc";

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
#[derive(Debug, Clone, Default, Serialize, Deserialize, Args)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// The SPDX license ID or expression (case-insensitive).
    ///
    /// If a value partially matches a SPDX license ID, it is automatically transformed
    /// into the most likely SPDX license expression match. For example, "apache" would
    /// become "Apache-2.0", "mits" is transformed into "MIT" and so on.
    /// However, an error is thrown if no match is found for the imprecise expression.
    ///
    /// For a comprehensive list of the available SPDX refer to https://spdx.org/licenses/.
    #[arg(short = 't', long = "type", verbatim_doc_comment)]
    #[arg(value_name = "ID")]
    #[arg(value_parser = crate::parser::parse_license_id)]
    pub license: Option<LicenseId>,

    /// The copyright owner.
    #[arg(short, long, verbatim_doc_comment, value_name = "NAME")]
    pub owner: Option<String>,

    /// Represents the copyright year or a range of years.
    ///
    /// This field is used to define the copyright duration when applying license headers.
    /// When providing a range, it signifies the inclusive span of years.
    ///
    /// The special keyword `present` indicates the current year, e.g. "2022-present".
    ///
    /// === EXAMPLE USAGE ================================================
    ///     
    ///     licensa <COMMAND> --year 2020
    ///     licensa <COMMAND> --year 2020-2023
    ///     licensa <COMMAND> --year 2020-present
    #[cfg(not(doctest))]
    #[arg(long, verbatim_doc_comment)]
    #[arg(value_name = "YEAR | PERIOD")]
    #[arg(value_parser = crate::parser::parse_license_year)]
    pub year: Option<LicenseYear>,

    /// A list of glob patterns to exclude specific files or directories from the licensing process.
    ///
    /// Using this field, you can prevent the application of license headers or other licensing-related
    /// actions to files that match the specified patterns. This is useful for excluding things like:
    ///
    /// - Generated files or third-party code that shouldn't be modified.
    /// - Files already containing appropriate license information.
    /// - Files irrelevant to the licensing process.
    ///
    ///
    /// === EXAMPLE USAGE ================================================
    ///
    ///     licensa apply --exclude *.txt **/target
    ///
    /// === IMPORTANT NOTES ==============================================
    ///
    /// - Glob patterns follow standard `.gitignore` patterns.
    /// - Patterns are case-sensitive.
    /// - Exclusion applies to files within the workspace or project directory.
    /// - If a file matches multiple patterns, it's still excluded.
    /// - Provide multiple patterns as separate space-delimited arguments when using command-line options.
    #[cfg(not(doctest))]
    #[arg(long, verbatim_doc_comment)]
    #[arg(value_name = "GLOB[,...]", value_delimiter = ' ', num_args = 1..)]
    pub exclude: Option<Vec<String>>,
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_defaults() -> Self {
        let empty = Config::new();
        Config {
            license: empty.license().map(|s| s.into()),
            owner: empty.holder().map(|s| s.to_owned()),
            year: empty.year().map(|s| s.to_owned()),
            exclude: Some(empty.exclude().to_vec()),
        }
    }

    pub fn update(&mut self, source: Config) {
        if let Some(exclude) = source.exclude.as_deref() {
            self.exclude = Some(exclude.to_owned())
        }
        if let Some(holder) = source.owner.as_deref() {
            self.owner = Some(holder.to_owned())
        }
        if let Some(license) = source.license.as_deref() {
            self.license = Some(LicenseId(license.to_string()))
        }
        if let Some(year) = source.year.as_ref() {
            self.year = Some(year.to_owned())
        }
    }

    pub fn exclude(&self) -> &[String] {
        self.exclude.as_ref().map(|v| v.as_ref()).unwrap_or(&[])
    }

    pub fn holder(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    pub fn license(&self) -> Option<&str> {
        self.license.as_deref()
    }

    pub fn year(&self) -> Option<&LicenseYear> {
        self.year.as_ref()
    }

    /// Try to resolve workspace configuration and merge those with self.
    pub fn with_workspace_config<T>(&mut self, workspace_root: T) -> Result<Config>
    where
        T: AsRef<Path>,
    {
        let ws = find_workspace_config(workspace_root.as_ref());
        if let Ok(ws) = ws {
            let parsed = serde_json::from_str::<Config>(&ws);
            if let Err(err) = parsed {
                // Config file found but failed parsing.
                return Err(anyhow!("Failed to parse Licensa config file.\n {}", err));
            }

            let mut ws_config = parsed.unwrap();
            ws_config.update(self.to_owned());
            return Ok(ws_config);
        }

        Ok(self.to_owned())
    }
}

pub struct Copyright {
    pub license: LicenseId,
    pub owner: String,
    pub year: Option<LicenseYear>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Args)]
pub struct CopyrightArgs {
    /// The SPDX identifier of the license (case-insensitive).
    ///
    /// SPDX (Software Package Data Exchange) is a standard format for communicating the components,
    /// licenses, and copyrights associated with software.
    ///
    /// See https://spdx.org/licenses/
    #[arg(short = 't', long = "type", value_parser = crate::parser::parse_license_id)]
    pub license: Option<LicenseId>,

    /// The copyright owner.
    #[arg(short, long, value_name = "NAME")]
    pub owner: Option<String>,

    /// Represents the copyright year or a range of years.
    ///
    /// This field is used to define the copyright duration when applying license headers.
    /// When providing a range, it signifies the inclusive span of years.
    ///
    /// The special keyword `present` indicates the current year, e.g. `2022-present`.
    #[arg(long, value_name = "YYYY | YYYY-YYYY | YYYY-present", value_parser = crate::parser::parse_license_year)]
    pub year: Option<LicenseYear>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_config_invalid_license_year() {
        let config = serde_json::from_value::<Config>(json!({
            "year": 20033,
        }));
        assert!(config.is_err());

        let config = serde_json::from_value::<Config>(json!({
            "year": null,
        }));
        assert!(config.is_ok());

        let config = serde_json::from_value::<Config>(json!({
            "year": "2025-2024",
        }));
        assert!(config.is_err());
    }
}
