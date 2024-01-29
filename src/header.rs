// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

//! # Source Headers
//!
//! The `SourceHeaders` struct provides methods for finding header definitions and prefixes based on file extensions.
//! It contains a predefined list of `SourceHeaderDefinition` instances.
//!
//! # Example
//!
//! ```no_run
//! use licensa::scanner::SourceHeaders;
//!
//! let extension = ".rs";
//! if let Some(header_prefix) = SourceHeaders::find_header_prefix_for_extension(extension) {
//!     println!("Header Prefix: {}", header_prefix.top);
//! }
//! ```
//!
//! # Source Header Definitions
//!
//! Each source header definition includes a list of file extensions and a corresponding `SourceHeaderPrefix`.
//!
//! # Example
//!
//! ```no_run
//! use licensa::scanner::{SourceHeaders, SourceHeaderPrefix};
//!
//! let header_definition = SourceHeaders::find_header_definition_by_extension(".rs");
//! if let Some(header_def) = header_definition {
//!     println!("Extensions: {:?}", header_def.extensions);
//!     println!("Header Prefix: {}", header_def.header_prefix.top);
//! }
//! ```

use anyhow::Result;
use lazy_static::lazy_static;

lazy_static! {
  /// Represents a predefined list of source header definitions.
  static ref HEADER_DEFINITIONS: Vec<HeaderDefinition<'static>> = vec![
    HeaderDefinition {
      extensions: vec![".c", ".h", ".gv", ".java", ".scala", ".kt", ".kts"],
      header_prefix: HeaderPrefix::new("/*", " * ", " */"),
    },
    HeaderDefinition {
      extensions: vec![
        ".js", ".mjs", ".cjs", ".jsx", ".tsx", ".css", ".scss", ".sass", ".ts",
      ],
      header_prefix: HeaderPrefix::new("/**", " * ", " */"),
    },
    HeaderDefinition {
      extensions: vec![
        ".cc", ".cpp", ".cs", ".go", ".hcl", ".hh", ".hpp", ".m", ".mm", ".proto", ".rs",
        ".swift", ".dart", ".groovy", ".v", ".sv", ".php",
      ],
      header_prefix: HeaderPrefix::new("", "// ", ""),
    },
    HeaderDefinition {
      extensions: vec![
        ".py",
        ".sh",
        ".yaml",
        ".yml",
        ".dockerfile",
        "dockerfile",
        ".rb",
        "gemfile",
        ".tcl",
        ".tf",
        ".bzl",
        ".pl",
        ".pp",
        "build",
        ".build",
        ".toml",
      ],
      header_prefix: HeaderPrefix::new("", "# ", ""),
    },
    HeaderDefinition {
      extensions: vec![".el", ".lisp"],
      header_prefix: HeaderPrefix::new("", ";; ", ""),
    },
    HeaderDefinition {
      extensions: vec![".erl"],
      header_prefix: HeaderPrefix::new("", "% ", ""),
    },
    HeaderDefinition {
      extensions: vec![".hs", ".sql", ".sdl"],
      header_prefix: HeaderPrefix::new("", "-- ", ""),
    },
    HeaderDefinition {
      extensions: vec![".html", ".xml", ".vue", ".wxi", ".wxl", ".wxs"],
      header_prefix: HeaderPrefix::new("<!--", " ", "-->"),
    },
    HeaderDefinition {
      extensions: vec![".j2"],
      header_prefix: HeaderPrefix::new("{#", "", "#}"),
    },
    HeaderDefinition {
      extensions: vec![".ml", ".mli", ".mll", ".mly"],
      header_prefix: HeaderPrefix::new("(**", "   ", "*)"),
    },
    // TODO: 	handle cmake files
  ];
}

const HEAD: &[&str] = &[
    // shell script
    "#!",
    // XML declaratioon
    "<?xml",
    // HTML doctype
    "<!doctype",
    // Ruby encoding
    "# encoding:",
    // Ruby interpreter instruction
    "# frozen_string_literal:",
    // PHP opening tag
    "<?php",
    // Dockerfile directive https://docs.docker.com/engine/reference/builder/#parser-directives
    "# escape",
    "# syntax",
];

