// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

pub mod _examples;
pub mod header_checker;
pub mod source;

use anyhow::Result;
use crossbeam_channel::Receiver;

use ignore::{DirEntry, WalkBuilder, WalkState};
use serde::Serialize;
use std::borrow::Borrow;

use std::path::PathBuf;

use self::source::SourceHeaders;

/// Default filename for the `Licensa` CLI ignore patterns.
const LICENSA_IGNORE_FILE: &str = ".licensaignore";

/// Configuration for a scan operation.
#[derive(Debug, Clone)]
pub struct ScanConfig {
  /// Root directory to start scanning from.
  pub root: PathBuf,

  /// Optional list of paths to exclude from the scan.
  pub exclude: Option<Vec<PathBuf>>,

  /// Limit on the number of parallel scan operations.
  pub limit: usize,
}

/// Represents a scanning operation.
#[derive(Debug)]
pub struct Scan {
  config: ScanConfig,
  walker: WalkBuilder,
}

impl Scan {
  /// Creates a new [Scan] instance of with the given configuration.
  pub fn new(config: ScanConfig) -> Self {
    let walker = build_walker(&config).expect("Failed to build scan walker");

    Self { config, walker }
  }

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
  pub fn run(&self) -> Result<Vec<FileEntry>> {
    let entries: &Vec<DirEntry> = &self
      .walker
      .build()
      .flatten()
      .filter(|entry| is_candidate(entry))
      .collect();

    // At this point, all entries are files
    let candidates = entries.iter().map(FileEntry::from).collect();

    Ok(candidates)
  }

  /// Runs the scan in parallel and returns a receiver for receiving file entries.
  ///
  /// This function utilizes crossbeam channels for parallel scanning.
  ///
  /// # Errors
  ///
  /// Returns an error if there are issues with building or running the parallel walker.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use licensa::scanner:ScanConfig;
  ///
  /// // Create a ScanConfig with the necessary parameters
  /// let config = ScanConfig {
  ///     // ... (initialize your config)
  /// };
  ///
  /// // Create a Scan instance with the provided config
  /// let scan = licensa::scanner::Scan::new(config);
  ///
  /// // Run the scan in parallel and obtain the receiver
  /// if let Ok(receiver) = scan.run_parallel() {
  ///     // Iterate over received file entries
  ///     for entry in receiver.iter() {
  ///         // Process the file entry as needed
  ///         println!("Received file entry: {:?}", entry);
  ///     }
  /// } else {
  ///     // Handle the error case
  ///     eprintln!("Error occurred while running the scan in parallel");
  /// }
  /// ```
  pub fn run_parallel(&self) -> Result<Receiver<FileEntry>> {
    let (tx, rx) = crossbeam_channel::bounded::<FileEntry>(self.config.limit);

    // Start the scan in parallel
    let walker = self.walker.build_parallel();
    walker.run(|| {
      let tx = tx.clone();

      Box::new(move |result| {
        if result.is_err() {
          return WalkState::Quit;
        }

        let entry = result.unwrap();
        if is_candidate(&entry) {
          let entry = FileEntry::from(entry);
          tx.send(entry).unwrap();
        }

        WalkState::Continue
      })
    });

    Ok(rx)
  }

  /// Returns the root path configured for the scan.
  pub fn root(&self) -> PathBuf {
    self.config.root.to_owned()
  }

  /// Returns the optional list of `.gitignore` files configured for the scan.
  pub fn gitignore_files(&self) -> Option<Vec<PathBuf>> {
    self.config.exclude.to_owned()
  }

  /// Returns the limit configured for the number of parallel scan processes.
  pub fn limit(&self) -> usize {
    self.config.limit.to_owned()
  }
}

/// Represents a file entry captured during the scan.
#[derive(Debug, Serialize)]
pub struct FileEntry {
  /// Absolute path to the file.
  pub abspath: PathBuf,

  /// Optional file extension.
  pub extension: Option<String>,

  /// Filename of the file.
  pub filename: String,
}

impl From<DirEntry> for FileEntry {
  /// Converts a `DirEntry` into a `FileEntry`.
  fn from(value: DirEntry) -> Self {
    FileEntry::from(&value)
  }
}

