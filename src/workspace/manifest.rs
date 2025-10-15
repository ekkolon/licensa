// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::Result;
use crate::{
    license::{LicenseId, LicensePeriod},
    workspace::ops::find_workspace_config,
};

use clap::Args;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const POSSIBLE_CONFIG_FILENAMES: &[&str] = &[
    ".licensarc",
    ".licensarc.json",
    "licensa.yaml",
    "licensa.yml",
];

/// The filename for Licensa's ignore file, which contains glob patterns
/// used to exclude specific files or directories from licensing operations.
pub const LICENSA_IGNORE_FILENAME: &str = ".licensaignore";

lazy_static! {
    pub static ref LICENSA_IGNORE: &'static str = std::include_str!("../../.licensaignore");
}

/// The filename for Licensa's configuration file, which holds workspace-specific settings.
pub const LICENSA_CONFIG_FILENAME: &str = ".licensarc";

pub trait IntoWorkspaceConfig {
    fn into_workspace_config(self) -> Result<LicensaManifest>;
}

const HELP_FLAG_EXCLUDE: &str = color_print::cstr!(
    r#"Glob patterns to exclude specific files or directories from the licensing process.

Use this option to prevent license headers or other actions from applying to files that match the given patterns.

This is useful for excluding:

  - Generated files or third-party code that should remain unchanged.
  - Files that already include the correct license information.
  - Files irrelevant to the licensing process (e.g., temporary or build files).

<bold>USAGE</bold>
  Glob patterns use standard .gitignore syntax, are case-sensitive, and apply to files within the workspace or project directory.
  Patterns can be combined, and multiple patterns should be separated by spaces.

<bold>Examples:</bold>
  Exclude a single file:
    $ licensa add --exclude path/to/file.rs

  Exclude a directory and specific files:
    $ licensa add --exclude **/dist/**
    $ licensa add --exclude **/dist/** **/vendor/*.js
"#
);

/// The configuration for a Licensa workspace, loaded from a config file.
///
/// A Licensa config file contains workspace-wide settings that, when present, are merged
/// with the command arguments during execution. CLI flags always take precedence, unless
/// a `--config` flag is explicitly used to specify a custom config file.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Args)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields, default)]
pub struct LicensaManifest {
    /// The SPDX license ID or expression.
    ///
    /// Automatically resolves common abbreviations (e.g., "apache" -> "Apache-2.0").
    /// For more information, see https://spdx.org/licenses/.
    #[arg(
        value_name = "LICENSE",
        short = 't',
        long = "type",
        verbatim_doc_comment
    )]
    pub license: Option<LicenseId>,

    /// The copyright holder.
    #[arg(value_name = "NAME", short, long)]
    pub owner: Option<String>,

    /// The copyright year or range.
    ///
    /// This is used to specify the copyright duration when applying license headers.
    /// A range is inclusive (e.g., "2020-2023"). The special keyword `present` represents
    /// the current year (e.g., "2022-present").
    #[arg(value_name = "YEAR | PERIOD",  alias = "year", long)]
    #[serde(alias = "year")]
    pub period: Option<LicensePeriod>,

    /// Exclude specific files or directories from the licensing process.
    //#[serde(default = "Vec::new")]
    #[arg(
        value_name = "PATTERN[,...]", 
        long_help = HELP_FLAG_EXCLUDE, 
        long, 
        default_values_t = Vec::<String>::new(), 
        value_delimiter = ' ', 
        num_args = 1..
    )]
    pub exclude: Vec<String>,
}

impl LicensaManifest {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_defaults() -> Self {
        let empty = LicensaManifest::new();
        LicensaManifest {
            license: empty.license().map(|s| s.into()),
            owner: empty.holder().map(|s| s.to_owned()),
            period: empty.year().map(|s| s.to_owned()),
            exclude: empty.exclude().to_vec(),
        }
    }

    pub fn update(&mut self, source: LicensaManifest) {
        if !source.exclude.is_empty() {
            let mut patterns = source.exclude;
            self.exclude.append(&mut patterns);
        }
        if let Some(holder) = source.owner.as_deref() {
            self.owner = Some(holder.to_owned())
        }
        if let Some(license) = source.license.as_deref() {
            self.license = Some(LicenseId(license.to_string()))
        }
        if let Some(year) = source.period.as_ref() {
            self.period = Some(year.to_owned())
        }
    }

    pub fn exclude(&self) -> &[String] {
        self.exclude.as_ref()
    }

    pub fn holder(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    pub fn license(&self) -> Option<&str> {
        self.license.as_deref()
    }

    pub fn year(&self) -> Option<&LicensePeriod> {
        self.period.as_ref()
    }

    /// Try to resolve workspace configuration and merge with current config.
    pub fn merge_into_existing_at_path<T>(&mut self, workspace_root: T) -> Result<LicensaManifest>
    where
        T: AsRef<Path>,
    {
        let ws = find_workspace_config(workspace_root.as_ref());
        if let Ok(ws) = ws {
            let mut ws_config = serde_json::from_str::<LicensaManifest>(&ws)?;
            ws_config.update(self.to_owned());
            return Ok(ws_config);
        }

        Ok(self.to_owned())
    }
}

pub struct Copyright {
    pub license: LicenseId,
    pub owner: String,
    pub year: Option<LicensePeriod>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Args)]
pub struct CopyrightArgs {
    /// The SPDX license identifier.
    ///
    /// For more information, visit https://spdx.org/licenses/.
    #[arg(short = 't', long = "type")]
    pub license: Option<LicenseId>,

    /// The copyright owner.
    #[arg(short, long, value_name = "NAME")]
    pub owner: Option<String>,

    /// The copyright year or range.
    ///
    /// Special keyword `present` indicates the current year (e.g., "2022-present").
    #[arg(long, value_name = "YYYY | YYYY-YYYY | YYYY-present")]
    pub year: Option<LicensePeriod>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_config_invalid_license_year() {
        let config = serde_json::from_value::<LicensaManifest>(json!({
            "year": 20033,
        }));
        assert!(config.is_err());

        let config = serde_json::from_value::<LicensaManifest>(json!({
            "year": null,
        }));
        assert!(config.is_ok());

        let config = serde_json::from_value::<LicensaManifest>(json!({
            "year": "2025-2024",
        }));
        assert!(config.is_err());
    }
}
