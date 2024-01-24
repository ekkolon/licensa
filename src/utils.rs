// Copyright 2024 Nelson Dominguez
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::Result;
use chrono::{Datelike, TimeZone};
use std::{
  fs::File,
  io::Write,
  path::{Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

/// Returns the current year as determined by the OS.
///
/// This function panics if the current timestamp cannot be determined
/// or is invalid, that is the timestamp seconds is out of range.
pub fn current_year() -> u16 {
  let current_timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Failed to get current timestamp")
    .as_secs();

  chrono::Utc
    .timestamp_opt(current_timestamp as i64, 0)
    .unwrap()
    .year() as u16
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
pub fn check_any_file_exists<P>(path: P, filenames: &[&str]) -> Option<PathBuf>
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
  fn test_current_year() {
    // Test the current_year function
    let current_year = current_year();

    // Get the current year using chrono
    let chrono_current_year = chrono::Utc::now().year() as u16;

    // Ensure that the current year matches the one obtained from chrono
    assert_eq!(current_year, chrono_current_year);
  }

  #[test]
  fn test_check_any_file_exists_single_file_exists() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let base_path = temp_dir.path();

    // Create a sample file in the temporary directory
    let sample_filename = "file1.txt";
    let sample_file_path = base_path.join(sample_filename);
    File::create(&sample_file_path).expect("Failed to create sample file");

    // Test when the single file exists
    let result = check_any_file_exists(base_path, &[sample_filename]);
    assert_eq!(result, Some(sample_file_path.clone()));

    // Cleanup
    drop(sample_file_path);
    temp_dir.close().expect("Failed to close temp directory");
  }

  #[test]
  fn test_check_any_file_exists_multiple_files_exist() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let base_path = temp_dir.path();

    // Create sample files in the temporary directory
    let filenames = ["file1.txt", "file2.txt", "file3.txt"];
    for &filename in &filenames {
      let file_path = base_path.join(filename);
      File::create(&file_path).expect("Failed to create sample file");

      // Cleanup
      drop(file_path);
    }

    // Test when multiple files exist
    let result = check_any_file_exists(base_path, &filenames);
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
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let base_path = temp_dir.path();

    // Test when none of the files exist
    let result = check_any_file_exists(base_path, &["nonexistent_file.txt"]);
    assert_eq!(result, None);
  }
}