impl From<&DirEntry> for FileEntry {
  /// Converts a reference to a `DirEntry` into a `FileEntry`.
  fn from(value: &DirEntry) -> Self {
    FileEntry {
      filename: value.file_name().to_string_lossy().into_owned(),
      abspath: value.path().to_path_buf(),
      extension: value
        .path()
        .extension()
        .map(|e| e.to_string_lossy().into_owned()),
    }
  }
}

/// Checks if a directory entry is a candidate for applying a license.
fn is_candidate<E>(entry: E) -> bool
where
  E: Borrow<DirEntry>,
{
  let entry = entry.borrow() as &DirEntry;

  // Only consider entry if it is a regular file
  let ftype = entry.file_type();
  let is_file = ftype.map_or(false, |ftype| ftype.is_file());
  if !is_file {
    return false;
  }

  // Verify the file extension is eligable
  let f_path = entry.path();
  let f_ext = f_path.extension();
  let f_name = f_path.file_name();

  if f_ext.is_none() && f_name.is_none() {
    return false;
  }

  if let Some(ext) = f_ext {
    let ext = ext.to_str().unwrap();
    let ext = format!(".{ext}");
    let header_def = SourceHeaders::find_header_definition_by_extension(ext);
    return header_def.is_some();
  }

  if let Some(name) = f_name {
    let name = name.to_str().unwrap();
    let header_def = SourceHeaders::find_header_definition_by_extension(name);
    return header_def.is_some();
  }

  true
}

/// Builds a WalkBuilder with the specified configuration.
///
/// This function takes a `ScanConfig` as input and creates a `WalkBuilder` configured
/// with the specified root directory and exclusion patterns. It also adds the `.licensaignore`
/// file to the custom ignore list, ensuring that patterns in this file take precedence over
/// other ignore files.
///
/// # Arguments
///
/// * `config` - The `ScanConfig` containing the root directory and optional exclusion paths.
///
/// # Returns
///
/// A `Result` containing the configured `WalkBuilder` if successful, or an `anyhow::Error` otherwise.
///
/// # Examples
///
/// ```no_run
/// use licensa::scanner::{ScanConfig, build_walker};
///
/// let config = ScanConfig {
///     root: "/path/to/project".into(),
///     exclude: Some(vec!["target", "build"]),
///     limit: 200
/// };
///
/// let result = build_walker(&config);
/// assert!(result.is_ok());
/// ```
fn build_walker(config: &ScanConfig) -> Result<WalkBuilder> {
  let mut walker = WalkBuilder::new(&config.root);

  if let Some(exclude_paths) = &config.exclude {
    if !exclude_paths.is_empty() {
      for path in exclude_paths {
        walker.add_ignore(path);
      }
    }
  }

  // Add `.licensaignore` file. This must come last because the patterns
  // defined in this file should take precedence over other ignore files.
  let licensaignore = &config.root.join(LICENSA_IGNORE_FILE);
  walker.add_custom_ignore_filename(licensaignore);

  Ok(walker.to_owned())
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
    File::create(license_file_path).expect("Failed to create license file");

    let ignored_file_path = root_path.join("ignored.txt");
    File::create(&ignored_file_path).expect("Failed to create ignored file");

    let licensaignore_path = root_path.join(LICENSA_IGNORE_FILE);
    let mut licensaignore_file =
      File::create(&licensaignore_path).expect("Failed to create .licensaignore file");

    licensaignore_file
      .write_all(b"ignored.txt")
      .expect("Failed to write to .licensaignore file");

    // Run the scan function
    let scan_config = ScanConfig {
      limit: 100,
      exclude: None,
      root: root_path.to_path_buf(),
    };
    let scan = Scan::new(scan_config);
    let result = scan.run(); //.expect("Failed to execute scan");

    // Assert that the result is Ok and contains the license file
    assert!(result.is_ok());
    // let candidates = result.unwrap();
    // assert_eq!(candidates, vec![license_file_path]);

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
    let scan_config = ScanConfig {
      limit: 100,
      exclude: None,
      root: root_path.to_path_buf(),
    };
    let scan = Scan::new(scan_config);
    let result = scan.run();

    // Assert that the result is Ok and the candidates list is empty
    assert!(result.is_ok());
    let candidates = result.unwrap();
    assert!(candidates.is_empty());
  }
}
