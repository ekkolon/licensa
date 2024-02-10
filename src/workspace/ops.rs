// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Provides functions for managing workspace configurations and ignore files.
//!
//! This module offers tools for:
//!
//! - Locating and reading Licensa configuration files.
//! - Writing configurations and ignore files to workspace directories.
//! - Ensuring workspace directories are valid and checking for existing files.
//! - Cleaning config specific JSON values by removing null fields.
//!
//! # Errors
//!
//! Functions in this module may return `WorkspaceResult` errors to indicate issues like:
//!
//! - Missing configuration files.
//! - File reading or writing errors.
//! - Invalid workspace directories.
//! - Existing configuration or ignore files when not expected.
//!
//! # Related Modules
//!
//! - `workspace::error` contains the `WorkspaceError` type used for error handling.

use crate::workspace::error::{WorkspaceError, WorkspaceResult};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use std::borrow::Borrow;
use std::fs::{self};
use std::path::{Path, PathBuf};

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
pub fn read_config_into<T, P, F>(workspace_root: P, file_name: F) -> WorkspaceResult<T>
where
    T: ?Sized,
    for<'de> T: Deserialize<'de>,
    P: AsRef<Path>,
    F: AsRef<str>,
{
    let config = read_config(workspace_root, file_name)?;
    let content = serde_json::from_str::<T>(&config)
        .with_context(|| "failed to parse .licensarc config file")?;
    Ok(content)
}

/// Reads the configuration file from the specified workspace directory.
///
/// # Arguments
///
/// * `workspace_root` - The directory containing the configuration file.
/// * `file_name` - The name of the configuration file to read.
///
/// # Errors
///
/// Returns an error if:
///
/// * The `workspace_root` directory does not exist.
/// * The specified configuration file does not exist within `workspace_root`.
/// * The configuration file exists but is not a valid file.
/// * There's an error reading the contents of the configuration file.
pub fn read_config<P, F>(workspace_root: P, file_name: F) -> WorkspaceResult<String>
where
    P: AsRef<Path>,
    F: AsRef<str>,
{
    let workspace_root = workspace_root.as_ref();
    ensure_dir(workspace_root)?;

    let file_path = workspace_root.join(file_name.as_ref());
    if !file_path.exists() {
        let err = WorkspaceError::Generic(
            anyhow!("path does not exist: {}", file_path.display())
                .context("failed to read workspace config file"),
        );
        return Err(err);
    }
    if !file_path.is_file() {
        let err = WorkspaceError::Generic(
            anyhow!("{} is not a file", file_path.display())
                .context("failed to read workspace config file"),
        );
        return Err(err);
    }

    let config =
        fs::read_to_string(file_path).with_context(|| "failed to read workspace config file")?;

    Ok(config)
}

/// Reads a configuration file from the specified workspace directory or its parent directories,
/// deserializing it into the provided type.
///
/// # Arguments
///
/// * `workspace_root` - The starting directory for the search.
/// * `file_name` - The name of the configuration file to read.
///
/// # Returns
///
/// * `Ok(Some(config))` if the configuration file is found and successfully parsed.
/// * `Ok(None)` if the configuration file is not found in any of the parent directories.
/// * `Err(WorkspaceError)` if there's an error reading or parsing the file content.
pub fn resolve_config_into<T, P, F>(workspace_root: P, file_name: F) -> WorkspaceResult<Option<T>>
where
    T: ?Sized,
    for<'de> T: Deserialize<'de>,
    P: AsRef<Path>,
    F: AsRef<str>,
{
    let workspace_root = workspace_root.as_ref();
    ensure_dir(workspace_root)?;
    if let Some(path) = resolve_config_path(workspace_root, file_name) {
        let content =
            fs::read_to_string(path).with_context(|| "failed to read .licensarc config file")?;

        let config = serde_json::from_str::<T>(&content)
            .with_context(|| "failed to parse .licensarc config file")?;

        return Ok(Some(config));
    }

    Ok(None)
}

/// Searches for a config file with the specified name in the parent directories.
///
/// # Arguments
///
/// * `root_path` - The starting directory for the search.
/// * `file_name` - The name of the config file to find.
///
/// # Returns
///
/// * `Some(PathBuf)` if the file is found in a parent directory.
/// * `None` if the file is not found in any of the parent directories.
pub fn resolve_config_path<R, F>(root_path: R, file_name: F) -> Option<PathBuf>
where
    R: AsRef<Path>,
    F: AsRef<str>,
{
    let mut current_dir = root_path.as_ref().to_path_buf();
    let mut file_path = current_dir.join(file_name.as_ref());

    loop {
        if file_path.is_file() {
            return Some(file_path);
        }
        if !current_dir.pop() {
            break;
        }
        file_path = current_dir.join(file_name.as_ref());
    }

    None
}

