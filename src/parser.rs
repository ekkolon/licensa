// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::str::FromStr;

use anyhow::Result;

use crate::schema::LicenseId;

pub fn parse_license_id(input: &str) -> Result<LicenseId> {
    // We trim leading and trailing `"` in case an user provides a single license ID
    // as `--type "MIT"`, whereas it should be provided as `--type MIT`.
    let typ = input.trim_matches('"');
    LicenseId::from_str(input)
}
