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

use lazy_static::lazy_static;

lazy_static! {
  /// Represents a predefined list of source header definitions.
  static ref SOURCE_HEADER_DEFINITIONS: Vec<SourceHeaderDefinition<'static>> = vec![
    SourceHeaderDefinition {
      extensions: vec![".c", ".h", ".gv", ".java", ".scala", ".kt", ".kts"],
      header_prefix: SourceHeaderPrefix::new("/*", " * ", " */"),
    },
    SourceHeaderDefinition {
      extensions: vec![
        ".js", ".mjs", ".cjs", ".jsx", ".tsx", ".css", ".scss", ".sass", ".ts",
      ],
      header_prefix: SourceHeaderPrefix::new("/**", " * ", " */"),
    },
    SourceHeaderDefinition {
      extensions: vec![
        ".cc", ".cpp", ".cs", ".go", ".hcl", ".hh", ".hpp", ".m", ".mm", ".proto", ".rs",
        ".swift", ".dart", ".groovy", ".v", ".sv", ".php",
      ],
      header_prefix: SourceHeaderPrefix::new("", "// ", ""),
    },
    SourceHeaderDefinition {
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
      header_prefix: SourceHeaderPrefix::new("", "# ", ""),
    },
    SourceHeaderDefinition {
      extensions: vec![".el", ".lisp"],
      header_prefix: SourceHeaderPrefix::new("", ";; ", ""),
    },
    SourceHeaderDefinition {
      extensions: vec![".erl"],
      header_prefix: SourceHeaderPrefix::new("", "% ", ""),
    },
    SourceHeaderDefinition {
      extensions: vec![".hs", ".sql", ".sdl"],
      header_prefix: SourceHeaderPrefix::new("", "-- ", ""),
    },
    SourceHeaderDefinition {
      extensions: vec![".html", ".xml", ".vue", ".wxi", ".wxl", ".wxs"],
      header_prefix: SourceHeaderPrefix::new("<!--", " ", "-->"),
    },
    SourceHeaderDefinition {
      extensions: vec![".j2"],
      header_prefix: SourceHeaderPrefix::new("{#", "", "#}"),
    },
    SourceHeaderDefinition {
      extensions: vec![".ml", ".mli", ".mll", ".mly"],
      header_prefix: SourceHeaderPrefix::new("(**", "   ", "*)"),
    },
    // TODO: 	handle cmake files
  ];
}
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
  ) -> Option<&'a SourceHeaderDefinition<'a>> {
    SOURCE_HEADER_DEFINITIONS
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
  ) -> Option<&'a SourceHeaderPrefix<'a>> {
    SourceHeaders::find_header_definition_by_extension(&extension)
      .map(|source| &source.header_prefix)
  }
}

/// Represents a source header definition with a list of file extensions and a corresponding prefix.
pub struct SourceHeaderDefinition<'a> {
  /// List of file extensions associated with the header definition.
  pub extensions: Vec<&'a str>,
  /// Corresponding source header prefix.
  pub header_prefix: SourceHeaderPrefix<'a>,
}

impl<'a> SourceHeaderDefinition<'a> {
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
pub struct SourceHeaderPrefix<'a> {
  /// Top part of the header.
  pub top: &'a str,
  /// Middle part of the header.
  pub mid: &'a str,
  /// Bottom part of the header.
  pub bottom: &'a str,
}

impl<'a> SourceHeaderPrefix<'a> {
  /// Creates a new `SourceHeaderPrefix` instance with the specified top, mid, and bottom parts.
  ///
  /// # Example
  ///
  /// ```no_run
  /// use licensa::scanner::SourceHeaderPrefix;
  ///
  /// let header_prefix = SourceHeaderPrefix::new("/**", " * ", " */");
  /// ```
  pub fn new(top: &'a str, mid: &'a str, bottom: &'a str) -> SourceHeaderPrefix<'a> {
    SourceHeaderPrefix { top, mid, bottom }
  }
}
