// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

pub mod error;
pub mod license;
mod template;

pub use template::*;

use self::error::SpdxError;

pub type Result<T> = anyhow::Result<T, SpdxError>;
