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

use chrono::{Datelike, TimeZone};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current year as determined by the OS.
///
/// This function panics if the current timestamp cannot be determined
/// or is invalid, that is the timestamp seconds is out of range.
pub fn current_year() -> u16 {
  let current_timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Failed to get current timestamp")
    .as_secs();

  chrono::Utc
    .timestamp_opt(current_timestamp as i64, 0)
    .unwrap()
    .year() as u16
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_current_year() {
    // Test the current_year function
    let current_year = current_year();

    // Get the current year using chrono
    let chrono_current_year = chrono::Utc::now().year() as u16;

    // Ensure that the current year matches the one obtained from chrono
    assert_eq!(current_year, chrono_current_year);
  }

}
