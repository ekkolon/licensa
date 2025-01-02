// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Licensa configuration file parser and utils

mod error;
mod manifest;

pub mod ops;
pub mod utils;

use std::{fs, path::PathBuf};

pub use error::*;
pub use manifest::*;
use utils::{resolve_any_path, verify_dir};

use crate::license::{LicenseId, LicensePeriod};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct LicenseConfig {
    pub owner: String,
    pub license: LicenseId,
    pub exclude: Vec<String>,
    pub period: Option<LicensePeriod>,
}

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
pub struct Workspace {
    src_root: PathBuf,
}

impl Workspace {
    /// Find a Licensa configuration file in the directory specified by `workspace_root`.
    /// If a config file is found, read it and return it's contents.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - The lookup directory.
    ///
    /// # Errors
    ///
    /// Returns an error if none of the possible configuration file names exist in
    /// the provided directory path or if there's an issue reading the file content.
    pub fn try_find_config<P>(&self) -> Result<Option<LicenseConfig>> {
        verify_dir(&self.src_root)?;
        let config_path = resolve_any_path(&self.src_root, POSSIBLE_CONFIG_FILENAMES);
        if let Some(path) = config_path {
            let content = fs::read_to_string(path)?;
            let content_json: LicenseConfig = serde_json::from_str(&content)?;
            return Ok(Some(content_json));
        }
        Ok(None)
    }
}
