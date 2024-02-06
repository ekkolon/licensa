// Copyright 2021-present Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::spdx::try_find_by_id;
use crate::utils::validate::is_valid_year;

use anyhow::{anyhow, Result};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use std::{fmt, ops::Deref, str::FromStr};

// =========================================================
// =========================================================
// License SPDX ID
// =========================================================
#[derive(Debug, Clone)]
pub struct LicenseId(pub String);

impl From<String> for LicenseId {
    fn from(s: String) -> Self {
        LicenseId(s)
    }
}

impl<'a> From<&'a str> for LicenseId {
    fn from(s: &'a str) -> Self {
        LicenseId(s.to_string())
    }
}

impl FromStr for LicenseId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let expr = s.trim();
        if expr.is_empty() {
            return Err(anyhow!("License ID cannot be empty"));
        }

        let license_id = try_find_by_id(expr)?;
        if license_id.is_none() {
            let err_msg = format!("invalid SPDX License ID or expression '{}'", expr);
            return Err(anyhow!(err_msg));
        }

        Ok(LicenseId(license_id.unwrap()))
    }
}

impl Deref for LicenseId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for LicenseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for LicenseId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for LicenseId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = String::deserialize(deserializer)?;
        let input = input.trim_matches('"');

        let license_id = try_find_by_id(input);
        if let Err(err) = license_id {
            return Err(serde::de::Error::custom(err));
        }

        let license_id = license_id.unwrap();
        if license_id.is_none() {
            let err_msg = format!("invalid SPDX License ID or expression '{}'", input);
            return Err(serde::de::Error::custom(err_msg));
        }

        Ok(LicenseId(license_id.unwrap()))
    }
}

// =========================================================
// =========================================================
// License year
// =========================================================

#[derive(Debug, Error)]
pub enum LicenseYearError {
    #[error(
        "license year must be a non-empty string in one of the following formats: YYYY, YYYY-YYYY, or YYYY-present"
    )]
    EmptyString,

    #[error(
        "invalid license year format {0}. Expected value in one of the following formats: YYYY, YYYY-YYYY, or YYYY-present"
    )]
    InvalidFormat(String),

    #[error("{0} does not represent a calendar year")]
    InvalidYear(String),

    #[error("the starting year {0} of a license period must be less than the ending year {1} of the period")]
    InvalidPeriod(u32, u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LicenseYear {
    start: u32,
    end: Option<u32>,
    is_present: bool,
}

impl LicenseYear {
    // Constructor for single year
    pub fn single_year(year: u32) -> Result<Self, LicenseYearError> {
        if !is_valid_year(year) {
            return Err(LicenseYearError::InvalidYear(year.to_string()));
        }

        Ok(LicenseYear {
            start: year,
            end: None,
            is_present: false,
        })
    }

    // Constructor for present
    pub fn present_year(year: u32) -> Result<Self, LicenseYearError> {
        let mut license_year = LicenseYear::single_year(year)?;
        license_year.is_present = true;
        Ok(license_year)
    }

    // Constructor for range
    pub fn year_range(start: u32, end: u32) -> Result<Self, LicenseYearError> {
        let mut license_year = LicenseYear::single_year(start)?;

        if !is_valid_year(end) {
            return Err(LicenseYearError::InvalidYear(end.to_string()));
        }

        if start >= end {
            return Err(LicenseYearError::InvalidPeriod(start, end));
        }

        license_year.end = Some(end);

        Ok(license_year)
    }
}

impl FromStr for LicenseYear {
    type Err = LicenseYearError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = value.split('-').collect();

        if parts.is_empty() {
            return Err(LicenseYearError::EmptyString);
        }

        let num_parts = parts.len();
        if num_parts > 2 {
            return Err(LicenseYearError::InvalidFormat(value.to_string()));
        }

        let start = parts[0];
        if !is_valid_year(start) {
            return Err(LicenseYearError::InvalidYear(value.to_string()));
        }
        let start: u32 = start.parse().unwrap();

        if num_parts == 1 {
            return Ok(LicenseYear {
                end: None,
                is_present: false,
                start,
            });
        }

        let end = parts[1];
        if end == "present" {
            return Ok(LicenseYear {
                end: None,
                is_present: true,
                start,
            });
        } else if !is_valid_year(end) {
            return Err(LicenseYearError::InvalidYear(end.to_string()));
        }

        let end: u32 = end.parse().unwrap();

        if start >= end {
            return Err(LicenseYearError::InvalidPeriod(start, end));
        }

        Ok(LicenseYear {
            end: Some(end),
            is_present: false,
            start,
        })
    }
}

impl fmt::Display for LicenseYear {
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

impl Serialize for LicenseYear {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for LicenseYear {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LicenseYearVisitor;

        impl<'de> de::Visitor<'de> for LicenseYearVisitor {
            type Value = LicenseYear;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or an integer in one of the following formats: YYYY, YYYY-YYYY, or YYYY-present")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                visit_string(value)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                visit_int(value)
            }
        }

        deserializer.deserialize_any(LicenseYearVisitor)
    }
}

fn visit_string<E>(value: &str) -> Result<LicenseYear, E>
where
    E: de::Error,
{
    LicenseYear::from_str(value).map_err(|err| de::Error::custom::<LicenseYearError>(err))
}

fn visit_int<E>(value: u64) -> Result<LicenseYear, E>
where
    E: de::Error,
{
    if !is_valid_year(value) {
        return Err(de::Error::custom(LicenseYearError::InvalidYear(
            value.to_string(),
        )));
    }

    Ok(LicenseYear {
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
        let expected = LicenseYear {
            end: None,
            is_present: false,
            start: 2024,
        };

        let parsed = visit_int::<de::value::Error>(u64::from(year));
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), expected)
    }

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
        let expected = LicenseYear {
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
        let expected = LicenseYear {
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
        let expected = LicenseYear {
            end: None,
            is_present: true,
            start: 2022,
        };

        let parsed = visit_string::<de::value::Error>(year_range);

        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), expected)
    }
}
