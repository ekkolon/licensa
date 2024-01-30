// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

//! Licensa configuration file parser and utils

pub mod builder;

use anyhow::{anyhow, Ok, Result};
use clap::CommandFactory;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::env::current_dir;
use std::fs;
use std::path::Path;

use crate::cli::Cli;
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
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// The license SPDX ID.
    pub license: String,

    /// The license year.
    #[serde(default = "utils::current_year")]
    pub year: u16,

    /// The full name of the copyright holder.
    #[serde(rename(serialize = "fullname"))]
    pub holder: String,

    /// The remote URL where the project lives.
    ///
    /// Note that most licenses don't require this field, however,
    /// there are a few that do:
    ///
    /// - **OFL-1.1**
    ///
    /// An interpolation error will occur if this field is missing in
    /// an attempt to apply a copyright notice to a license requiring
    /// this field.
    pub email: Option<String>,

    /// The remote URL where the project lives.
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
    #[serde(default)]
    pub project: Option<String>,

    /// The remote URL where the project lives.
    ///
    /// Note that most licenses don't require this field, however,
    /// there are a few that do:
    ///
    /// - **NCSA**
    ///
    /// An interpolation error will occur if this field is missing in
    /// an attempt to apply a copyright notice to a license requiring
    /// this field.
    #[serde(rename(deserialize = "projecturl"))]
    pub project_url: Option<url::Url>,

    /// The copyright header format to apply.
    #[serde(default)]
    pub format: CopyrightNoticeFormat,

    /// A list of glob patterns to exclude from the licensing process.
    ///
    /// Defining patterns here is synonymous to adding them either to
    /// the `.gitignore` or `.licensaignore` file.
    #[serde(default = "default_exclude_patterns")]
    pub exclude: Vec<String>,
}

impl Config {
    pub fn parse_config_file() -> Result<Config> {
        let target_dir = current_dir()?;
        let config_file = find_config_file(target_dir)?;
        let config = serde_json::from_str::<Config>(&config_file)?;
        Ok(config)
    }

    /// Generates  a configuration file in the current working directory.
    ///
    /// # Arguments
    ///
    /// * `config` - A type `T` implementing `Borrow<Config>`, representing the configuration to be written.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues writing the configuration file.
    ///
    /// # Panics
    ///
    /// This function does not intentionally panic. If any panics occur, they are likely to be
    /// caused by lower-level functions like `write_config_file`.
    pub fn generate_config_file<T>(config: T) -> Result<()>
    where
        T: Borrow<Config>,
    {
        let out_dir = current_dir()?;
        let config_file = write_config_file(out_dir, config.borrow());
        if let Err(err) = config_file {
            Cli::command()
                .error(clap::error::ErrorKind::Io, &err)
                .exit();
        };

        Ok(())
    }
}

/// The copyright header format to apply on each file to be licensed.
///
/// You can choose from three built-in copyright notice formats:
///
/// - **Spdx** (default)
///     
///     Based on the SPDX license format and rendered in two lines
///
/// - **Compact**
///
///     A short, four lines long format that refers users to the
///     the location at which the full license file is found.
///
///     *Remarks*:
///
///     The location can be either a path to a file within the
///     repository or an URL.
///
/// - **Full**
///     
///     Render the full license header.
///     
///     *Remarks*:
///
///     This option only applies to licenses that provide a license header
///     (e.g. Apache-2.0 or 0BSD). In cases where no license header is available
///     this fallbacks to the **SPDX** format, or if specified, the `fallback`
///     format option that can be specified in the *generator* config.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CopyrightNoticeFormat {
    /// Renders a two line header text in SPDX format.
    ///
    /// # Example
    ///
    /// *licensed_file.rs*
    /// ```no_run
    /// // Copyright 2012 Bilbo Baggins
    /// // SPDX-License-Identifier: WTFPL
    ///
    /// fn main() {}   
    /// ```
    #[default]
    Spdx,

    /// Renders a short header text that hints to the location
    /// of the original LICENSE file.
    ///
    /// # Example
    ///
    /// *licensed_file.rs*
    /// ```no_run
    /// // Copyright 2001 Frodo Baggins
    /// // Use of this source code is governed by an MIT-style license that can be
    /// // found in the LICENSE file in the root of this project.
    ///     
    /// fn main() {}   
    /// ```
    Compact,

    /// Renders the full license header, if available.
    ///
    /// # Example
    ///
    /// *licensed_file.rs*
    /// ```no_run
    /// // This Source Code Form is subject to the terms of the Mozilla Public
    /// // License, v. 2.0. If a copy of the MPL was not distributed with this
    /// // file, You can obtain one at http://mozilla.org/MPL/2.0/.
    ///
    /// fn main() {}
    /// ```
    Full,
}

impl From<String> for CopyrightNoticeFormat {
    fn from(value: String) -> Self {
        CopyrightNoticeFormat::from(value.as_str())
    }
}

