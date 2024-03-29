// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::ops::RangeInclusive;

use crate::utils::current_year;

const EARLIEST_LICENSE_YEAR: u32 = 1956;
const SECONDS_IN_YEAR: u64 = 365 * 24 * 60 * 60;

/// Check whether a string slice represents a year that falls
/// within range [EARLIEST_LICENSE_YEAR] and *current* year.
pub fn acceptable_year(s: &str) -> Result<u32, String> {
    let year: u32 = s
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

pub fn is_valid_year<T>(input: T) -> bool
where
    T: ToString,
{
    let input = input.to_string();
    if input.len() != 4 {
        return false; // Year must be 4 digits
    }

    let digits: Vec<char> = input.chars().collect();
    for digit in digits {
        if !digit.is_ascii_digit() {
            return false; // Year must only contain digits
        }
    }

    // Assume parse succeeds
    let year = input.parse();
    if let Ok(year) = year {
        if !(1..=9999).contains(&year) {
            return false; // Year must be within 1 to 9999 range
        }

        if year % 4 == 0 && year % 100 == 0 && year % 400 != 0 {
            return false; // Not a valid leap year
        }

        return true; // Valid year
    }

    false
}

#[inline]
fn get_acceptable_year_range() -> RangeInclusive<u32> {
    EARLIEST_LICENSE_YEAR..=current_year()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_license_year() {
        assert_eq!(acceptable_year("1956"), Ok(1956));
    }

    #[test]
    fn valid_earliest_license_year() {
        assert_eq!(
            acceptable_year(&EARLIEST_LICENSE_YEAR.to_string()),
            Ok(EARLIEST_LICENSE_YEAR)
        );
    }

    #[test]
    fn valid_current_year() {
        let current_year = current_year();
        assert_eq!(acceptable_year(&current_year.to_string()), Ok(current_year));
    }

    #[test]
    fn invalid_non_numeric_input() {
        assert_eq!(
            acceptable_year("invalid"),
            Err("`invalid` isn't a valid year".to_string())
        );
    }

    #[test]
    fn invalid_future_year() {
        let this_year = current_year();
        let future_year = this_year + 1;
        let actual = acceptable_year(&future_year.to_string());
        let expected = Err(format!(
            "Year not in valid range {}-{}",
            EARLIEST_LICENSE_YEAR, this_year
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_earlier_than_earliest_license_year() {
        let earlier_year = EARLIEST_LICENSE_YEAR - 1;
        let actual = acceptable_year(&earlier_year.to_string());
        let expected = Err(format!(
            "Year not in valid range {}-{}",
            EARLIEST_LICENSE_YEAR,
            current_year()
        ));

        assert_eq!(actual, expected);
    }
}