/// Writes a configuration to a file in the specified directory.
///
/// # Arguments
///
/// * `workspace_root` - The directory to write the configuration file to.
/// * `file_name` - The name of the configuration file.
/// * `config` - The configuration data to write, as a serializable type.
///
/// # Errors
///
/// Returns an error if:
///
/// * The directory cannot be created.
/// * The configuration cannot be serialized.
/// * The configuration is not a valid JSON object.
/// * The file cannot be written.
pub fn save_config<P, F, T>(workspace_root: P, file_name: F, config: T) -> WorkspaceResult<()>
where
    P: AsRef<Path>,
    F: AsRef<str>,
    T: Serialize,
{
    let workspace_root = workspace_root.as_ref();
    ensure_dir(workspace_root)?;
    let config = serde_json::to_value(config.borrow())?;
    // Ensure config is an object
    if !config.is_object() {
        let err = anyhow!(WorkspaceError::InvalidConfigDataType)
            .context("failed to save workspace config file");
        return Err(err.into());
    }
    let config = remove_null_fields(config);
    let config = serde_json::to_string_pretty(&config)
        .with_context(|| "failed to serialize .licensarc config file")?;
    let out_path = workspace_root.join(file_name.as_ref());
    fs::write(out_path, config).with_context(|| "failed to save .licensarc config file")?;
    Ok(())
}

/// Saves a workspace ignore file to the specified directory.
///
/// # Arguments
///
/// * `workspace_root` - The directory to save the ignore file in.
/// * `file_name` - The name of the ignore file.
/// * `content` - The content to write to the ignore file, as a byte slice.
///
/// # Errors
///
/// Returns an error if:
///
/// * The directory cannot be created.
/// * The ignore file already exists in the specified directory.
/// * The file cannot be written.
pub fn save_ignore_file<P, F, C>(workspace_root: P, file_name: F, content: C) -> WorkspaceResult<()>
where
    P: AsRef<Path>,
    F: AsRef<str>,
    C: AsRef<[u8]>,
{
    let workspace_root = workspace_root.as_ref();
    ensure_dir(workspace_root)?;
    let ignore_path = workspace_root.join(file_name.as_ref());
    if ignore_path.exists() {
        let err = WorkspaceError::IgnoreFileAlreadyExists(workspace_root.to_path_buf());
        return Err(err);
    }
    fs::write(ignore_path, content).with_context(|| "failed to save workspace ignore file")?;
    Ok(())
}

/// Ensures that a configuration file with the specified name does not exist in the workspace directory.
///
/// # Arguments
///
/// * `workspace_root` - The directory to check for the configuration file.
/// * `config_file_name` - The name of the configuration file to verify absence.
///
/// # Errors
///
/// Returns an error if:
///
/// * The directory does not exist and cannot be created.
/// * The configuration file already exists in the specified directory.
pub fn ensure_config_missing<P, F>(workspace_root: P, config_file_name: F) -> WorkspaceResult<()>
where
    P: AsRef<Path>,
    F: AsRef<str>,
{
    let workspace_root = workspace_root.as_ref();
    ensure_dir(workspace_root)?;
    if has_config(workspace_root, config_file_name) {
        let err = WorkspaceError::ConfigFileAlreadyExists(workspace_root.to_path_buf());
        return Err(err);
    }
    Ok(())
}

/// Verifies that the specified path exists and is a directory.
///
/// # Arguments
///
/// * `path` - The path to check.
///
/// # Errors
///
/// Returns an error if:
///
/// * The path does not exist.
/// * The path exists but is not a directory.
#[inline]
pub fn ensure_dir<P: AsRef<Path>>(path: P) -> WorkspaceResult<()> {
    if !path.as_ref().is_dir() {
        let err = WorkspaceError::NotADirectory(path.as_ref().to_path_buf());
        return Err(err);
    }
    Ok(())
}

