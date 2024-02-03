// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use spdx::{imprecise_license_id, license_id, Expression, LicenseId, ParseMode};

pub fn try_find_by_id<I>(id: I) -> Option<LicenseId>
where
    I: AsRef<str>,
{
    let license = Expression::parse_mode(id.as_ref(), ParseMode::LAX);
    if let Err(err) = license {
        if let Some((license_id, _)) = imprecise_license_id(id.as_ref()) {
            return Some(license_id);
        }
        return None;
    }
    let license = license.unwrap();
    license_id(license.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_find_by_id_exact() {
        let license_exp = "BSD-3-Clause";
        let license_id = try_find_by_id(&license_exp);
        assert!(&license_id.is_some());
    }
}
