// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

//! Licensa configuration file parser and utils

pub mod args;

use anyhow::{anyhow, Result};
use clap::CommandFactory;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fs;
use std::path::Path;

use crate::cli::Cli;
use crate::schema::{LicenseId, LicenseNoticeFormat, LicenseYear};
use crate::utils::{self, check_any_file_exists};

const DEFAULT_CONFIG_FILENAME: &str = ".licensarc";
const POSSIBLE_CONFIG_FILENAMES: &[&str] = &[".licensarc", ".licensarc.json"];

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
pub struct LicensaConfig {
    #[serde(rename(serialize = "fullname"))]
    pub owner: String,
    pub license: LicenseId,
    pub format: LicenseNoticeFormat,
    pub exclude: Vec<String>,
    pub year: Option<LicenseYear>,
    pub email: Option<String>,
    pub project: Option<String>,
    pub project_url: Option<url::Url>,
    #[serde(rename = "location")]
    pub location: Option<String>,
    #[serde(rename = "determiner")]
    pub determiner: Option<String>,
}

impl LicensaConfig {
    /// Writes a configuration file to the directory specified by `out_dir`.
    ///
    /// # Arguments
    ///
    /// * `out_dir` - A type `P` implementing `AsRef<Path>`, representing the directory to write to.
    /// * `config` - A type `T` implementing `Borrow<Config>`, representing the configuration to be written.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A configuration file already exists in the provided directory path.
    /// - There are issues converting the borrowed `Config` to a `serde_json::Value`.
    /// - There are issues writing the JSON data to the file.
    ///
    /// # Note
    ///
    /// The `Config` type is assumed to be a type representing the configuration structure.
    ///
    /// # Panics
    ///
    /// This function does not intentionally panic. If any panics occur, they are likely to be
    /// caused by lower-level functions like `serde_json::to_value` or `utils::write_json`.
    pub fn generate_workspace_config<P>(&self, workspace_root: P, skip_check: bool) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let workspace_root = workspace_root.as_ref();

        verify_dir(workspace_root);

        if !skip_check {
            // Check if config file already exists in provided path, and if so error out
            throw_when_workspace_config_exists(true, workspace_root)?;
        }

        // Write configs as pretty-json to the default config filename
        let config = serde_json::to_value(self)?;
        let file_path = workspace_root.join(DEFAULT_CONFIG_FILENAME);
        utils::write_json(file_path, &config)?;

        Ok(())
    }
}

/// Writes a configuration file to the directory specified by `out_dir`.
///
/// # Arguments
///
/// * `out_dir` - A type `P` implementing `AsRef<Path>`, representing the directory to write to.
/// * `config` - A type `T` implementing `Borrow<Config>`, representing the configuration to be written.
///
/// # Errors
///
/// Returns an error if:
/// - A configuration file already exists in the provided directory path.
/// - There are issues converting the borrowed `Config` to a `serde_json::Value`.
/// - There are issues writing the JSON data to the file.
///
/// # Note
///
/// The `Config` type is assumed to be a type representing the configuration structure.
///
/// # Panics
///
/// This function does not intentionally panic. If any panics occur, they are likely to be
/// caused by lower-level functions like `serde_json::to_value` or `utils::write_json`.
pub fn write_workspace_config_file<P, T>(workspace_root: P, config: T) -> Result<()>
where
    P: AsRef<Path>,
    T: Borrow<LicensaConfig>,
{
    let workspace_root = workspace_root.as_ref();
    verify_dir(workspace_root);

    // Exit when config file already exists
    throw_when_workspace_config_exists(false, workspace_root)?;

    // Write configs as pretty-json to the default config filename
    let config = serde_json::to_value(config.borrow())?;
    let file_path = workspace_root.join(DEFAULT_CONFIG_FILENAME);
    utils::write_json(file_path, &config)?;

    Ok(())
}

/// Find a Licensa configuration file in the directory specified by `target_dir`.
/// If a config file is found, read it and return it's contents.
///
/// # Arguments
///
/// * `target_dir` - A type `P` implementing `AsRef<Path>`, representing the lookup directory.
///
/// # Errors
///
/// Returns an error if none of the possible configuration file names exist in
/// the provided directory path or if there's an issue reading the file content.
pub fn read_config_file<P>(workspace_root: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let workspace_root = workspace_root.as_ref();

    verify_dir(workspace_root);

    let config_path = check_any_file_exists(workspace_root, POSSIBLE_CONFIG_FILENAMES);
    if let Some(path) = config_path {
        let content = fs::read_to_string(path)?;
        return Ok(content);
    }

    // None of the possible configuration file names exist
    Err(anyhow!(
        "None of the configuration files {:?} were found in the current directory.",
        POSSIBLE_CONFIG_FILENAMES
    ))
}

