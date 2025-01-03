// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod cli;
pub mod commands;
pub mod license;
pub mod workspace;
pub mod config;

mod error;
mod io;
mod utils;

pub use error::*;
mod console;
mod macros;
