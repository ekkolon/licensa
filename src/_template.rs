// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use std::{fs, io::Error};

const SPDX_TEMPLATE: &str = "Copyright [year] [author]
SPDX-License-Identifier: [spdx_id]";

// const TEMPLATE_MAP: Map<String, String> = {};

// Contains the data used to fill out a license template.
pub struct LicenseData {
  pub year: String,
  pub holder: String,
  pub spdx_id: String,
}

/// Return the license template for the specified SPDX license id
/// or from the the contents of PathBuf specified *file* argument.  
pub fn fetch_template(
  _license_id: String,
  license_file: Option<String>,
  spdx_only: Option<bool>,
) -> Result<String, Error> {
  // License template content
  let template = String::new();

  let is_spdx_only = spdx_only.unwrap_or(false);
  if is_spdx_only {
    return Ok(SPDX_TEMPLATE.to_string());
  };

  if let Some(license_file) = license_file {
    return fs::read_to_string(license_file);
  }

  Ok(template)
}

fn read_license_template() {}

#[cfg(test)]
#[path = "./template_test.rs"]
mod test;