/// Checks if a configuration file with the specified name exists in the given directory.
///
/// # Arguments
///
/// * `workspace_root` - The directory to search for the configuration file.
/// * `file_name` - The name of the configuration file to check for.
///
/// # Returns
///
/// `true` if the configuration file exists in the directory, `false` otherwise.
pub fn has_config<P, F>(workspace_root: P, file_name: F) -> bool
where
    P: AsRef<Path>,
    F: AsRef<str>,
{
    let path = workspace_root.as_ref().join(file_name.as_ref());
    path.exists() && path.is_file()
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
fn remove_null_fields(value: Value) -> Value {
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
    use crate::utils::testing::create_temp_file;

    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize)]
    struct ExampleWsConfig {
        prop1: String,
        prop2: usize,
    }

    #[test]
    fn test_save_ws_config() {
        let dir = tempdir().unwrap();

        // Expectes a JSON object but we provie a string
        let result = save_config(dir.as_ref(), "conf.toml", "str".to_string());
        let expected: Result<_, WorkspaceError> =
            Err::<(), WorkspaceError>(WorkspaceError::InvalidConfigDataType);
        assert!(result.is_err());
        assert!(matches!(result, expected));

        // Provide JSON object
        let result = save_config(
            dir.as_ref(),
            "conf.toml",
            ExampleWsConfig {
                prop1: "This prop has no meaning".to_string(),
                prop2: 23,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_ws_config_into() {
        let dir = tempdir().unwrap();

        // Create tmp file with dummy config as json
        let tmp_config_path = dir.as_ref().join("conf.json");
        let json_data = serde_json::to_string(&ExampleWsConfig {
            prop1: "This prop has no meaning".to_string(),
            prop2: 23,
        })
        .unwrap();
        fs::write(tmp_config_path, json_data).unwrap();

        let result: Result<ExampleWsConfig, WorkspaceError> =
            read_config_into(dir.as_ref(), "conf.json");
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_ws_config() {
        let dir = tempdir().unwrap();

        // At this point no config path exist so error must be some
        let result = read_config(dir.as_ref(), "conf.json");
        let expected: Result<_, WorkspaceError> =
            Err::<(), WorkspaceError>(WorkspaceError::MissingConfigFile);
        assert!(result.is_err());
        assert!(matches!(result, expected));

        // Create empty tmp config file. Content of read op must be ok.
        let tmp_config_path = dir.as_ref().join("conf.json");
        fs::write(tmp_config_path, b"example test config file").unwrap();
        let result = read_config(dir.as_ref(), "conf.json");
        assert!(result.is_ok());

        dir.close().unwrap();
    }

    #[test]
    fn test_ensure_missing_ws_config() {
        let (dir, config_path) = create_temp_file("conf.toml");
        let result = ensure_config_missing(dir.as_ref(), "conf.toml");
        let expected: Result<_, WorkspaceError> = Err::<(), WorkspaceError>(
            WorkspaceError::ConfigFileAlreadyExists(dir.as_ref().to_path_buf()),
        );
        assert!(matches!(result, expected));

        dir.close().unwrap();
    }

    #[test]
    fn test_save_ignore_file() {
        let dir = tempdir().unwrap();

        let file_name = ".ignoreme";
        let file_content = "this should be replaced with actual glob patterns";
        let file_result = save_ignore_file(dir.as_ref(), file_name, file_content);
        assert!(file_result.is_ok());

        // Attempting to save an ignore file in the same dir should result
        // in an WorkspaceError::IgnoreFileAlreadyExists err.
        let file_result = save_ignore_file(dir.as_ref(), ".ignoremetoo", "more ignore patterns");
        let expected: Result<_, WorkspaceError> = Err::<(), WorkspaceError>(
            WorkspaceError::IgnoreFileAlreadyExists(dir.as_ref().to_path_buf()),
        );
        assert!(matches!(file_result, expected));

        // Assert the saved ignore file has the same byte length as `LICENSA_IGNORE` static ref
        let saved_path = dir.as_ref().join(file_name);
        let saved_content = fs::read_to_string(saved_path).unwrap();
        assert_eq!(file_content.len(), saved_content.len());

        dir.close().unwrap();
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

    #[test]
    fn test_ensure_is_dir() {
        let dir = tempdir().unwrap();

        let result = ensure_dir(dir.as_ref());
        assert!(result.is_ok());

        let useless_file_path = dir.as_ref().join("uselessfile.txt");

        let result = ensure_dir(&useless_file_path);
        let expected: Result<(), WorkspaceError> =
            Err(WorkspaceError::NotADirectory(useless_file_path));
        assert!(result.is_err());
        assert!(matches!(result, expected));

        dir.close().unwrap();
    }

    #[test]
    fn test_find_workspace_config() {
        let root_dir = tempdir().unwrap();
        let root_path = root_dir.as_ref();
        let file_name = "test_file.txt";

        // Create a temporary file in a parent directory
        let mut parent_dir = root_path.to_path_buf();
        parent_dir.push("sub1");
        let sub1_dir = &parent_dir.clone();
        let file_path = sub1_dir.join(file_name);
        parent_dir.push("sub2");

        std::fs::create_dir_all(&parent_dir).unwrap();
        std::fs::write(&file_path, "test content").unwrap();

        let result = resolve_config_path(sub1_dir, file_name);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), file_path);

        // Clean up
        root_dir.close().unwrap();
    }

    #[test]
    fn test_find_workspace_config_not_found() {
        let root_path = Path::new("/tmp");
        let file_name = "nonexistent_file.txt";
        let result = resolve_config_path(root_path, file_name);
        assert!(result.is_none());
    }
}
