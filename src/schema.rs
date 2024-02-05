// Copyright 2021-present Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::utils::validate::is_valid_year;
use crate::{error::exit_invalid_value_err, spdx::try_find_by_id};

use anyhow::anyhow;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

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

#[derive(Debug, Clone)]
pub struct LicenseYear {
    start: u32,
    end: Option<u32>,
    is_present: bool,
}

impl LicenseYear {
    // Constructor for single year
    pub fn single_year(year: u32) -> Self {
        if !is_valid_year(year) {
            exit_invalid_value_err("year", &year.to_string(), None)
        }

        LicenseYear {
            start: year,
            end: None,
            is_present: false,
        }
    }

    // Constructor for present
    pub fn present_year(year: u32) -> Self {
        if !is_valid_year(year) {
            exit_invalid_value_err("year", &year.to_string(), None)
        }

        LicenseYear {
            start: year,
            end: None,
            is_present: true,
        }
    }

    // Constructor for range
    pub fn year_range(start: u32, end: u32) -> Self {
        if !is_valid_year(start) {
            exit_invalid_value_err("start", &start.to_string(), None)
        }
        if !is_valid_year(end) {
            exit_invalid_value_err("end", &end.to_string(), None)
        }

        LicenseYear {
            start,
            end: Some(end),
            is_present: false,
        }
    }

    // Add a validation method to check if the years are valid
    pub fn is_valid(&self) -> bool {
        if self.is_present {
            true
        } else if let Some(end) = self.end {
            self.start <= end
        } else {
            true
        }
    }
}

impl FromStr for LicenseYear {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        match parts.len() {
            1 => {
                let year = parts[0].parse::<u32>().map_err(|_| "Invalid year")?;
                Ok(LicenseYear::single_year(year))
            }
            2 => {
                let start = parts[0].parse::<u32>().map_err(|_| "Invalid start year")?;
                if parts[1] == "present" {
                    Ok(LicenseYear::present_year(start))
                } else {
                    let end = parts[1].parse::<u32>().map_err(|_| "Invalid end year")?;
                    Ok(LicenseYear::year_range(start, end))
                }
            }
            _ => Err("Invalid format"),
        }
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
                formatter.write_str("a string or an integer")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                parse_string(value)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                parse_integer(value)
            }
        }

        deserializer.deserialize_any(LicenseYearVisitor)
    }
}

fn parse_string<E>(value: &str) -> Result<LicenseYear, E>
where
    E: de::Error,
{
    if let Some(index) = value.find('-') {
        let start = &value[..index];
        let end = &value[(index + 1)..];

        match (start.parse::<u32>(), end.parse::<u32>()) {
            (Ok(start), Ok(end)) => Ok(LicenseYear {
                start,
                end: Some(end),
                is_present: false,
            }),
            _ => Err(de::Error::custom("Invalid year range")),
        }
    } else if let Ok(start) = value.parse::<u32>() {
        Ok(LicenseYear {
            start,
            end: None,
            is_present: false,
        })
    } else {
        Err(de::Error::custom(format!("Invalid year format: {}", value)))
    }
}

fn parse_integer<E>(value: u64) -> Result<LicenseYear, E>
where
    E: de::Error,
{
    // Parse as u16 and convert to LicenseYear
    if is_valid_year(value) {
        Ok(LicenseYear {
            start: value as u32,
            end: None,
            is_present: false,
        })
    } else {
        Err(de::Error::custom("Negative value is not a valid year"))
    }
}