/// Represents a utility for working with source headers.
pub struct SourceHeaders;

impl SourceHeaders {
    /// Finds the header definition based on the given file extension.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use licensa::scanner::SourceHeaders;
    ///
    /// let extension = ".rs";
    /// if let Some(header_definition) = SourceHeaders::find_header_definition_by_extension(extension) {
    ///     // Do something with the header definition
    /// }
    /// ```
    pub fn find_header_definition_by_extension<'a, E: AsRef<str>>(
        extension: E,
    ) -> Option<&'a HeaderDefinition<'a>> {
        HEADER_DEFINITIONS
            .iter()
            .find(|source| source.contains_extension(Some(&extension)))
    }

    /// Finds the header prefix based on the given file extension.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use licensa::scanner::SourceHeaders;
    ///
    /// let extension = ".rs";
    /// if let Some(header_prefix) = SourceHeaders::find_header_prefix_for_extension(extension) {
    ///     // Do something with the header prefix
    /// }
    /// ```
    pub fn find_header_prefix_for_extension<'a, E: AsRef<str>>(
        extension: E,
    ) -> Option<&'a HeaderPrefix<'a>> {
        SourceHeaders::find_header_definition_by_extension(&extension)
            .map(|source| &source.header_prefix)
    }
}

/// Represents a source header definition with a list of file extensions and a corresponding prefix.
pub struct HeaderDefinition<'a> {
    /// List of file extensions associated with the header definition.
    pub extensions: Vec<&'a str>,
    /// Corresponding source header prefix.
    pub header_prefix: HeaderPrefix<'a>,
}

impl<'a> HeaderDefinition<'a> {
    /// Checks if the given extension is contained in the list of file extensions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use licensa::scanner::SourceHeaderDefinition;
    ///
    /// let header_def = SourceHeaderDefinition {
    ///     extensions: vec![".rs", ".toml"],
    ///     header_prefix: Default::default(),
    /// };
    ///
    /// let is_contained = header_def.contains_extension(Some(".rs"));
    /// assert_eq!(is_contained, true);
    /// ```
    pub fn contains_extension<E: AsRef<str>>(&self, extension: Option<E>) -> bool {
        extension
            .map_or(false, |e| self.extensions.contains(&e.as_ref()))
            .to_owned()
    }
}

/// Represents the prefix structure for a source header.
#[derive(Debug, Clone)]
pub struct HeaderPrefix<'a> {
    /// Top part of the header.
    pub top: &'a str,
    /// Middle part of the header.
    pub mid: &'a str,
    /// Bottom part of the header.
    pub bottom: &'a str,
}

impl<'a> HeaderPrefix<'a> {
    // execute_template will execute a license template t with data d
    // and prefix the result with top, middle and bottom.
    pub fn apply<T>(&self, template: T) -> Result<String>
    where
        T: AsRef<str>,
    {
        let Self { bottom, mid, top } = &self;

        let mut out = String::new();
        if !top.is_empty() {
            out.push_str(top);
            out.push('\n');
        }

        let lines = template.as_ref().lines();
        for line in lines {
            out.push_str(mid);
            out.push_str(line.trim_end_matches(char::is_whitespace));
            out.push('\n');
        }

        if !bottom.is_empty() {
            out.push_str(bottom);
            out.push('\n');
        }

        out.push('\n');

        Ok(out)
    }

