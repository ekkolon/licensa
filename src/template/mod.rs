// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

pub mod cache;
pub mod copyright;
pub mod header;
pub mod interpolation;

use regex::Regex;

pub fn has_copyright_notice<F: AsRef<str>>(file_content: F) -> bool {
    let comment_regex = Regex::new(r#"(//|/\*|\*/|\#|<!--|--|'|<!--|""|--\[\[|--\[)"#).unwrap();

    // Check for common license-related keywords within comments
    let license_keywords = ["copyright", "license", "spdx-license-identifier"];

    for keyword in &license_keywords {
        let pattern = format!(r"(?i)\b{}\b", keyword); // Use (?i) for case-insensitive matching
        let full_pattern = format!(r#"{}.*{}"#, comment_regex.as_str(), pattern);
        let regex = Regex::new(&full_pattern).expect("Invalid regex");

        if regex.is_match(file_content.as_ref()) {
            return true;
        }
    }

    false
}
