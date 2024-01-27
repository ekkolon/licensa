// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

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