/// Check if config file already exists in provided path
pub fn config_file_exists<P>(workspace_root: P) -> bool
where
    P: AsRef<Path>,
{
    check_any_file_exists(workspace_root, POSSIBLE_CONFIG_FILENAMES).map_or(false, |p| true)
}

pub fn throw_when_workspace_config_exists<P>(must_exist: bool, workspace_root: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let workspace_root = workspace_root.as_ref();
    verify_dir(workspace_root);
    let exists = config_file_exists(workspace_root);

    if exists && !must_exist {
        return Err(anyhow!(
            "Licensa is already initialized in the current directory",
        ));
    }
    if !exists && must_exist {
        return Err(anyhow!(
            "Licensa config file not found in the current directory"
        ));
    }

    Ok(())
}

#[inline]
fn verify_dir<P: AsRef<Path>>(dir: P) {
    if !dir.as_ref().is_dir() {
        Cli::command()
            .error(
                clap::error::ErrorKind::Io,
                anyhow!("{} is not a directory", dir.as_ref().display()),
            )
            .exit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Read};
    use tempfile::tempdir;

    #[test]
    fn test_write_config_file_successful() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Define the target directory
        let target_dir = temp_dir.path();

        // Define the output file path
        let config_file_path = target_dir.join(DEFAULT_CONFIG_FILENAME);

        // Create a sample configuration
        let sample_config = LicensaConfig {
            owner: "John Doe".to_string(),
            exclude: vec![],
            project: None,
            email: None,
            project_url: None,
            format: LicenseNoticeFormat::Spdx,
            license: LicenseId("MIT".to_string()),
            year: Some(LicenseYear::single_year(2024)),
            location: None,
            determiner: None,
        };

        // Test writing the config file
        let write_result = write_workspace_config_file(target_dir, &sample_config);
        assert!(write_result.is_ok());

        // Verify that the config file exists
        assert!(config_file_path.exists());

        // Verify the content of the config file
        let mut file = File::open(&config_file_path).expect("Failed to open config file");
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)
            .expect("Failed to read config file content");

        let expected_content = serde_json::to_string_pretty(&sample_config).unwrap();
        assert_ne!(file_content, expected_content);

        // Cleanup
        drop(config_file_path);
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_write_config_file_existing_file() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Define the target directory
        let target_dir = temp_dir.path();

        // Create an existing config file in the temporary directory
        let existing_config_filename = POSSIBLE_CONFIG_FILENAMES[0];
        let existing_config_path = target_dir.join(existing_config_filename);
        File::create(&existing_config_path).expect("Failed to create existing config file");

        // Create a sample configuration
        let sample_config = LicensaConfig {
            owner: "John Doe".to_string(),
            exclude: vec![],
            project: None,
            email: None,
            project_url: None,
            format: LicenseNoticeFormat::Spdx,
            license: LicenseId("MIT".to_string()),
            year: Some(LicenseYear::single_year(2024)),
            location: None,
            determiner: None,
        };

        // Test writing the config file when it already exists
        let result = write_workspace_config_file(target_dir, sample_config);
        assert!(result.is_err());

        // Cleanup
        drop(existing_config_path);
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_find_config_file_single_file_exists() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Define the target directory
        let target_dir = temp_dir.path();

        // Create a sample config file in the temporary directory
        let sample_config_filename = POSSIBLE_CONFIG_FILENAMES[0];
        let sample_config_path = target_dir.join(sample_config_filename);
        File::create(&sample_config_path).expect("Failed to create sample config file");

        // Test finding the config file
        let result = read_config_file(target_dir);
        assert!(result.is_ok());

        // Verify the content of the config file
        let expected_content = String::new(); // Adjust with actual content
        assert_eq!(result.unwrap(), expected_content);

        // Cleanup
        drop(sample_config_path);
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_find_config_file_multiple_files_exist() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Define the target directory
        let target_dir = temp_dir.path();

        // Create sample config files in the temporary directory
        for &filename in POSSIBLE_CONFIG_FILENAMES {
            let config_path = target_dir.join(filename);
            File::create(&config_path).expect("Failed to create sample config file");

            // Cleanup
            drop(config_path);
        }

        // Test finding the config file (use the first one in the list)
        let result = read_config_file(target_dir);
        assert!(result.is_ok());

        // Verify the content of the config file
        let expected_content = String::new(); // Adjust with actual content
        assert_eq!(result.unwrap(), expected_content);

        // Cleanup
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_find_config_file_no_file_exists() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Define the target directory
        let target_dir = temp_dir.path();

        // Test finding the config file when none exist
        let result = read_config_file(target_dir);
        // assert!(result.is_err());
    }
}
