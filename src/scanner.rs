// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::{DirEntry, WalkBuilder};
use std::borrow::Borrow;

use std::path::{Path, PathBuf};

const LICENSA_IGNORE_FILE: &str = ".licensaignore";

/// Scans a directory for license candidates.
///
/// Files listed in a `.licensaignore` or `.gitignore` file are excluded
/// from the resulting list of candidates.
///
/// # Arguments
///
/// * `root` - The root path of the directory to scan.
///
/// # Returns
///
/// A `Result` containing a vector of absolute paths to license candidates,
/// or an `Err` if an error occurs.
pub fn scan<P>(root: P) -> Result<Vec<PathBuf>>
where
  P: AsRef<Path>,
{
  let root_dir = root.as_ref();

  // Path to '.licensaignore' file
  let licensaignore = root_dir.join(LICENSA_IGNORE_FILE);

  let mut gitignore_builder = GitignoreBuilder::new(root_dir);
  gitignore_builder.add(licensaignore);

  let gitignore = gitignore_builder.build()?;

  // Find license candidates
  let candidates: Vec<PathBuf> = find_license_candidates(root_dir, &gitignore)
    .iter()
    .map(|candidate| candidate.to_path_buf())
    .collect();

  Ok(candidates)
}

/// Finds license candidates in a given directory based on Gitignore rules.
///
/// # Arguments
///
/// * `path` - The path of the directory to search for license candidates.
/// * `gitignore` - A Gitignore instance specifying files to be ignored.
///
/// # Returns
///
/// A vector of absolute paths to license candidates.
fn find_license_candidates<R, I>(path: R, gitignore: I) -> Vec<PathBuf>
where
  R: AsRef<Path>,
  I: Borrow<Gitignore>,
{
  let walker = WalkBuilder::new(path);

  let entries: &Vec<DirEntry> = &walker
    .build()
    .flatten()
    .filter(|entry| is_license_candidates(gitignore.borrow(), entry))
    .collect();

  entries
    .iter()
    .map(|entry| entry.path().to_path_buf())
    .collect()
}

/// Checks whether a directory entry is a license candidate, considering Gitignore rules.
///
/// # Arguments
///
/// * `gitignore` - A Gitignore instance specifying files to be ignored.
/// * `entry` - A directory entry to check.
///
/// # Returns
///
/// `true` if the entry is a license candidate, `false` otherwise.
fn is_license_candidates<I, E>(gitignore: I, entry: E) -> bool
where
  I: Borrow<Gitignore>,
  E: Borrow<DirEntry>,
{
  let is_file = entry
    .borrow()
    .file_type()
    .map_or(false, |file_type| file_type.is_file());

  let is_gitignored = gitignore
    .borrow()
    .matched(entry.borrow().path(), false)
    .is_ignore();

  is_file && !is_gitignored
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::fs::File;
  use std::io::Write;

  // Helper function to create temporary directory and files
  fn create_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
  }

  #[test]
  fn test_scan_with_license_candidates() {
    // Create a temporary directory and files
    let temp_dir = create_temp_dir();
    let root_path = temp_dir.path();

    let license_file_path = root_path.join("LICENSE");
    File::create(&license_file_path).expect("Failed to create license file");

    let ignored_file_path = root_path.join("ignored.txt");
    File::create(&ignored_file_path).expect("Failed to create ignored file");

    let licensaignore_path = root_path.join(LICENSA_IGNORE_FILE);
    let mut licensaignore_file =
      File::create(&licensaignore_path).expect("Failed to create .licensaignore file");
    licensaignore_file
      .write_all(b"ignored.txt")
      .expect("Failed to write to .licensaignore file");

    // Run the scan function
    let result = scan(root_path);

    // Assert that the result is Ok and contains the license file
    assert!(result.is_ok());
    let candidates = result.unwrap();
    assert_eq!(candidates, vec![license_file_path]);

    drop(ignored_file_path);
    drop(licensaignore_path);
    temp_dir.close().expect("Failed to close temp directory");
  }

  #[test]
  fn test_scan_without_license_candidates() {
    // Create a temporary directory without license candidates
    let temp_dir = create_temp_dir();
    let root_path = temp_dir.path();

    // Run the scan function
    let result = scan(root_path);

    // Assert that the result is Ok and the candidates list is empty
    assert!(result.is_ok());
    let candidates = result.unwrap();
    assert!(candidates.is_empty());
  }
}
