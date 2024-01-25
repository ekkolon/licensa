// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use lazy_static::lazy_static;

lazy_static! {
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

pub struct SourceHeaders;

impl SourceHeaders {
  pub fn find_header_definition_by_extension<'a, E: AsRef<str>>(
    extension: E,
  ) -> Option<&'a SourceHeaderDefinition<'a>> {
    SOURCE_HEADER_DEFINITIONS
      .iter()
      .find(|source| source.contains_extension(Some(&extension)))
  }

  pub fn find_header_prefix_for_extension<'a, E: AsRef<str>>(
    extension: E,
  ) -> Option<&'a SourceHeaderPrefix<'a>> {
    SourceHeaders::find_header_definition_by_extension(&extension)
      .map(|source| &source.header_prefix)
  }
}

pub struct SourceHeaderDefinition<'a> {
  pub extensions: Vec<&'a str>,
  pub header_prefix: SourceHeaderPrefix<'a>,
}

impl<'a> SourceHeaderDefinition<'a> {
  pub fn contains_extension<E: AsRef<str>>(&self, extension: Option<E>) -> bool {
    extension
      .map_or(false, |e| self.extensions.contains(&e.as_ref()))
      .to_owned()
  }
}

#[derive(Debug, Clone)]
pub struct SourceHeaderPrefix<'a> {
  pub top: &'a str,
  pub mid: &'a str,
  pub bottom: &'a str,
}

impl<'a> SourceHeaderPrefix<'a> {
  pub fn new(top: &'a str, mid: &'a str, bottom: &'a str) -> SourceHeaderPrefix<'a> {
    SourceHeaderPrefix { top, mid, bottom }
  }
}
