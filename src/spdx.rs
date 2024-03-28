// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{anyhow, Result};
use spdx::identifiers::LICENSES;
use spdx::{imprecise_license_id, license_id, Expression, ParseMode};

/// Tries to find a SPDX license identifier based on the provided expression.
///
/// This function accepts SPDX license expressions in various forms, such as
/// single license IDs ("MIT", "mit"), complex expressions ("MIT OR Apache-2.0"),
/// or a combination. It aims to provide a normalized and canonicalized SPDX
/// license identifier that can be used for further processing.
///
/// # Arguments
///
/// - `expr`: The SPDX license expression to analyze and process.
///
/// # Returns
///
/// Returns a `Result` with an `Option<String>`:
///
/// - `Ok(Some(license_id))`: If a SPDX license identifier is successfully found.
/// - `Ok(None)`: If the provided expression is not in its valid form but can be
///   canonicalized and parsed.
/// - `Err(_)`: If the expression cannot be canonicalized or other errors occur.
pub fn try_find_by_id<I>(expr: I) -> Result<Option<String>>
where
    I: AsRef<str>,
{
    let expr = expr.as_ref();

    if is_single_expr(expr) {
        // The casing doesn't matter here. We try to find the license id based
        // on a single SPDX expression such as "MIT", "mit", "apache2" etc.
        let license_id = partially_find_license(expr);
        return Ok(license_id);
    }

    if let Ok(license) = Expression::parse_mode(expr, ParseMode::LAX) {
        // At this point we just parse the expression in a non-strict mode.
        // We don't care about errors. In cases where the provided expression
        // is already in it's valid form (e.g "MIT OR Apache-2.0") the parser
        // will be happy.
        let license_id = Some(license.to_string());
        return Ok(license_id);
    }

    // If we reach the next line, the provided expression is not in it's valid form yet.
    // The `canonicalize` method converts the input expression to one that can be parsed
    // in strict mode.
    let expr = Expression::canonicalize(expr)?;
    Ok(expr)
}

fn partially_find_license(expr: &str) -> Option<String> {
    license_id(expr)
        .or_else(|| imprecise_license_id(expr).map(|(license_id, _)| license_id))
        .map(|l| l.name.to_string())
}

fn is_single_expr(expr: &str) -> bool {
    expr.split(' ').collect::<Vec<&str>>().len() == 1
}

pub fn list_spdx_license_names() -> Vec<String> {
    LICENSES
        .iter()
        .map(|(_, fullname, _)| fullname.to_string())
        .collect()
}

pub fn id_from_license_fullname(name: &str) -> Result<String> {
    let item = LICENSES
        .iter()
        .find(|(_, fullname, _)| name == *fullname)
        .map(|(id, fullname, _)| *id);

    if item.is_none() {
        return Err(anyhow!("no SPDX ID found for name: '{}'", name));
    }

    Ok(item.unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_find_by_id_exact() {
        let expr = "BSD-3-Clause-No-Nuclear-License";
        let license_id = try_find_by_id(expr);
        println!("{:?}", license_id);
        assert!(&license_id.is_ok());
    }

    #[test]
    fn test_try_find_by_id_invalid() {
        let expr = "BSE";
        let license_id = try_find_by_id(expr);
        assert!(&license_id.unwrap().is_none());
    }

    #[test]
    fn test_try_find_by_id_combined() {
        let expr = "mit or apache";
        let license_id = try_find_by_id(expr);
        assert!(&license_id.is_ok());
    }
}
