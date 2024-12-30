// Copyright 2021-present Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::license::{Error, Result};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub struct LicensePeriod {
    start: u32,
    end: Option<u32>,
    is_present: bool,
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

impl LicensePeriod {
    // Constructor for single year
    pub fn single_year(year: u32) -> Result<Self> {
        if !is_valid_year(year) {
            return Err(Error::NotACalenderYear(year.clone().to_string()));
        }

        Ok(LicensePeriod {
            start: year,
            end: None,
            is_present: false,
        })
    }

    // Constructor for present
    pub fn present_year(year: u32) -> Result<Self> {
        Ok(LicensePeriod {
            is_present: true,
            ..LicensePeriod::single_year(year)?
        })
    }

    // Constructor for range
    pub fn year_range(start: u32, end: u32) -> Result<Self> {
        let mut license_year = LicensePeriod::single_year(start)?;

        if !is_valid_year(end) {
            return Err(Error::NotACalenderYear(end.to_string()));
        }

        if start >= end {
            return Err(Error::PeriodOutOfRange(start, end));
        }

        license_year.end = Some(end);

        Ok(license_year)
    }
}

impl FromStr for LicensePeriod {
    type Err = Error;

    fn from_str(value: &str) -> core::result::Result<Self, Self::Err> {
        // Trim leading and trailing `"` in case an user provides a single license year
        // as `--year "2003"`, where it should be provided as `--type 2003`.
        let value = value.trim_matches('"');

        let parts: Vec<&str> = value.split('-').collect();

        if parts.is_empty() {
            return Err(Error::InvalidPeriodFormat(value.into()));
        }

        let num_parts = parts.len();
        if num_parts > 2 {
            return Err(Error::InvalidPeriodFormat(value.to_string()));
        }

        let start = parts[0];
        if !is_valid_year(start) {
            return Err(Error::NotACalenderYear(start.to_string()));
        }
        let start: u32 = start.parse().unwrap();

        if num_parts == 1 {
            return Ok(LicensePeriod {
                end: None,
                is_present: false,
                start,
            });
        }

        let end = parts[1];
        if end == "present" {
            return Ok(LicensePeriod {
                end: None,
                is_present: true,
                start,
            });
        } else if !is_valid_year(end) {
            return Err(Error::NotACalenderYear(end.to_string()));
        }

        let end: u32 = end.parse().unwrap();

        if start >= end {
            return Err(Error::PeriodOutOfRange(start, end));
        }

        Ok(LicensePeriod {
            end: Some(end),
            is_present: false,
            start,
        })
    }
}

impl fmt::Display for LicensePeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_present {
            write!(f, "{}-present", self.start)
        } else if let Some(end) = self.end {
            write!(f, "{}-{}", self.start, end)
        } else {
            write!(f, "{}", self.start)
        }
    }
}

impl Serialize for LicensePeriod {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for LicensePeriod {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LicensePeriodVisitor;

        impl de::Visitor<'_> for LicensePeriodVisitor {
            type Value = LicensePeriod;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or an integer in one of the following formats: YYYY, YYYY-YYYY, or YYYY-present")
            }

            fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                visit_string(value)
            }

            fn visit_u64<E>(self, value: u64) -> core::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                visit_int(value)
            }
        }

        deserializer.deserialize_any(LicensePeriodVisitor)
    }
}

fn visit_string<E>(value: &str) -> core::result::Result<LicensePeriod, E>
where
    E: de::Error,
{
    LicensePeriod::from_str(value).map_err(|err| de::Error::custom::<Error>(err))
}

fn visit_int<E>(value: u64) -> core::result::Result<LicensePeriod, E>
where
    E: de::Error,
{
    if !is_valid_year(value) {
        return Err(de::Error::custom(Error::NotACalenderYear(
            value.to_string(),
        )));
    }

    Ok(LicensePeriod {
        start: value as u32,
        end: None,
        is_present: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_license_year_single_int() {
        let year: u32 = 2024;
        let expected = LicensePeriod {
            end: None,
            is_present: false,
            start: 2024,
        };

        let parsed = visit_int::<de::value::Error>(u64::from(year));
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), expected)
    }

    #[test]
    fn test_parse_license_year_invalid_single_int() {
        let year: u32 = 193;
        let parsed = visit_int::<de::value::Error>(u64::from(year));
        assert!(parsed.is_err());

        let year: u32 = 20244;
        let parsed = visit_int::<de::value::Error>(u64::from(year));
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_license_year_invalid_range_start_equals_end() {
        let period = "2022-2022";
        let parsed = visit_string::<de::value::Error>(period);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_license_year_invalid_range_start_greater_end() {
        let period = "2023-2022";
        let parsed = visit_string::<de::value::Error>(period);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_license_year_invalid_string() {
        let year = "209O";
        let parsed = visit_string::<de::value::Error>(year);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_license_year_single_str() {
        let year = "2024";
        let expected = LicensePeriod {
            end: None,
            is_present: false,
            start: 2024,
        };

        let parsed = visit_string::<de::value::Error>(year);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), expected)
    }

    #[test]
    fn test_parse_license_year_to_year() {
        let period = "2011-2014";
        let expected = LicensePeriod {
            end: Some(2014),
            is_present: false,
            start: 2011,
        };

        let parsed = visit_string::<de::value::Error>(period);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), expected)
    }

    #[test]
    fn test_parse_license_year_to_present() {
        let year_range = "2022-present";
        let expected = LicensePeriod {
            end: None,
            is_present: true,
            start: 2022,
        };

        let parsed = visit_string::<de::value::Error>(year_range);

        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), expected)
    }
}
