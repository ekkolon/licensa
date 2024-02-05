// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code, unused_variables)]

pub mod cli;
pub mod commands;
pub mod config;
pub mod template;
pub mod workspace;

mod env;
mod error;
mod license;
mod ops;
mod parser;
mod schema;
mod spdx;
mod utils;
