// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

//! This module provides tools for efficiently walking through a directory tree,
//! filtering entries based on various criteria and providing control over the walk flow.

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use ignore::overrides::OverrideBuilder;
use ignore::{DirEntry, WalkBuilder as InternalWalkBuilder, WalkParallel, WalkState};

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Represents the result of visiting a directory entry during the walk.
///
/// It's either Ok(DirEntry) containing the entry information or
/// Err(ignore::Error) if an error occurred.
pub type WalkResult = Result<DirEntry, ignore::Error>;

/// A closure type that receives a WalkResult and returns a WalkState indicating how
/// the walk should proceed.
pub type FnVisitor<'s> = Box<dyn FnMut(WalkResult) -> WalkState + Send + 's>;

type WalkPredicate = Arc<dyn Fn(WalkResult) -> bool + Send + Sync + 'static>;

/// Represents a workspace walker.
///
/// This type allows configuring and executing walks through a workspace directory tree,
/// filtering entries based on conditions and controlling the walk flow.
pub struct Walk {
    inner: WalkParallel,
    max_capacity: Option<usize>,
    quit_while: WalkPredicate,
    send_while: WalkPredicate,
}

impl Walk {
    pub fn new(inner: WalkParallel, max_capacity: Option<usize>) -> Self {
        Self {
            inner,
            max_capacity,
            quit_while: Arc::new(|_| false),
            send_while: Arc::new(|_| true),
        }
    }

    /// Executes the walk using the provided FnVisitor closure to process each directory entry.
    pub fn run<'a, F>(self, visit: F)
    where
        F: FnMut() -> FnVisitor<'a>,
    {
        self.inner.run(visit)
    }

    /// Starts the walk asynchronously and returns a receiver for collecting [WalkResult]s.
    pub fn run_task(self) -> Receiver<WalkResult> {
        let (tx, rx) = self.chan::<WalkResult>();
        self.inner.run(|| {
            let tx = tx.clone();
            let quit_fn = self.quit_while.clone();
            let send_fn = self.send_while.clone();
            Box::new(move |result| {
                if quit_fn(result.clone()) {
                    return WalkState::Quit;
                }
                if send_fn(result.clone()) {
                    tx.send(result.clone()).unwrap();
                }
                WalkState::Continue
            })
        });

        rx
    }

    /// Sets a condition (closure) for deciding when to send directory entries
    /// to the receiver during the walk.
    #[inline]
    pub fn send_while<T>(&mut self, when: T) -> &Self
    where
        T: Fn(WalkResult) -> bool + Sync + Send + 'static,
    {
        self.send_while = Arc::new(when);
        self
    }

    /// Sets a condition (closure) for stopping the walk early.
    #[inline]
    pub fn quit_while<T>(&mut self, when: T) -> &Self
    where
        T: Fn(WalkResult) -> bool + Sync + Send + 'static,
    {
        self.quit_while = Arc::new(when);
        self
    }

    /// Sets the optional maximum capacity for the receiver in when using `run_task`.
    #[inline]
    pub fn max_capacity(&mut self, limit: Option<usize>) -> &Self {
        if limit.is_none() && self.max_capacity.is_none() {
            return self;
        }
        self.max_capacity = limit;
        self
    }

    #[inline]
    fn chan<T>(&self) -> (Sender<T>, Receiver<T>) {
        match &self.max_capacity {
            None => crossbeam_channel::unbounded::<T>(),
            Some(cap) => crossbeam_channel::bounded::<T>(*cap),
        }
    }
}

/// A builder for configuring and creating a `Walk` instance.
///
/// This type allows setting the workspace root, exclusion/inclusion patterns,
/// and other options to customize the walk behavior.
#[derive(Clone)]
pub struct WalkBuilder {
    /// The root directory of the workspace to be walked.
    workspace_root: PathBuf,

    /// The internal `WalkBuilder` used for managing the walk configuration.
    walker_builder: InternalWalkBuilder,

    /// The `OverrideBuilder` used for handling custom ignore and include patterns.
    override_builder: OverrideBuilder,

    /// The optional maximum capacity of the receiver for walk results.
    max_capacity: Option<usize>,

    exclude: Vec<String>,
    include: Vec<String>,
}

impl WalkBuilder {
    /// Creates a new builder with the workspace root directory.
    pub fn new<P>(workspace_root: P) -> Self
    where
        P: AsRef<Path>,
    {
        let workspace_root = workspace_root.as_ref();
        let walker_builder = InternalWalkBuilder::new(workspace_root);
        let override_builder = OverrideBuilder::new(workspace_root);
        Self {
            walker_builder,
            override_builder,
            workspace_root: workspace_root.into(),
            max_capacity: None,
            exclude: vec![],
            include: vec![],
        }
    }

    /// Builds and returns a Walk instance based on the provided configuration.
    pub fn build(mut self) -> Result<Walk> {
        self.build_overrides()?;
        let walk_parallel = self.walker_builder.build_parallel();
        let walk = Walk::new(walk_parallel, self.max_capacity);
        Ok(walk)
    }