    /// Creates a new `SourceHeaderPrefix` instance with the specified top, mid, and bottom parts.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use licensa::scanner::SourceHeaderPrefix;
    ///
    /// let header_prefix = SourceHeaderPrefix::new("/**", " * ", " */");
    /// ```
    pub fn new(top: &'a str, mid: &'a str, bottom: &'a str) -> HeaderPrefix<'a> {
        HeaderPrefix { top, mid, bottom }
    }
}

/// Extracts the hash-bang line from the given byte slice.
///
/// The hash-bang line is the first line in the slice ending with a newline character.
/// It checks if the lowercase hash-bang line starts with any of the specified prefixes.
///
/// Returns the hash-bang line if a matching prefix is found, otherwise returns `None`.
pub fn extract_hash_bang(b: &[u8]) -> Option<Vec<u8>> {
    let mut line = Vec::new();

    for &c in b {
        line.push(c);
        if c == b'\n' {
            break;
        }
    }

    let first = String::from_utf8_lossy(&line).to_lowercase();

    for &h in HEAD {
        if first.starts_with(h) {
            return Some(line);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        copyright_notice::{
            CompactCopyrightNotice, SpdxCopyrightNotice, COMPACT_COPYRIGHT_NOTICE,
            SPDX_COPYRIGHT_NOTICE,
        },
        header::SourceHeaders,
        interpolation::interpolate, // test_utils::create_temp_file,
    };

    #[test]
    fn test_execute_template_spdx_copyright_notice() {
        let rs_header_prefix = SourceHeaders::find_header_prefix_for_extension(".rs").unwrap();

        // Test case 1
        let data = SpdxCopyrightNotice {
            year: 2022,
            fullname: "Jane Doe".to_string(),
            license: "MIT".to_string(),
        };

        let result = rs_header_prefix
            .apply(interpolate!(SPDX_COPYRIGHT_NOTICE, &data).unwrap())
            .unwrap();
        let expected: &str = r#"// Copyright 2022 Jane Doe
// SPDX-License-Identifier: MIT
"#;
        assert_eq!(&result, expected);

        // Test case 2: Empty template and prefix
        let empty_template = "";
        let result = rs_header_prefix.apply(empty_template).unwrap();
        let expected = "";
        assert_eq!(&result, expected);

        // JavaScript
        let js_header_prefix = SourceHeaders::find_header_prefix_for_extension(".js").unwrap();
        let result = js_header_prefix
            .apply(interpolate!(SPDX_COPYRIGHT_NOTICE, &data).unwrap())
            .unwrap();

        // Disable linting for template whitespace to be valid
        #[deny(clippy::all)]
        let expected: &str = r#"/**
 * Copyright 2022 Jane Doe
 * SPDX-License-Identifier: MIT
 */
"#;
        assert_eq!(&result, expected);

        let data = CompactCopyrightNotice {
            year: 2024,
            fullname: "John Doe".into(),
            license: "Apache-2.0".into(),
            determiner: "in".into(),
            location: "the root of this project".into(),
        };

        let js_header_prefix = SourceHeaders::find_header_prefix_for_extension(".js").unwrap();
        let result = js_header_prefix
            .apply(interpolate!(COMPACT_COPYRIGHT_NOTICE, &data).unwrap())
            .unwrap();

        // Disable linting for template whitespace to be valid
        #[deny(clippy::all)]
        let expected: &str = r#"/**
 * Copyright 2024 John Doe
 * 
 * Use of this source code is governed by an Apache-2.0-style license that can be
 * found in the LICENSE file in the root of this project.
 */
"#;

        assert_eq!(&result, expected);
    }

    #[test]
    fn test_hash_bang_with_valid_prefix() {
        // Test with a valid hash-bang line
        let input = "#!/bin/bash\nrest of the script".as_bytes();
        let result = extract_hash_bang(input);
        let expected = Some(b"#!/bin/bash\n".to_vec());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hash_bang_with_invalid_prefix() {
        // Test with an invalid hash-bang line
        let input = "Invalid hash-bang line\nrest of the script".as_bytes();
        let result = extract_hash_bang(input);
        let expected = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hash_bang_with_multiple_valid_prefixes() {
        // Test with multiple valid hash-bang prefixes
        let input = "<?xml\nrest of the content".as_bytes();
        let result = extract_hash_bang(input);
        let expected = Some(b"<?xml\n".to_vec());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hash_bang_with_empty_input() {
        // Test with an empty input
        let input = "".as_bytes();
        let result = extract_hash_bang(input);
        let expected = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hash_bang_with_partial_line() {
        // Test with a partial line (no newline character)
        let input = "#!/usr/bin/env python".as_bytes();
        let result = extract_hash_bang(input);
        let expected = Some("#!/usr/bin/env python".as_bytes().to_vec());
        assert_eq!(result, expected);
    }
}
