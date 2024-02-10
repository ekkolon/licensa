// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Contains custom error types used throughout the Licensa workspace functionality.
//!
//! This module defines various errors that can occur during operations like reading
//! configuration files, writing ignore files, handling data serialization, and working
//! with file paths. Each error type provides a meaningful message and error details
//! for easier debugging and troubleshooting.
//!
//! # Related Modules
//!
//! - `workspace::ops` uses these error types for error handling.

use thiserror::Error;

use std::path::PathBuf;

/// Represents various errors that can occur during Licensa workspace operations.
///
/// This enum defines types for different error scenarios encountered when handling
/// configuration files, ignore files, data serialization, file paths, and general
/// unexpected issues.
#[derive(Error, Debug)]
pub enum WorkspaceError {
    /// Error indicating the required `.licensarc` configuration file is missing.
    ///
    /// This error occurs when attempting to read or access the configuration file,
    /// but it's not present in the expected location within the workspace.
    #[error("workspace is missing a .licensarc config file")]
    MissingConfigFile,

    /// Error indicating the `.licensaignore` file is missing from the workspace.
    ///
    /// This error occurs when attempting to read or use the ignore file for
    /// configuration, but it's not present in the expected location within the
    /// workspace.
    #[error("workspace is missing a .licensaignore file")]
    MissingIgnoreFile,

    /// Error indicating Licensa configuration already exists for the given path.
    ///
    /// This error occurs when attempting to configure Licensa in a workspace
    /// that already has a configuration file.
    #[error("licensa is already configured for {0}")]
    AlreadyConfigured(PathBuf),

    /// Error indicating a `.licensarc` configuration file already exists.
    ///
    /// This error occurs when attempting to create a new `.licensarc` file,
    /// but one already exists in the specified location.
    #[error(".licensarc config file already exists in {0}")]
    ConfigFileAlreadyExists(PathBuf),

    /// Error indicating a `.licensaignore` file already exists.
    ///
    /// This error occurs when attempting to create a new `.licensaignore` file,
    /// but one already exists in the specified location.
    #[error(".licensaignore file already exists in {0}")]
    IgnoreFileAlreadyExists(PathBuf),

    /// Error indicating malformed config data.
    ///
    /// This error occurs when attempting to save workspace configuration
    /// that is not a valid JSON object.
    #[error("invalid config data type. Provided value must be an object")]
    InvalidConfigDataType,

    /// Transparent error wrapper for serialization/deserialization issues.
    #[error(transparent)]
    Data(#[from] serde_json::error::Error),

    /// Transparent error wrapper for file I/O operations.
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// Error indicating a provided path is not a directory.
    ///
    /// This error occurs when expecting a directory path (e.g., for a workspace
    /// root), but the provided path points to a file or non-existent location.
    #[error("path {0} is not a directory")]
    NotADirectory(PathBuf),

    /// Other unexpected errors.
    ///
    /// This variant catches any other unforeseen errors not covered by the above
    /// specific cases.
    #[error(transparent)]
    Generic(#[from] anyhow::Error),
}

/// A type alias for `anyhow::Result<T, WorkspaceError>`.
pub type WorkspaceResult<T> = anyhow::Result<T, WorkspaceError>;
