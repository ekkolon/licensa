// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{cli::Exit, exit_with_error, io::TreeSnapshot};
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

    // --- Core
    #[error(transparent)]
    Inquire(#[from] inquire::InquireError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    TreeInfoPoisoned(#[from] std::sync::PoisonError<TreeSnapshot>),

    #[error(transparent)]
    Ignore(#[from] ignore::Error),

    #[error(transparent)]
    TemplateRenderer(#[from] handlebars::RenderError),
}

impl Exit for Error {
    /// Prints the error and exits.
    fn exit(self) -> ! {
        match self {
            Error::MissingRequiredArgument(err) => {
                exit_with_error!(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    err.to_string()
                )
            }
            Error::ArgumentDeserializationFailed { reason, .. }
            | Error::ArgumentSerializationFailed { reason, .. } => {
                exit_with_error!(clap::error::ErrorKind::ValueValidation, reason)
            }
            Error::TreeInfoPoisoned(err) => {
                exit_with_error!(clap::error::ErrorKind::Io, err.to_string())
            }
            Error::Io(err) => exit_with_error!(clap::error::ErrorKind::Io, err.to_string()),
            Error::Ignore(err) => exit_with_error!(clap::error::ErrorKind::Io, err.to_string()),
            Error::TemplateRenderer(err) => {
                exit_with_error!(clap::error::ErrorKind::Io, err.to_string())
            }
            Error::Inquire(err) => {
                exit_with_error!(clap::error::ErrorKind::ValueValidation, err.to_string())
            }
            Error::Json(err) => {
                exit_with_error!(clap::error::ErrorKind::ValueValidation, err.to_string())
            }
            Error::License(err) => {
                exit_with_error!(clap::error::ErrorKind::ValueValidation, err.to_string())
            }

            Error::Workspace(err) => match err {
                crate::workspace::Error::Data(_) => {
                    exit_with_error!(clap::error::ErrorKind::ValueValidation, err.to_string())
                }
                _ => exit_with_error!(clap::error::ErrorKind::Io, err.to_string()),
            },
        }
    }
}
