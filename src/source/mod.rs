// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;

use anyhow::Result;

fn prepend_license(path: &Path, tmpl: &str, data: LicenseData) -> Result<bool> {
    let lic = license_header(path, tmpl, &data)?;
    if lic.is_none() {
        return Ok(false);
    }

    let mut content = fs::read(path)?;
    if has_license(&content) || is_generated(&content) {
        return Ok(false);
    }

    let mut line = hash_bang(&content).unwrap_or_default();
    if !line.is_empty() {
        content = content.split_off(line.len());
        if line[line.len() - 1] != b'\n' {
            line.push(b'\n');
        }
        content = [line, lic.unwrap(), content].concat();
    } else {
        content = [lic.unwrap(), content].concat();
    }

    fs::write(path, content)?;

    Ok(true)
}

fn license_header(path: &Path, tmpl: &str, data: &LicenseData) -> Result<Option<Vec<u8>>> {
    todo!()
}

fn has_license(content: &[u8]) -> bool {
    false
}

fn is_generated(content: &[u8]) -> bool {
    false
}

struct LicenseData;

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

/// Extracts the hash-bang line from the given byte slice.
///
/// The hash-bang line is the first line in the slice ending with a newline character.
/// It checks if the lowercase hash-bang line starts with any of the specified prefixes.
///
/// Returns the hash-bang line if a matching prefix is found, otherwise returns `None`.
fn hash_bang(b: &[u8]) -> Option<Vec<u8>> {
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
