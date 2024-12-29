// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::template::header::SourceHeaders;
use ignore::DirEntry;

use std::borrow::Borrow;
use std::path::Path;

/// Checks if a directory entry is a candidate for applying a license.
pub fn is_candidate<E>(entry: E) -> bool
where
    E: Borrow<DirEntry>,
{
    let entry = entry.borrow();

    // Only consider entry if it is a regular file
    if !entry.file_type().map_or(false, |ftype| ftype.is_file()) {
        return false;
    }

    let path = entry.path();
    if path.file_name().is_none() && path.extension().is_none() {
        return false;
    }

    let lookup_name = get_path_suffix(path);
    SourceHeaders::find_header_definition_by_extension(&lookup_name).is_some()
}

#[inline]
pub fn get_path_suffix<P>(path: P) -> String
where
    P: AsRef<Path>,
{
    path.as_ref().extension().map_or_else(
        || {
            path.as_ref()
                .file_name()
                .and_then(|name| name.to_str())
                .map_or(String::new(), |s| s.to_owned())
        },
        |extension| {
            let mut lookup_name = String::with_capacity(extension.len() + 1);
            lookup_name.push('.');
            lookup_name.push_str(extension.to_str().unwrap_or_default());
            lookup_name
        },
    )
}
