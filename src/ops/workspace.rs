// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::utils::{resolve_any_path, verify_dir, write_json};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use std::borrow::Borrow;
use std::fs;
use std::path::Path;

lazy_static! {
    static ref LICENSA_IGNORE: &'static str = std::include_str!("../../.licensaignore");
}

const LICENSA_IGNORE_FILENAME: &str = ".licensaignore";

const DEFAULT_CONFIG_FILENAME: &str = ".licensarc";
const POSSIBLE_CONFIG_FILENAMES: &[&str] = &[".licensarc", ".licensarc.json"];

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
pub fn find_workspace_config<P>(workspace_root: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let workspace_root = workspace_root.as_ref();
    verify_dir(workspace_root)?;
    let config_path = resolve_any_path(workspace_root, POSSIBLE_CONFIG_FILENAMES);
    if let Some(path) = config_path {
        let content = fs::read_to_string(path)?;
        return Ok(content);
    }
    Err(anyhow!(
        "None of the configuration files {:?} found in the current directory.",
        POSSIBLE_CONFIG_FILENAMES
    ))
}

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
pub fn resolve_workspace_config<T>(workspace_root: impl AsRef<Path>) -> Result<T>
where
    T: ?Sized,
    for<'de> T: Deserialize<'de>,
{
    let workspace_root = workspace_root.as_ref();
    verify_dir(workspace_root)?;

    let config_path = resolve_any_path(workspace_root, POSSIBLE_CONFIG_FILENAMES);

    if let Some(path) = config_path {
        let content = fs::read_to_string(path)?;
        let content = serde_json::from_str::<T>(&content)?;
        return Ok(content);
    }

    Err(anyhow!(
        "None of the configuration files {:?} found in the current directory.",
        POSSIBLE_CONFIG_FILENAMES
    ))
}

/// Try find a Licensa configuration file in the directory specified by `workspace_root`.
pub fn try_resolve_workspace_config<P>(workspace_root: P) -> Result<Option<Value>>
where
    P: AsRef<Path>,
{
    let workspace_root = workspace_root.as_ref();
    verify_dir(workspace_root)?;

    if let Some(path) = resolve_any_path(workspace_root, POSSIBLE_CONFIG_FILENAMES) {
        let content = fs::read_to_string(path)?;
        let content = serde_json::from_str::<Value>(&content)?;
        return Ok(Some(content));
    }

    Ok(None)
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
pub fn save_workspace_config<P, T>(workspace_root: P, config: T) -> Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let workspace_root = workspace_root.as_ref();
    verify_dir(workspace_root)?;
    let config = serde_json::to_value(config.borrow())?;
    let config = remove_null_fields(config);
    let config_path = workspace_root.join(DEFAULT_CONFIG_FILENAME);
    write_json(config_path, &config)?;
    Ok(())
}

pub fn throw_workspace_config_exists<P>(workspace_root: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let workspace_root = workspace_root.as_ref();
    verify_dir(workspace_root)?;

    if workspace_config_exists(workspace_root) {
        return Err(anyhow!(
            "Licensa is already initialized in the current directory",
        ));
    }
    Ok(())
}
pub fn throw_no_workspace_config_exists<P>(workspace_root: P) -> Result<()>
where
    P: AsRef<Path>,
{
    if !workspace_config_exists(workspace_root) {
        return Err(anyhow!(
            "Licensa config file not found in the current directory"
        ));
    }
    Ok(())
}

/// Check if config file already exists in provided path
pub fn workspace_config_exists<P>(workspace_root: P) -> bool
where
    P: AsRef<Path>,
{
    resolve_any_path(workspace_root, POSSIBLE_CONFIG_FILENAMES).map_or(false, |p| true)
}

/// Save `.licensaignore` file to provided directory.
pub fn save_workspace_ignore<P>(workspace_root: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let workspace_root = workspace_root.as_ref();
    verify_dir(workspace_root)?;

    let ignore_path = workspace_root.join(LICENSA_IGNORE_FILENAME);
    if ignore_path.exists() {
        return Err(anyhow!(
            ".licensaignore file already exists at '{}'",
            workspace_root.display()
        ));
    }

    fs::write(ignore_path, LICENSA_IGNORE.as_bytes())?;
    Ok(())
}

