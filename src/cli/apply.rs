// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use crate::utils;
use crate::validator;

use clap::Args;

#[derive(Args, Debug)]
pub struct ApplyArgs {
  /// License type as SPDX id.
  #[arg(short, long)]
  pub license: String,

  /// The copyright owner.
  #[arg(short, long)]
  pub author: String,

  /// The copyright year.
  #[arg(short, long, value_parser = validator::acceptable_year)]
  #[arg(default_value_t = utils::current_year())]
  pub year: u16,
}
