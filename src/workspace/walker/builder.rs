// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Walk
//!
//! This module provides functionality for walking through workspaces,
//! interacting with directory entries, and performing various tasks
//! based on the results received from a `WorkspaceWalk` task execution.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::Result;
use ignore::{overrides::OverrideBuilder, WalkBuilder as InternalWalkBuilder};

use super::walk::Walk;

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

    /// Builds and returns a `WorkspaceWalk`.
    pub fn build(mut self) -> Result<Walk> {
        self.build_overrides()?;
        let walk_parallel = self.walker_builder.build_parallel();
        let walk = Walk::new(walk_parallel, self.max_capacity);
        Ok(walk)
    }

    /// Adds a custom ignore file.
    #[inline]
    pub fn add_ignore<P>(&mut self, file_name: P) -> &Self
    where
        P: AsRef<OsStr>,
    {
        let file_path = &self.workspace_root().join(file_name.as_ref());
        self.inner_mut().add_custom_ignore_filename(file_path);
        self
    }

    /// Sets whether to disable git ignore rules.
    ///
    /// This method only effects walk runs without `include` or `exclude` patterns.
    /// We expose this method because sometimes you might want to explicitly turn off
    /// gitignore pattern matches.
    #[inline]
    pub fn disable_git_ignore(&mut self, yes: bool) -> &Self {
        self.inner_mut().git_ignore(!yes);
        self
    }

    /// Returns a reference to the root directory of the workspace.
    pub fn workspace_root(&self) -> &Path {
        self.workspace_root.as_ref()
    }

    /// Returns a reference to the internal `WalkBuilder` used for configuring the walk.
    pub fn inner(&self) -> &InternalWalkBuilder {
        &self.walker_builder
    }

    /// Returns a mutable reference to the internal `WalkBuilder`.
    pub fn inner_mut(&mut self) -> &mut InternalWalkBuilder {
        &mut self.walker_builder
    }

    /// Returns a reference to the `OverrideBuilder` used for managing custom include/exclude patterns.
    pub fn override_builder(&self) -> &OverrideBuilder {
        &self.override_builder
    }

    /// Returns the optional maximum capacity for the receiver of walk results, if set.
    pub fn max_capacity(&self) -> Option<usize> {
        self.max_capacity
    }

    /// Adds a set of glob *exclude* patterns to the overrides.
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

    /// Adds a set of glob *include* patterns to the overrides.
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
        let overrides = self.override_builder().build()?;
        self.walker_builder.overrides(overrides);

        Ok(())
    }
}

/// Switch pattern negation.
///
/// Pattern without `!` are prefixed with one. Similarly, pattern starting
/// with `!` will have that removed.
///
/// Note: This function assumes the pattern is not an empty string,
/// and/or would not become an empty string after stripping `!`
/// if it contains one.
#[inline]
fn switch_pattern_negation(pattern: &str) -> String {
    pattern
        .strip_prefix('!')
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("!{pattern}"))
}

#[derive(Clone, PartialEq)]
pub enum OverridePatterns {
    Include(Vec<String>),
    Exclude(Vec<String>),
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