/// Recursively removes all fields with `null` values from a JSON object.
///
/// This function takes a serde_json Value representing a JSON object and recursively
/// removes all fields with `null` values. If the input value is not an object or
/// contains non-object values, it returns the value as is.
///
/// # Arguments
///
/// * `value` - A serde_json Value representing a JSON object.
///
/// # Returns
///
/// A serde_json Value with `null` fields removed.
///
/// # Examples
///
/// ```no_run,ignore
/// use serde_json::{json, Value};
/// use licensa::ops::workspace::remove_null_fields;
///
/// let json_value = json!({
///     "name": "John",
///     "age": null,
///     "address": {
///         "city": "New York",
///         "zip": null
///     },
///     "scores": [10, null, 20]
/// });
///
/// let cleaned_value = remove_null_fields(json_value);
///
/// assert_eq!(cleaned_value, json!({
///     "name": "John",
///     "address": {
///         "city": "New York"
///     },
///     "scores": [10, null, 20]
/// }));
/// ```
pub fn remove_null_fields(value: Value) -> Value {
    match value {
        Value::Null => Value::Null,
        Value::Bool(_) => value,
        Value::Number(_) => value,
        Value::String(_) => value,
        Value::Array(arr) => {
            let cleaned_array: Vec<Value> = arr.into_iter().map(remove_null_fields).collect();
            Value::Array(cleaned_array)
        }
        Value::Object(obj) => {
            let mut cleaned_obj: Map<String, Value> = Map::new();
            for (key, val) in obj {
                if val != Value::Null {
                    cleaned_obj.insert(key, remove_null_fields(val));
                }
            }
            Value::Object(cleaned_obj)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{fs::File, io::Read};
    use tempfile::tempdir;

    #[derive(Serialize)]
    struct ExampleWorkspace {
        prop1: String,
        prop2: usize,
    }

    #[test]
    fn test_write_config_file_successful() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Define the target directory
        let target_dir = temp_dir.path();

        // Define the output file path
        let config_file_path = target_dir.join(DEFAULT_CONFIG_FILENAME);

        // Create a sample configuration
        let sample_config = ExampleWorkspace {
            prop1: "hello world".to_string(),
            prop2: 1234,
        };

        // Test writing the config file
        let write_result = save_workspace_config(target_dir, &sample_config);
        assert!(write_result.is_ok());

        // Verify that the config file exists
        assert!(config_file_path.exists());

        // Verify the content of the config file
        let mut file = File::open(&config_file_path).expect("Failed to open config file");
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)
            .expect("Failed to read config file content");

        let expected_content = serde_json::to_string_pretty(&sample_config).unwrap();
        assert_eq!(file_content, expected_content);

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
        let sample_config = ExampleWorkspace {
            prop1: "hello world".to_string(),
            prop2: 1234,
        };

        // Test writing the config file when it already exists
        let result = save_workspace_config(target_dir, sample_config);
        assert!(result.is_ok());

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
        let result: Result<Value> = resolve_workspace_config(target_dir);
        assert!(result.is_err());

        // Verify the content of the config file
        // let expected_content = String::new(); // Adjust with actual content
        // assert_eq!(result.unwrap(), expected_content);

        // Cleanup
        drop(sample_config_path);
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_find_config_file_multiple_files_exist() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let target_dir = temp_dir.path();

        for &filename in POSSIBLE_CONFIG_FILENAMES {
            let config_path = target_dir.join(filename);
            File::create(&config_path).expect("Failed to create sample config file");

            // Cleanup
            drop(config_path);
        }

        let result: Result<Value> = resolve_workspace_config(target_dir);
        assert!(result.is_err());

        // TODO:  Verify the content of the config file

        // Cleanup
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_find_config_file_no_file_exists() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let target_dir = temp_dir.path();
        let result: Result<Value> = resolve_workspace_config(target_dir);
        // assert!(result.is_err());
    }

    #[test]
    fn test_remove_null_fields() {
        let json_value = json!({
            "name": "John",
            "age": null,
            "address": {
                "city": "New York",
                "zip": null
            },
            "scores": [10, null, 20]
        });

        let cleaned_value = remove_null_fields(json_value.clone());

        assert_eq!(
            cleaned_value,
            json!({
                "name": "John",
                "address": {
                    "city": "New York"
                },
                "scores": [10, null, 20]
            })
        );

        // Ensure input value is not modified
        assert_eq!(
            json_value,
            json!({
                "name": "John",
                "age": null,
                "address": {
                    "city": "New York",
                    "zip": null
                },
                "scores": [10, null, 20]
            })
        );
    }
}
