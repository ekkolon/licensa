// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::template::header::SourceHeaders;
use rayon::prelude::*;

use crossbeam_channel::Receiver;
use ignore::{DirEntry, WalkState};

use std::borrow::Borrow;
use std::path::{Path, PathBuf};

use crate::workspace::walker::{Walk, WalkBuilder};

/// Default filename for the `Licensa` CLI ignore patterns.
const LICENSA_IGNORE_FILE: &str = ".licensaignore";

/// Configuration for a scan operation.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Root directory to start scanning from.
    pub root: PathBuf,

    /// Optional list of paths to exclude from the scan.
    pub exclude: Option<Vec<&'static str>>,

    /// Limit on the number of parallel scan operations.
    pub limit: usize,
}

/// Represents a scanning operation.
pub struct Scan {
    config: ScanConfig,
    walker: Walk,
}

impl Scan {
    /// Creates a new [Scan] instance of with the given configuration.
    pub fn new(config: ScanConfig) -> Self {
        let exclude = config.exclude.clone().unwrap_or_default();
        let mut walk_builder = WalkBuilder::new(&config.root);
        walk_builder.add_ignore(LICENSA_IGNORE_FILE);

        walk_builder.exclude(Some(exclude)).unwrap();
        let walker = walk_builder.build().unwrap();

        Self { config, walker }
    }

    pub fn find_candidates(mut self) -> Vec<DirEntry> {
        self.walker.quit_while(|res| res.is_err());
        self.walker
            .send_while(|res| res.is_ok() && is_candidate(res.unwrap()));
        self.walker.max_capacity(None);
        self.walker
            .run_task()
            .iter()
            .par_bridge()
            .into_par_iter()
            .filter_map(Result::ok)
            .collect()
    }

    /// Runs the scan in parallel and returns a receiver for receiving file entries.
    ///
    /// This function utilizes crossbeam channels for parallel scanning.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues with building or running the parallel walker.
    pub fn run(self) -> Receiver<FileEntry> {
        let (tx, rx) = crossbeam_channel::bounded::<FileEntry>(self.config.limit);
        self.walker.run(|| {
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

        rx
    }

    /// Returns the root path configured for the scan.
    #[inline]
    pub fn root(&self) -> PathBuf {
        self.config.root.to_owned()
    }
}

/// Represents a file entry captured during the scan.
#[derive(Debug, Clone)]
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
pub fn is_candidate<E>(entry: E) -> bool
where
    E: Borrow<DirEntry>,
{
    let entry = &entry.borrow() as &DirEntry;

    // Only consider entry if it is a regular file
    if !entry.file_type().map_or(false, |ftype| ftype.is_file()) {
        return false;
    }

    let path = entry.path();
    if path.file_name().is_none() && path.extension().is_none() {
        return false;
    }

    let lookup_name = get_path_suffix(path);
    SourceHeaders::find_header_definition_by_extension(&lookup_name).is_some()
}

#[inline]
pub fn get_path_suffix<P>(path: P) -> String
where
    P: AsRef<Path>,
{
    path.as_ref().extension().map_or_else(
        || {
            path.as_ref()
                .file_name()
                .and_then(|name| name.to_str())
                .map_or(String::new(), |s| s.to_owned())
        },
        |extension| {
            let mut lookup_name = String::with_capacity(extension.len() + 1);
            lookup_name.push('.');
            lookup_name.push_str(extension.to_str().unwrap_or_default());
            lookup_name
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::walker::WalkBuilder;

    #[allow(unused_imports)]
    use rayon::prelude::*;
    use std::env::current_dir;
    use std::fs::File;
    use std::io::Write;

    // Helper function to create temporary directory and files
    fn create_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory")
    }

    #[test]
    fn test_example_scan() {
        let config = ScanConfig {
            exclude: Some(vec!["!**/target/*.py"]), // "!**/*.py", "!**/*.sh"
            limit: 200,
            root: current_dir().unwrap(),
        };

        let exclude = config.exclude.clone().unwrap_or_default();
        let mut walk_builder = WalkBuilder::new(&config.root);
        walk_builder.add_ignore(LICENSA_IGNORE_FILE);
        walk_builder.exclude(Some(exclude)).unwrap();

        let mut walker = walk_builder.build().unwrap();
        walker.quit_while(|res| res.is_err());
        walker.send_while(|res| res.is_ok() && is_candidate(res.unwrap()));
        walker.max_capacity(None);

        let result = walker.run_task();
        let result = result.into_iter();
        let entries: Vec<String> = result
            .par_bridge()
            .into_par_iter()
            .filter_map(|res| res.ok())
            .map(|res| res.file_name().to_str().unwrap().to_owned())
            .collect();

        println!("{:#?}", entries);
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

        // TODO: Implement tests

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
        // TODO implement
    }

    #[test]
    fn test_parallel_file_tree_walker() {}
}
