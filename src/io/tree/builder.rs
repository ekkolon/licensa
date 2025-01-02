#![allow(unused)]

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use ignore::{overrides::OverrideBuilder, WalkBuilder, WalkState};

use super::{DocumentRef, DocumentSnapshot};
use crate::Result;

pub struct TreeBuilder {
    src_root: PathBuf,
    max_capacity: usize,
    dry_run: bool,
    exclude: Vec<String>,
    include: Vec<String>,
    /// The internal `WalkBuilder` used for managing the walk configuration.
    walker_builder: WalkBuilder,
    /// The `OverrideBuilder` used for handling custom ignore and include patterns.
    override_builder: OverrideBuilder,
}

const MAX_CAPACITY: usize = 1000;

use ignore::DirEntry;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use serde::Serialize;

use super::TreeSnapshot;

type WalkerResult = std::result::Result<DirEntry, ignore::Error>;

pub struct Tree {
    src_root: PathBuf,
    dry_run: bool,
    walk_builder: WalkBuilder,
    max_capacity: usize,
}

impl Tree {
    // TODO: Implement src_root usage
    pub fn src_root(&self) -> &Path {
        &self.src_root
    }

    pub fn update_license_info<T: Serialize + Clone + Sync + Send>(
        &self,
        data: T,
    ) -> Result<TreeSnapshot> {
        let (unlicensed, modified, licensed, failed): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) = self
            .find_license_candidates()
            .into_par_iter()
            .map(|doc_ref| {
                let doc = match self.dry_run {
                    true => doc_ref.read_dry_run(),
                    false => doc_ref.read(),
                }?;

                let doc = doc.add_license(data.clone())?;
                Ok(doc)
            })
            .filter_map(|snapshot: Result<DocumentSnapshot>| snapshot.ok())
            .fold(
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |mut acc, snap| {
                    match snap {
                        DocumentSnapshot::Untouched { .. } => acc.0.push(snap),
                        DocumentSnapshot::Modified { .. } => acc.1.push(snap),
                        DocumentSnapshot::Licensed { .. } => acc.2.push(snap),
                        DocumentSnapshot::Failed { .. } => acc.3.push(snap),
                    }
                    acc
                },
            )
            .reduce(
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |mut acc1, acc2| {
                    acc1.0.extend(acc2.0);
                    acc1.1.extend(acc2.1);
                    acc1.2.extend(acc2.2);
                    acc1.3.extend(acc2.3);
                    acc1
                },
            );

        Ok(TreeSnapshot {
            failed,
            licensed,
            unlicensed,
            modified,
            src_root: self.src_root.clone(),
        })
    }

    pub fn read_license_info(&self) -> Result<TreeSnapshot> {
        let (untouched, modified, licensed, failed): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) = self
            .find_license_candidates()
            .into_par_iter()
            .map(|doc_ref| doc_ref.read_to_snapshot())
            .filter_map(|doc_ref| doc_ref.ok())
            .fold(
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |mut acc, snap| {
                    match snap {
                        DocumentSnapshot::Untouched { .. } => acc.0.push(snap),
                        DocumentSnapshot::Modified { .. } => acc.1.push(snap),
                        DocumentSnapshot::Licensed { .. } => acc.2.push(snap),
                        DocumentSnapshot::Failed { .. } => acc.3.push(snap),
                    }
                    acc
                },
            )
            .reduce(
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |mut acc1, acc2| {
                    acc1.0.extend(acc2.0);
                    acc1.1.extend(acc2.1);
                    acc1.2.extend(acc2.2);
                    acc1.3.extend(acc2.3);
                    acc1
                },
            );

        Ok(TreeSnapshot {
            failed,
            licensed,
            unlicensed: untouched,
            modified,
            src_root: self.src_root.clone(),
        })
    }

    pub fn find_license_candidates(&self) -> Vec<DocumentRef> {
        let (tx, rx) = crossbeam_channel::bounded::<Result<DocumentRef>>(self.max_capacity);

        let walker = self.walk_builder.build_parallel();
        walker.run(|| {
            let tx = tx.clone();
            Box::new(move |result: WalkerResult| {
                let result = result.map(DocumentRef::new).map_err(|err| err.into());

                match result {
                    Err(err) => tx.send(Err(err)).unwrap(),
                    Ok(doc_ref) => {
                        if !doc_ref.is_licensable_file() {
                            return WalkState::Continue;
                        }

                        tx.send(Ok(doc_ref)).unwrap()
                    }
                };

                WalkState::Continue
            })
        });

        drop(tx);

        let entries: Vec<DocumentRef> = rx
            .into_iter()
            .par_bridge()
            .into_par_iter()
            .filter_map(|e| e.ok())
            .collect();

        entries
    }
}

impl TreeBuilder {
    /// Creates a new builder with the workspace root directory.
    pub fn new<P>(src_root: P) -> Self
    where
        P: AsRef<Path>,
    {
        let src_root = src_root.as_ref();
        let walker_builder = WalkBuilder::new(src_root);
        let override_builder = OverrideBuilder::new(src_root);
        Self {
            walker_builder,
            override_builder,
            src_root: src_root.into(),
            max_capacity: MAX_CAPACITY,
            dry_run: false,
            exclude: vec![],
            include: vec![],
        }
    }

