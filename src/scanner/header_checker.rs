// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use regex::Regex;

pub fn contains_copyright_notice<F: AsRef<str>>(file_content: F) -> bool {
    let spdx_notice = r"Copyright(?: \d{4})? .+[\n\r]?.*SPDX-License-Identifier:";
    let compact_notice = r"Copyright(?: \d{4})? .+[\n\r]?.*Use of this source code is governed by an .+-style license that can be.*found in the LICENSE file.*";

    let spdx_regex = Regex::new(spdx_notice).expect("Invalid regex");
    let compact_regex = Regex::new(compact_notice).expect("Invalid regex");

    let f_content = file_content.as_ref();
    spdx_regex.is_match(f_content) || compact_regex.is_match(f_content)
}