    /// Adds a custom file containing *.gitignore*-like patterns to ignore during the walk.
    #[inline]
    pub fn add_ignore<P>(&mut self, file_name: P) -> &Self
    where
        P: AsRef<OsStr>,
    {
        let file_path = &self.workspace_root().join(file_name.as_ref());
        self.walker_builder.add_custom_ignore_filename(file_path);
        self
    }

    /// Controls whether to use Git ignore rules (default: enabled).
    #[inline]
    pub fn disable_git_ignore(&mut self, yes: bool) -> &Self {
        self.walker_builder.git_ignore(!yes);
        self
    }

    /// Returns a reference to the workspace root directory.
    pub fn workspace_root(&self) -> &Path {
        self.workspace_root.as_ref()
    }

    /// Sets the optional maximum capacity for the receiver in `run_task`.
    pub fn max_capacity(&self) -> Option<usize> {
        self.max_capacity
    }

    /// Adds glob patterns to exclude files and directories.
    pub fn exclude<T>(&mut self, patterns: Option<Vec<T>>) -> Result<()>
    where
        T: 'static + AsRef<str>,
    {
        let patterns = patterns.unwrap_or_default();
        if patterns.is_empty() {
            return Ok(());
        }
        let mut patterns: Vec<String> = patterns
            .iter()
            .map(|p| switch_pattern_negation(p.as_ref()))
            .collect();
        self.exclude.append(&mut patterns);
        Ok(())
    }

    /// Adds glob patterns to include certain files and directories (overrides excludes).
    pub fn include<T>(&mut self, patterns: Option<Vec<T>>) -> Result<()>
    where
        T: 'static + AsRef<str>,
    {
        let patterns = patterns.unwrap_or_default();
        if patterns.is_empty() {
            return Ok(());
        }
        let mut patterns: Vec<String> = patterns.iter().map(|p| p.as_ref().to_string()).collect();
        self.include.append(&mut patterns);
        Ok(())
    }

    // `include` patterns take precedence over exclude patterns.
    // Leave the override builder untouched if both include and exclude patterns are empty.
    fn build_overrides(&mut self) -> Result<()> {
        if self.include.is_empty() && self.exclude.is_empty() {
            return Ok(());
        }
        let patterns = match self.include.is_empty() {
            true => &self.exclude,
            false => &self.include,
        };
        for pattern in patterns {
            self.override_builder.add(pattern)?;
        }
        let overrides = self.override_builder.build()?;
        self.walker_builder.overrides(overrides);

        Ok(())
    }
}