    /// Builds and returns a Walk instance based on the provided configuration.
    pub fn build(mut self) -> Result<Tree> {
        self.build_overrides()?;
        Ok(Tree {
            dry_run: self.dry_run,
            src_root: self.src_root,
            max_capacity: self.max_capacity,
            walk_builder: self.walker_builder,
        })
    }

    /// Sets the optional maximum capacity for the receiver in `run_task`.
    pub fn src_root(&self) -> &Path {
        &self.src_root
    }

    /// Sets the optional maximum capacity for the receiver in `run_task`.
    pub fn set_dry_run(mut self, yes: bool) -> Self {
        self.dry_run = yes;
        self
    }

    /// Sets the optional maximum capacity for the receiver in `run_task`.
    pub fn set_max_capacity(mut self, max_capacity: usize) -> Self {
        self.max_capacity = max_capacity;
        self
    }

    /// Sets the optional maximum capacity for the receiver in `run_task`.
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }

    /// Adds glob patterns to exclude files and directories.
    pub fn exclude<T>(mut self, patterns: Vec<T>) -> Result<Self>
    where
        T: 'static + AsRef<str>,
    {
        if patterns.is_empty() {
            return Ok(self);
        }
        let mut patterns: Vec<String> = patterns
            .iter()
            .map(|p| switch_pattern_negation(p.as_ref()))
            .collect();

        self.exclude.append(&mut patterns);
        Ok(self)
    }

    /// Adds glob patterns to exclude files and directories.
    pub fn set_exclude<T>(&mut self, patterns: Vec<T>) -> Result<()>
    where
        T: 'static + AsRef<str>,
    {
        self.exclude = patterns
            .iter()
            .map(|p| switch_pattern_negation(p.as_ref()))
            .collect();

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

    /// Adds a custom file containing *.gitignore*-like patterns to ignore during the walk.
    #[inline]
    pub fn add_ignore<P>(&mut self, file_name: P) -> &Self
    where
        P: AsRef<OsStr>,
    {
        let file_path = &self.src_root().join(file_name.as_ref());
        self.walker_builder.add_custom_ignore_filename(file_path);
        self
    }

    /// Controls whether to use Git ignore rules (default: enabled).
    #[inline]
    pub fn disable_git_ignore(&mut self, yes: bool) -> &Self {
        self.walker_builder.git_ignore(!yes);
        self
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
    use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
    use tempfile::{tempdir, TempDir};

    // Helper function to create a test workspace walk builder
    fn create_test_builder() -> (TempDir, TreeBuilder) {
        let dir = tempdir().unwrap();
        let builder = TreeBuilder::new(&dir);
        (dir, builder)
    }

    // Testing Basic Construction:
    // ============================================================

    #[test]
    fn test_walkbuilder_construction() {
        let root_dir = PathBuf::from("my_workspace");
        let builder = TreeBuilder::new(&root_dir);

        assert_eq!(builder.src_root(), &root_dir);
        // assert!(builder.inner_mut().overrides().is_empty());
    }

    #[test]
    fn test_walkincludebuilder_construction() {
        let root_dir = PathBuf::from("my_project");
        let builder = TreeBuilder::new(&root_dir);

        assert_eq!(builder.src_root(), &root_dir);
        // assert!(builder.inner_mut().overrides().is_empty());
    }

    #[test]
    fn test_walkexcludebuilder_construction() {
        let root_dir = PathBuf::from("my_app");
        let builder = TreeBuilder::new(&root_dir);

        assert_eq!(builder.src_root(), &root_dir);
        // assert!(builder.inner_mut().overrides().is_empty());
    }

    // Testing Builder Modifiers:
    // ============================================================
    #[test]
    fn test_walkbuilder_disable_git_ignore() {
        let mut builder = TreeBuilder::new("my_dir");
        builder.disable_git_ignore(true);

        // assert_eq!(builder.inner_mut().git_ignore_enabled(), false);
    }

    // Testing Builder Output:
    // =====================================================================

    #[test]
    fn test_walk_builder_build() {
        let builder = TreeBuilder::new("my_workspace");
        let walk = builder.build();

        assert!(walk.is_ok());
        // Add more assertions based on walk properties and functionality
    }

    #[test]
    fn test_walk_include_builder_build() {
        let mut builder = TreeBuilder::new("my_root");
        builder.include(Some(vec!["src/**/*.rs"])).unwrap();
        let walk = builder.build();

        assert!(walk.is_ok());
        // Add more assertions based on walk behavior with include patterns
    }

    #[test]
    fn test_walk_exclude_builder_build() {
        let mut tree = TreeBuilder::new("my_project")
            .exclude(vec!["vendor/**"])
            .unwrap()
            .build();

        assert!(tree.is_ok());
        // Add more assertions based on walk behavior with exclude patterns
    }

    // Others
    // ===================================================================

    #[test]
    fn test_workspace_walk_with_invalid_ignore() {
        let (_, mut builder) = create_test_builder();
        builder.add_ignore("nonexistent_ignore_file");

        let result = builder.build();
        assert!(result.is_ok());
    }
}
