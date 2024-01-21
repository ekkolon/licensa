// Copyright 2024 Nelson Dominguez
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::ops::RangeInclusive;

use crate::utils::current_year;

const EARLIEST_LICENSE_YEAR: u16 = 1956;
const SECONDS_IN_YEAR: u64 = 365 * 24 * 60 * 60;

/// Check whether a string slice represents a year that falls
/// within range [EARLIEST_LICENSE_YEAR] and *current* year.
pub fn acceptable_year(s: &str) -> Result<u16, String> {
  let year: u16 = s
    .parse()
    .map_err(|_| format!("`{}` isn't a valid year", s))?;

  let acceptable_range = get_acceptable_year_range();
  if acceptable_range.contains(&year) {
    Ok(year)
  } else {
    Err(format!(
      "Year not in valid range {}-{}",
      acceptable_range.start(),
      acceptable_range.end()
    ))
  }
}

#[inline]
fn get_acceptable_year_range() -> RangeInclusive<u16> {
  EARLIEST_LICENSE_YEAR..=current_year()
}

#[cfg(test)]
#[path = "./validator_test.rs"]
mod tests;