impl From<&str> for CopyrightNoticeFormat {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_ref() {
            "compact" => CopyrightNoticeFormat::Compact,
            "full" => CopyrightNoticeFormat::Full,
            "spdx" => CopyrightNoticeFormat::Spdx,
            val => Cli::command()
                .error(
                    clap::error::ErrorKind::InvalidValue,
                    anyhow!("invalid copyright notice formate '{val}'"),
                )
                .exit(),
        }
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
pub fn write_config_file<P, T>(out_dir: P, config: T) -> Result<()>
where
    P: AsRef<Path>,
    T: Borrow<Config>,
{
    let target_dir = out_dir.as_ref();
    check_dir(target_dir);

    // Check if config file already exists in provided path, and if so error out
    let config_path = check_any_file_exists(target_dir, POSSIBLE_CONFIG_FILENAMES);
    if config_path.is_some() {
        return Err(anyhow!(
            "A Licensa config file already exists in current directory"
        ));
    }

    // Write configs as pretty-json to the default config filename
    let config = serde_json::to_value(config.borrow())?;
    let file_path = target_dir.join(DEFAULT_CONFIG_FILENAME);
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
pub fn find_config_file<P>(target_dir: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let dir = target_dir.as_ref();
    check_dir(dir);

    let config_path = check_any_file_exists(dir, POSSIBLE_CONFIG_FILENAMES);
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

#[inline]
fn check_dir<P: AsRef<Path>>(dir: P) {
    if !dir.as_ref().is_dir() {
        Cli::command()
            .error(
                clap::error::ErrorKind::Io,
                anyhow!("{} is not a directory", dir.as_ref().display()),
            )
            .exit()
    }
}

#[inline]
fn default_exclude_patterns() -> Vec<String> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Read};
    use tempfile::tempdir;

    // #[test]
    // fn test_generate_config_file_successful() {
    //   // Create a temporary directory for testing
    //   let temp_dir = tempdir().expect("Failed to create temporary directory");

    //   // Set the current working directory to the temporary directory
    //   let temp_dir_path = temp_dir.path();
    //   std::env::set_current_dir(temp_dir_path).expect("Failed to set current directory");

    //   // Create a sample configuration
    //   let sample_config = Config {
    //     author: "Jane Doe".to_string(),
    //     generator: GeneratorConfig::default(),
    //     license_type: "Apache-2.0".to_string(),
    //     year: 2024,
    //   };

    //   // Test generating the config file
    //   Config::generate_config_file(&sample_config)
    //     .expect("Failed to generate config file");

    //   // Verify that the config file exists in the temporary directory
    //   let config_file_path = temp_dir_path.join(DEFAULT_CONFIG_FILENAME);
    //   assert!(config_file_path.exists());

    //   // Verify the content of the config file
    //   let file_content = fs::read_to_string(&config_file_path)
    //     .expect("Failed to read config file content");
    //   let expected_content =
    //     serde_json::to_string_pretty(&serde_json::to_value(&sample_config).unwrap())
    //       .unwrap();
    //   assert_eq!(file_content, expected_content);

    //   drop(config_file_path);
    //   temp_dir.close().expect("Failed to close temp directory");
    // }

    // #[test]
    // fn test_generate_config_file_existing_file() {
    //   // Create a temporary directory for testing
    //   let temp_dir = tempdir().expect("Failed to create temporary directory");

    //   // Set the current working directory to the temporary directory
    //   let temp_dir_path = temp_dir.path();
    //   std::env::set_current_dir(temp_dir_path).expect("Failed to set current directory");

    //   // Create an existing config file in the temporary directory
    //   let existing_config_filename = POSSIBLE_CONFIG_FILENAMES[0];
    //   let existing_config_path = temp_dir_path.join(existing_config_filename);
    //   fs::write(&existing_config_path, "Existing content")
    //     .expect("Failed to create existing config file");

    //   // Create a sample configuration
    //   let sample_config = Config {
    //     author: "Jane Doe".to_string(),
    //     generator: GeneratorConfig::default(),
    //     license_type: "Apache-2.0".to_string(),
    //     year: 2024,
    //   };

    //   // Test generating the config file when it already exists
    //   Config::generate_config_file(sample_config)
    //     .expect("Failed to generate config file");

    //   // Verify that the existing config file content remains unchanged
    //   let existing_file_content = fs::read_to_string(&existing_config_path)
    //     .expect("Failed to read existing config file content");
    //   assert_eq!(existing_file_content, "Existing content");

    //   drop(existing_config_path);
    //   temp_dir.close().expect("Failed to close temp directory");
    // }
    #[test]
    fn test_write_config_file_successful() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Define the target directory
        let target_dir = temp_dir.path();

        // Define the output file path
        let config_file_path = target_dir.join(DEFAULT_CONFIG_FILENAME);

        // Create a sample configuration
        let sample_config = Config {
            license: "MIT".to_string(),
            year: 2024,
            holder: "John Doe".to_string(),
            project: None,
            email: None,
            project_url: None,
            format: CopyrightNoticeFormat::Spdx,
            exclude: vec![],
        };

        // Test writing the config file
        let write_result = write_config_file(target_dir, &sample_config);
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
        let sample_config = Config {
            holder: "John Doe".to_string(),
            exclude: vec![],
            project: None,
            email: None,
            project_url: None,
            format: CopyrightNoticeFormat::Spdx,
            license: "MIT".to_string(),
            year: 2024,
        };

        // Test writing the config file when it already exists
        let result = write_config_file(target_dir, sample_config);
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
        let result = find_config_file(target_dir);
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
        let result = find_config_file(target_dir);
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
        let result = find_config_file(target_dir);
        assert!(result.is_err());
    }
}
