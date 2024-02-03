// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use spdx::{imprecise_license_id, license_id, Expression, LicenseId, LicenseItem, ParseMode};

pub struct SpdxSearchResult {
    license_id: LicenseId,
    single: bool,
}

pub fn try_find_by_id<I>(expr: I) -> Option<SpdxSearchResult>
where
    I: AsRef<str>,
{
    let license = Expression::parse_mode(expr.as_ref(), ParseMode::LAX).ok()?;
    let requirements: Vec<&LicenseItem> = license.requirements().map(|r| &r.req.license).collect();
    license_id(license.as_ref())
        .or_else(|| imprecise_license_id(expr.as_ref()).map(|(license_id, _)| license_id))
        .map(|l| SpdxSearchResult {
            license_id: l,
            single: requirements.len() == 1,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_find_by_id_exact() {
        let expr = "BSD-3-Clause";
        let license_id = try_find_by_id(expr);
        assert!(&license_id.is_some());
    }

    #[test]
    fn test_try_find_by_id_invalid() {
        let expr = "BSE";
        let license_id = try_find_by_id(expr);
        assert!(&license_id.is_none());
    }

    #[test]
    fn test_try_find_by_id_combined() {
        let expr = "mit or apache";
        let license_id = try_find_by_id(expr);
        assert!(&license_id.is_some());
    }
}
