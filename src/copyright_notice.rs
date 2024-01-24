use crate::utils::current_year;
use serde::{Deserialize, Serialize};

/// Represents a compact template for a copyright notice.
pub const COMPACT_COPYRIGHT_NOTICE: &str = r#"Copyright $(year) $(fullname)

Use of this source code is governed by an $(license)-style license that can be
found in the LICENSE file $(determiner) $(location)."#;

/// Holds information for a copyright notice.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CompactCopyrightNotice {
  /// The full name of the copyright holder.
  pub fullname: String,

  /// The type of license governing the use of the source code.
  pub license: String,

  /// The year(s) to be included in the copyright notice.
  #[serde(default = "current_year")]
  pub year: u16,

  /// A word indicating where to find the LICENSE file (e.g., "in").
  #[serde(default = "default_determiner")]
  pub determiner: String,

  /// The location where the LICENSE file can be found.
  #[serde(default = "default_location")]
  pub location: String,
}

fn default_determiner() -> String {
  "in".to_string()
}

fn default_location() -> String {
  "the root of this project".to_string()
}
