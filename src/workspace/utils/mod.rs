// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::{Path, PathBuf};

#[cfg(test)]
use std::{fs::File, io::Write};

use super::{Error, Result};

/// Writes pretty-formatted JSON data to a file, creating the file if it does not exist.
///
/// # Arguments
///
/// * `file_path` - The path to the file where JSON data will be written.
/// * `json_data` - The JSON data to be written to the file.
///
/// # Errors
///
/// Returns an error if there are issues creating or writing to the file.
#[cfg(test)]
pub fn write_json<P: AsRef<Path>>(file_path: P, json_data: &serde_json::Value) -> Result<()> {
    let mut file = File::create(&file_path)?;
    let json_string = serde_json::to_string_pretty(json_data)?;
    file.write_all(json_string.as_bytes())?;
    file.flush()?;
    Ok(())
}

#[inline]
pub fn verify_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    if !path.as_ref().is_dir() {
        return Err(Error::NotADirectory(path.as_ref().into()));
    }

    Ok(())
}

/// Checks if any of the specified filenames exist in the given path.
///
/// # Arguments
///
/// * `path` - The base path where the function checks for the existence of the specified files.
/// * `filenames` - A slice of strings representing the filenames to check for existence.
///
/// # Returns
///
/// Returns an `Option<PathBuf>` representing the path of the first existing file, if any.
/// Returns `None` if none of the specified files exist in the given path.
///
/// # Panics
///
/// This function does not intentionally panic
pub fn resolve_any_path<P>(path: P, filenames: &[&str]) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    let mut out_path: Option<PathBuf> = None;
    filenames.iter().for_each(|filename: &&str| {
        let file_path = path.as_ref().join(filename);
        if file_path.exists() {
            let _ = out_path.insert(file_path);
        }
    });

    out_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use tempfile::tempdir;

    #[test]
    fn test_write_json_invalid_file_path() {
        let invalid_file_path = "/nonexistent_directory/output.json";
        let json_data = serde_json::json!({
            "name": "John Doe",
            "age": 30,
            "city": "Example City"
        });
        let result = write_json(invalid_file_path, &json_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_json_with_seek() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("output.json");
        let json_data = serde_json::json!({
            "name": "John Doe",
            "age": 30,
            "city": "Example City"
        });

        write_json(&file_path, &json_data).expect("Failed to write JSON to file");
        assert!(file_path.exists());

        let mut file = File::open(&file_path).expect("Failed to open file");
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)
            .expect("Failed to read file content");

        let expected_content =
            serde_json::to_string_pretty(&json_data).expect("Failed to serialize JSON");
        assert_eq!(file_content, expected_content);

        // Seek back to the beginning of the file
        file.seek(SeekFrom::Start(0))
            .expect("Failed to seek back to the beginning");

        // Verify the content of the file after seeking back
        let mut file_content_after_seek = String::new();
        file.read_to_string(&mut file_content_after_seek)
            .expect("Failed to read file content after seek");
        assert_eq!(file_content_after_seek, expected_content);

        // Cleanup
        drop(file_path);
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_check_any_file_exists_single_file_exists() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let base_path = temp_dir.path();
        let sample_filename = "file1.txt";
        let sample_file_path = base_path.join(sample_filename);
        File::create(&sample_file_path).expect("Failed to create sample file");

        let result = resolve_any_path(base_path, &[sample_filename]);
        assert_eq!(result, Some(sample_file_path.clone()));

        // Cleanup
        drop(sample_file_path);
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_check_any_file_exists_multiple_files_exist() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let base_path = temp_dir.path();

        let filenames = ["file1.txt", "file2.txt", "file3.txt"];
        for &filename in &filenames {
            let file_path = base_path.join(filename);
            File::create(&file_path).expect("Failed to create sample file");

            // Cleanup
            drop(file_path);
        }

        let result = resolve_any_path(base_path, &filenames);
        assert!(result.is_some());
        assert!(filenames.iter().any(|&filename| {
            result
                .as_ref()
                .map_or(false, |path| path.ends_with(filename))
        }));

        // Cleanup
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_check_any_file_exists_no_file_exists() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let base_path = temp_dir.path();
        let result = resolve_any_path(base_path, &["nonexistent_file.txt"]);
        assert_eq!(result, None);
    }
}
