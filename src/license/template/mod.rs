// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod cache;
pub mod copyright;
pub mod header;

use cache::Cachable;

const BREAKWORDS: &[&str] = &[
    "spdx-license-identifier: ",
    "copyright (c)",
    "all rights reserved",
    "mozilla public license",
    "academic free license",
    "gnu affero general public license",
    "gnu lesser general public license",
    "gnu free documentation license",
    "educational community license",
    "mulan psl v2",
    "copyright ",
];

// FIXME: This is a simple, naive attempt to detect licene headers.
// One improvement would be to only consider breakwords within
// comment lines.
pub fn has_copyright_notice(b: &[u8]) -> bool {
    let n = std::cmp::min(1000, b.len());
    let lower_b: Vec<u8> = b[..n].iter().map(|&c| c.to_ascii_lowercase()).collect();

    let bytes = BREAKWORDS.iter().map(|w| w.as_bytes());

    for license in bytes {
        if lower_b
            .windows(license.len())
            .any(|window| window == license)
        {
            return true;
        }
    }

    false
}

#[derive(Debug, Clone)]
pub struct HeaderTemplate {
    pub extension: String,
    pub template: String,
}

impl Cachable for HeaderTemplate {
    fn cache_id(&self) -> String {
        self.extension.to_owned()
    }
}
