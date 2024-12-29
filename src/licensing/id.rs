// Copyright 2021-present Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::licensing::license::try_find_by_id;
use crate::licensing::Error;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, ops::Deref, str::FromStr};

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
    type Err = Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        // We trim leading and trailing `"` in case an user provides a single license ID
        // as `--type "MIT"`, whereas it should be provided as `--type MIT`.
        let expr = s.trim().trim_matches('"');

        if expr.is_empty() {
            return Err(Error::EmptyLicenseId);
        }

        let license_id = try_find_by_id(expr)?;
        if license_id.is_none() {
            return Err(Error::InvslidLicenseIdOrExpression(expr.into()));
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
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for LicenseId {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
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
