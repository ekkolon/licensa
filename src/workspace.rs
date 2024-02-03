// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Licensa configuration file parser and utils

use crate::schema::{LicenseHeaderFormat, LicenseId, LicenseYear};

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct LicensaWorkspace {
    pub owner: String,
    pub license: LicenseId,
    pub format: LicenseHeaderFormat,
    pub exclude: Vec<String>,
    pub year: Option<LicenseYear>,
    pub email: Option<String>,
    pub project: Option<String>,
    #[serde(rename(serialize = "projecturl"))]
    pub project_url: Option<url::Url>,
    pub location: Option<String>,
    pub determiner: Option<String>,
}
