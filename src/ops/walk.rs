//! # Walk
//!
//! This module provides functionality for walking through workspaces,
//! interacting with directory entries, and performing various tasks
//! based on the results received from a `WorkspaceWalk` task execution.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use ignore::{overrides::OverrideBuilder, DirEntry, WalkBuilder, WalkParallel, WalkState};

/// A type representing a result of walking through a workspace.
pub type WorkspaceWalkResult = Result<DirEntry, ignore::Error>;

/// A function type representing a visitor for walking through a workspace.
pub type FnVisitor<'s> = Box<dyn FnMut(WorkspaceWalkResult) -> WalkState + Send + 's>;

type WorkspaceWalkPredicate = Box<dyn FnMut(WorkspaceWalkResult) -> bool + Send + 'static>;

/// Represents a workspace walker.
pub struct WorkspaceWalk {
    inner: WalkParallel,
    max_capacity: Option<usize>,
    quit_while: Option<WorkspaceWalkPredicate>,
    send_while: Option<WorkspaceWalkPredicate>,
}

impl WorkspaceWalk {
    /// Runs the workspace walk with the provided visitor function.
    pub fn run<'a, F>(self, visit: F)
    where
        F: FnMut() -> FnVisitor<'a>,
    {
        self.inner.run(visit)
    }

    /// Runs the workspace walk and returns a receiver for collecting results.
    pub fn run_task(self) -> Receiver<WorkspaceWalkResult> {
        let (tx, rx) = self.chan::<WorkspaceWalkResult>();
        let quit_fn = self.quit_while.unwrap_or_else(|| Box::new(|_| false));
        let quit_fn_arc = Arc::new(Mutex::new(quit_fn));
        let send_fn = self.send_while.unwrap_or_else(|| Box::new(|_| false));
        let send_fn_arc = Arc::new(Mutex::new(send_fn));
        self.inner.run(|| {
            let tx = tx.clone();
            let quit_fn = quit_fn_arc.clone();
            let send_fn = send_fn_arc.clone();
            Box::new(move |result| {
                let mut quit_fn = quit_fn.lock().unwrap();
                let mut send_fn = send_fn.lock().unwrap();
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

    /// Sets a condition for sending results while walking the workspace.
    pub fn send_while<T>(&mut self, when: T) -> &Self
    where
        T: FnMut(WorkspaceWalkResult) -> bool + Send + 'static,
    {
        let _ = self.send_while.insert(Box::new(when));
        self
    }

    /// Sets a condition for quitting the workspace walk.
    pub fn quit_while<T>(&mut self, when: T) -> &Self
    where
        T: FnMut(WorkspaceWalkResult) -> bool + Send + 'static,
    {
        let _ = self.quit_while.insert(Box::new(when));
        self
    }

    /// Sets the maximum capacity of the channel for collecting results.
    pub fn max_capacity(&mut self, limit: Option<usize>) -> &Self {
        if limit.is_none() && self.max_capacity.is_none() {
            return self;
        }
        self.max_capacity = limit;
        self
    }

    fn chan<T>(&self) -> (Sender<T>, Receiver<T>) {
        match &self.max_capacity {
            None => crossbeam_channel::unbounded::<T>(),
            Some(cap) => crossbeam_channel::bounded::<T>(*cap),
        }
    }
}

/// A builder for configuring and constructing a `WorkspaceWalk`.
#[derive(Clone)]
pub struct WorkspaceWalkBuilder {
    workspace_root: PathBuf,
    walker_builder: WalkBuilder,
    override_builder: OverrideBuilder,
    max_capacity: Option<usize>,
}

impl WorkspaceWalkBuilder {
    /// Creates a new `WorkspaceWalkBuilder` with the provided workspace root.
    pub fn new<P>(workspace_root: P) -> Self
    where
        P: AsRef<Path>,
    {
        let workspace_root = workspace_root.as_ref();
        let walker_builder = WalkBuilder::new(workspace_root);
        let override_builder = OverrideBuilder::new(workspace_root);
        Self {
            walker_builder,
            override_builder,
            workspace_root: workspace_root.into(),
            max_capacity: None,
        }
    }

    /// Builds and returns a `WorkspaceWalk`.
    pub fn build(mut self) -> Result<WorkspaceWalk> {
        let overrides = self.override_builder.build()?;
        if !overrides.is_empty() {
            self.walker_builder.overrides(overrides);
        }
        let walk_parallel = self.walker_builder.build_parallel();
        Ok(WorkspaceWalk {
            inner: walk_parallel,
            max_capacity: self.max_capacity,
            quit_while: None,
            send_while: None,
        })
    }

    /// Sets whether to disable git ignore rules.
    pub fn disable_git_ignore(&mut self, yes: bool) -> &Self {
        self.walker_builder.git_ignore(!yes);
        self
    }

    /// Adds a custom ignore file.
    pub fn add_ignore<P>(&mut self, file_name: P) -> &Self
    where
        P: AsRef<OsStr>,
    {
        let file_path = &self.workspace_root.join(file_name.as_ref());
        self.walker_builder.add_custom_ignore_filename(file_path);
        self
    }

    /// Adds a set of glob patterns to the overrides.
    pub fn add_overrides(&mut self, patterns: &Vec<&'static str>) -> Result<()> {
        if !patterns.is_empty() {
            for pattern in patterns {
                if let Some(pattern) = Self::switch_pattern_negation(pattern) {
                    self.override_builder.add(&pattern)?;
                }
            }
        }
        Ok(())
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
    fn switch_pattern_negation(pattern: &str) -> Option<String> {
        if pattern.starts_with('!') {
            return pattern.strip_prefix('!').map(|p| p.to_string());
        }
        Some(format!("!{pattern}"))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use crate::utils::testing::*;
    use rayon::prelude::*;
    use tempfile::{tempdir, TempDir};

    // Helper function to create a test workspace walk builder
    fn create_test_builder() -> (TempDir, WorkspaceWalkBuilder) {
        let dir = tempdir().unwrap();
        let builder = WorkspaceWalkBuilder::new(&dir);
        (dir, builder)
    }

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
        let builder = WorkspaceWalkBuilder::new(&tmp_dir);

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

        let builder = WorkspaceWalkBuilder::new(tmp_dir);
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
