// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpdxError {
    #[error("SPDX license ID \"{0}\" not found")]
    NotFound(String),

    #[error("Failed to read licenses metadata file")]
    MetadataFileNotFound,

    #[error(transparent)]
    HttpError(#[from] reqwest::Error),

    #[error(transparent)]
    DataError(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] io::Error),
}

// impl Error for SpdxError {}

pub fn to_clap_error(err: impl ToString) -> String {
    let val = err.to_string();
    println!("{val}");
    val
}
