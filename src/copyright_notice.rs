// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use crate::interpolation::{interpolate, Interpolate};
use crate::utils::current_year;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Represents a simple SPDX copyright notice.
pub const SPDX_COPYRIGHT_NOTICE: &str = r#"Copyright $(year) $(fullname)
SPDX-License-Identifier: $(license)"#;

/// Represents a compact template for a copyright notice.
pub const COMPACT_COPYRIGHT_NOTICE: &str = r#"Copyright $(year) $(fullname)

Use of this source code is governed by an $(license)-style license that can be
found in the LICENSE file $(determiner) $(location)."#;

/// Holds information for a simple SPDX copyright notice.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SpdxCopyrightNotice {
    /// The full name of the copyright holder.
    pub fullname: String,

    /// The type of license governing the use of the source code.
    pub license: String,

    /// The year(s) to be included in the copyright notice.
    #[serde(default = "current_year")]
    pub year: u16,
}

impl Interpolate for SpdxCopyrightNotice {
    fn interpolate(&self) -> Result<String> {
        interpolate!(SPDX_COPYRIGHT_NOTICE, &self)
    }
}

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
    #[serde(default = "CompactCopyrightNotice::default_determiner")]
    pub determiner: String,

    /// The location where the LICENSE file can be found.
    #[serde(default = "CompactCopyrightNotice::default_location")]
    pub location: String,
}

impl CompactCopyrightNotice {
    fn default_determiner() -> String {
        "in".to_string()
    }

    fn default_location() -> String {
        "the root of this project".to_string()
    }
}

impl Interpolate for CompactCopyrightNotice {
    fn interpolate(&self) -> Result<String> {
        interpolate!(COMPACT_COPYRIGHT_NOTICE, &self)
    }
}

pub fn contains_copyright_notice<F: AsRef<str>>(file_content: F) -> bool {
    let spdx_notice = r"Copyright(?: \d{4})? .+[\n\r]?.*SPDX-License-Identifier:";
    let compact_notice = r"Copyright(?: \d{4})? .+[\n\r]?.*Use of this source code is governed by an .+-style license that can be.*found in the LICENSE file.*";

    let spdx_regex = Regex::new(spdx_notice).expect("Invalid regex");
    let compact_regex = Regex::new(compact_notice).expect("Invalid regex");

    let f_content = file_content.as_ref();
    spdx_regex.is_match(f_content) || compact_regex.is_match(f_content)
}
