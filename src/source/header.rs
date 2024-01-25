// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

pub struct SourceHeaderPrefix<'a>(&'a str, &'a str, &'a str);

pub struct SourceHeaderDef<'a> {
  pub extensions: &'a [&'a str],
  pub prefix: SourceHeaderPrefix<'a>,
}

const SOURCE_HEADERS_DEFS: [SourceHeaderDef; 10] = [
  SourceHeaderDef {
    extensions: &[".c", ".h", ".gv", ".java", ".scala", ".kt", ".kts"],
    prefix: SourceHeaderPrefix("/*", " * ", " */"),
  },
  SourceHeaderDef {
    extensions: &[
      ".js", ".mjs", ".cjs", ".jsx", ".tsx", ".css", ".scss", ".sass", ".ts",
    ],
    prefix: SourceHeaderPrefix("/**", " * ", " */"),
  },
  SourceHeaderDef {
    extensions: &[
      ".cc", ".cpp", ".cs", ".go", ".hcl", ".hh", ".hpp", ".m", ".mm", ".proto", ".rs",
      ".swift", ".dart", ".groovy", ".v", ".sv", ".php",
    ],
    prefix: SourceHeaderPrefix("", "// ", ""),
  },
  SourceHeaderDef {
    extensions: &[
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
    prefix: SourceHeaderPrefix("", "# ", ""),
  },
  SourceHeaderDef {
    extensions: &[".el", ".lisp"],
    prefix: SourceHeaderPrefix("", ";; ", ""),
  },
  SourceHeaderDef {
    extensions: &[".erl"],
    prefix: SourceHeaderPrefix("", "% ", ""),
  },
  SourceHeaderDef {
    extensions: &[".hs", ".sql", ".sdl"],
    prefix: SourceHeaderPrefix("", "-- ", ""),
  },
  SourceHeaderDef {
    extensions: &[".html", ".xml", ".vue", ".wxi", ".wxl", ".wxs"],
    prefix: SourceHeaderPrefix("<!--", " ", "-->"),
  },
  SourceHeaderDef {
    extensions: &[".j2"],
    prefix: SourceHeaderPrefix("{#", "", "#}"),
  },
  SourceHeaderDef {
    extensions: &[".ml", ".mli", ".mll", ".mly"],
    prefix: SourceHeaderPrefix("(**", "   ", "*)"),
  },
  // TODO: 	handle cmake files
];
