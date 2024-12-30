// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::Cli;
use clap::CommandFactory;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // --- Custom
    #[error("failed to serialize argument '{arg}'.\nReason: {reason}")]
    ArgumentSerializationFailed { arg: &'static str, reason: String },

    #[error("failed to deserialize argument '{arg}'. \nReason: {reason}")]
    ArgumentDeserializationFailed { arg: &'static str, reason: String },

    #[error("missing required argument '{0}'")]
    MissingRequiredArgument(&'static str),

    /// Error thrown when working with licensa workspaces.
    #[error(transparent)]
    Workspace(#[from] crate::workspace::Error),

    /// Error thrown throughout the licensing process.
    #[error(transparent)]
    License(#[from] crate::license::Error),

    /// Error thrown when printing to the console.
    #[error(transparent)]
    Terminal(#[from] crate::terminal::Error),

    // --- Core
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Prints the error and exits.
    pub fn exit(&self) -> ! {
        match self {
            Error::MissingRequiredArgument(err) => Cli::command()
                .error(clap::error::ErrorKind::MissingRequiredArgument, err)
                .exit(),

            Error::ArgumentDeserializationFailed { .. }
            | Error::ArgumentSerializationFailed { .. } => Cli::command()
                .error(clap::error::ErrorKind::ValueValidation, self.to_string())
                .exit(),
            Error::Io(err) => Cli::command().error(clap::error::ErrorKind::Io, err).exit(),
            Error::Json(err) => Cli::command()
                .error(clap::error::ErrorKind::ValueValidation, err)
                .exit(),
            Error::License(err) => match err {
                _ => Cli::command()
                    .error(clap::error::ErrorKind::ValueValidation, err)
                    .exit(),
            },
            Error::Terminal(err) => match err {
                _ => Cli::command()
                    .error(clap::error::ErrorKind::Format, err)
                    .exit(),
            },
            Error::Workspace(err) => match err {
                crate::workspace::Error::Data(_) => Cli::command()
                    .error(clap::error::ErrorKind::ValueValidation, err)
                    .exit(),
                _ => Cli::command().error(clap::error::ErrorKind::Io, err).exit(),
            },
        }
    }
}