/// Helper function to negate glob patterns (add/remove leading `!`).
///
/// Patterns without a leading `!` are prefixed with one.
/// Patterns with a leading `!` will have that prefix stripped.
///
/// Note:
///
/// This function assumes the pattern is not an empty string, and/or would not become
/// an empty string after removing the leading `!`, if it contains one.
#[inline]
fn switch_pattern_negation(pattern: &str) -> String {
    pattern
        .strip_prefix('!')
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("!{pattern}"))
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use crate::utils::testing::*;
    use ignore::DirEntry;
    use rayon::prelude::*;
    use tempfile::{tempdir, TempDir};

    // Helper function to create a test workspace walk builder
    fn create_test_builder() -> (TempDir, WalkBuilder) {
        let dir = tempdir().unwrap();
        let builder = WalkBuilder::new(&dir);
        (dir, builder)
    }

    // Testing Basic Construction:
    // ============================================================

    #[test]
    fn test_walkbuilder_construction() {
        let root_dir = PathBuf::from("my_workspace");
        let builder = WalkBuilder::new(&root_dir);

        assert_eq!(builder.workspace_root(), &root_dir);
        // assert!(builder.inner_mut().overrides().is_empty());
    }

    #[test]
    fn test_walkincludebuilder_construction() {
        let root_dir = PathBuf::from("my_project");
        let builder = WalkBuilder::new(&root_dir);

        assert_eq!(builder.workspace_root(), &root_dir);
        // assert!(builder.inner_mut().overrides().is_empty());
    }

    #[test]
    fn test_walkexcludebuilder_construction() {
        let root_dir = PathBuf::from("my_app");
        let builder = WalkBuilder::new(&root_dir);

        assert_eq!(builder.workspace_root(), &root_dir);
        // assert!(builder.inner_mut().overrides().is_empty());
    }

    // Testing Builder Modifiers:
    // ============================================================
    #[test]
    fn test_walkbuilder_disable_git_ignore() {
        let mut builder = WalkBuilder::new("my_dir");
        builder.disable_git_ignore(true);

        // assert_eq!(builder.inner_mut().git_ignore_enabled(), false);
    }

    #[test]
    fn test_walk_builder_add_ignore_file() {
        let mut builder = WalkBuilder::new("my_codebase");
        builder.add_ignore(".gitignore");

        let expected_path = builder.workspace_root().join(".gitignore");
        // assert_eq!(builder.inner_mut().custom_ignore_files(), &[expected_path]);
    }

    #[test]
    fn test_walk_include_builder_add_overrides() {
        let mut builder = WalkBuilder::new("my_repo");
        builder
            .include(Some(vec!["src/**/*.rs", "tests/**/*.rs"]))
            .unwrap();

        let expected_patterns = ["src/**/*.rs", "tests/**/*.rs"];
        // assert_eq!(
        //     builder.inner_mut().overrides().patterns(),
        //     &expected_patterns
        // );
    }

    #[test]
    fn test_walk_exclude_builder_add_overrides() {
        let mut builder = WalkBuilder::new("my_lib");
        builder
            .exclude(Some(vec!["vendor/**", ".target/**"]))
            .unwrap();

        let expected_patterns = ["vendor/**", ".target/**"];
        // let overrides = builder.override_builder().build().unwrap();
        let overrides_res = builder.build_overrides();
        assert!(overrides_res.is_ok());

        // assert_eq!(builder.inner_mut().o.patterns(), &expected_patterns);
    }

    // Testing Builder Output:
    // =====================================================================

    #[test]
    fn test_walk_builder_build() {
        let builder = WalkBuilder::new("my_workspace");
        let walk = builder.build();

        assert!(walk.is_ok());
        // Add more assertions based on walk properties and functionality
    }

    #[test]
    fn test_walk_include_builder_build() {
        let mut builder = WalkBuilder::new("my_root");
        builder.include(Some(vec!["src/**/*.rs"])).unwrap();
        let walk = builder.build();

        assert!(walk.is_ok());
        // Add more assertions based on walk behavior with include patterns
    }

    #[test]
    fn test_walk_exclude_builder_build() {
        let mut builder = WalkBuilder::new("my_project");
        builder.exclude(Some(vec!["vendor/**"])).unwrap();
        let walk = builder.build();

        assert!(walk.is_ok());
        // Add more assertions based on walk behavior with exclude patterns
    }

    // Others
    // ===================================================================

    #[test]
    fn test_workspace_walk_run_task() {
        // Arrange
        let (tmp_dir, mut builder) = create_test_builder();
        builder.add_ignore(".git");
        let walker = builder.build().expect("Failed to build workspace walk");

        // Act
        let rx = walker.run_task();

        // Assert
        // Add assertions for receiving results from the workspace walk
    }

    #[test]
    fn test_workspace_walk_send_while() {
        // Arrange
        let (tmp_dir, file_path) = create_temp_file("somefile.rs");
        let builder = WalkBuilder::new(&tmp_dir);

        let mut walker = builder
            .clone()
            .build()
            .expect("Failed to build workspace walk");

        let filter_file = |res: Result<DirEntry, ignore::Error>| {
            res.is_ok() && res.unwrap().file_type().unwrap().is_file()
        };
        // Only include files
        walker.send_while(filter_file);

        // Act
        let entries: Vec<DirEntry> = walker
            .run_task()
            .into_iter()
            .par_bridge()
            .into_par_iter()
            .filter_map(Result::ok)
            .collect();
        assert!(entries.len() == 1);

        drop(file_path);
        tmp_dir.close().unwrap();

        let tmp_dir = tempdir().unwrap();
        let tmp_dir = tmp_dir.path();
        let tmp_file_1 = tmp_dir.join("anotherfile.rs");
        let tmp_file_2 = tmp_dir.join("yetanotherfile.rs");
        File::create(tmp_file_1).unwrap();
        File::create(tmp_file_2).unwrap();

        let builder = WalkBuilder::new(tmp_dir);
        let mut walker = builder.build().expect("Failed to build workspace walk");
        walker.send_while(filter_file);
        let entries: Vec<DirEntry> = walker
            .run_task()
            .into_iter()
            .par_bridge()
            .into_par_iter()
            .filter_map(Result::ok)
            .collect();

        assert!(entries.len() == 2);
        // Assert
        // Add assertions to verify that the send_while condition is applied
    }

    #[test]
    fn test_workspace_walk_quit_while() {
        // Arrange
        let (tmp_dir, builder) = create_test_builder();
        let mut walker = builder.build().expect("Failed to build workspace walk");
        walker.quit_while(|_result| true);

        // Act
        let rx = walker.run_task();

        // Assert
        // Add assertions to verify that the quit_while condition is applied
    }

    #[test]
    fn test_workspace_walk_with_invalid_ignore() {
        // Arrange
        let (tmp_dir, mut builder) = create_test_builder();
        builder.add_ignore("nonexistent_ignore_file");

        // Act
        let result = builder.build();
        assert!(result.is_ok());

        // Assert
        // Add more assertions as needed
    }

    #[test]
    fn test_workspace_walk_with_disable_git_ignore() {
        // Arrange
        let (tmp_dir, mut builder) = create_test_builder();
        builder.disable_git_ignore(true);
        let walker = builder.build().expect("Failed to build workspace walk");

        // Act
        let rx = walker.run_task();

        // Assert
        // Add assertions for receiving results from the workspace walk with git ignore disabled
    }

    // Add more tests for other methods and scenarios as needed
}
