// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

//! # Interpolation
//!
//! The `interpolation` module provides functions for template interpolation, allowing the dynamic
//! insertion of values into a template string. It includes utilities for extracting template variables,
//! resolving interpolation maps, and replacing template variables with corresponding values.

use std::borrow::Borrow;

use anyhow::{anyhow, Ok, Result};
use regex::Regex;
use serde::Serialize;
use serde_json::{Map, Value};

const TEMPLATE_VARIABLE_REGEX_PATTERN: &str = r"\$\((\w+)\)";

pub trait Interpolate {
    fn interpolate(&self) -> Result<String>;
}

/// Interpolates template variables in the provided template string using the given values.
/// If the template has no variables, the original template string is returned unchanged.
///
/// # Arguments
///
/// * `template: &'a T` - The template string to interpolate variables in.
/// * `values: F` - Values to use for interpolation. Must be a type implementing `Serialize`.
///
/// # Returns
///
/// A `Result` containing the interpolated string if successful, or an error if interpolation fails.
///
/// # Errors
///
/// This macro returns an error if there is an issue with serialization or if the template
/// contains invalid variable placeholders.
///
/// # Panics
///
/// The `interpolate!` macro does not panic under normal circumstances. Panics may occur if the provided
/// template is not a valid UTF-8 string.
macro_rules! interpolate {
    ($template:expr, $values:expr) => {{
        $crate::template::interpolation::__private_interpolate_template(&$template, &$values)
    }};
}
pub(crate) use interpolate;

/// Interpolates template variables in the provided template string using the given values.
/// If the template has no variables, the original template string is returned unchanged.
///
/// # Arguments
///
/// * `template: &'a T` - The template string to interpolate variables in.
/// * `values: F` - Values to use for interpolation. Should be a type implementing `Serialize`.
///
/// # Returns
///
/// A `Result` containing the interpolated string if successful, or an error if interpolation fails.
///
/// # Errors
///
/// This function returns an error if there is an issue with serialization or if the template
/// contains invalid variable placeholders.
#[doc(hidden)]
pub fn __private_interpolate_template<'a, T, F>(template: &'a T, values: F) -> Result<String>
where
    T: AsRef<str> + 'a + ?Sized,
    F: Serialize,
{
    let fields = extract_template_variables(template);
    if fields.is_empty() {
        // No template variables to interpolate. Leave provided template untouched.
        return Ok(template.as_ref().to_string());
    }

    let replacements = resolve_interpolation_map(fields, values)?;
    let template = replace_template_variables(&template, &replacements);

    Ok(template)
}

/// Extracts template variables from the provided template string
/// and returns a vector of variable names.
///
/// # Arguments
///
/// * `template: &'a T` - The template string to extract variables from.
/// * `T: AsRef<str> + 'a` - The type of the template string.
///   The lifetime `'a` must be at least as long as the lifetime of the template string.
///
/// # Returns
///
/// A vector of references to the variable names found in the template string.
/// The references are valid for the lifetime `'a`.
fn extract_template_variables<'a, T>(template: &'a T) -> Vec<&'a str>
where
    T: AsRef<str> + 'a + ?Sized,
{
    // Find matching template variables in the provided template.
    let regex = Regex::new(TEMPLATE_VARIABLE_REGEX_PATTERN).unwrap();
    let matches = regex.captures_iter(template.as_ref());

    let mut vars: Vec<&'a str> = vec![];

    // Find all matches in the constant string
    for cap in matches {
        // Extract the variable name from the captured group
        if let Some(variable_name) = cap.get(1) {
            vars.push(variable_name.as_str())
        }
    }

    vars
}

/// Replace template variables in the provided template string using the given replacements.
///
/// # Arguments
///
/// * `template: &'a T` - The template string to replace variables in.
/// * `replacements: &[(key: &str, value: &str)]` - A slice of key-value pairs for replacements.
/// * `T: AsRef<str> + 'a` - The type of the template string.
///   The lifetime `'a` must be at least as long as the lifetime of the template string.
///
/// # Returns
///
/// A new String with template variables replaced.
fn replace_template_variables<'a, T>(template: &'a T, replacements: &Map<String, Value>) -> String
where
    T: AsRef<str> + 'a + ?Sized,
{
    let mut result = template.as_ref().to_owned();
    for (key, value) in replacements.iter() {
        let pattern = format!(r"\$\({}\)", regex::escape(key.borrow()));
        let replacement_value = normalize_replacement_value(value);
        let regex = Regex::new(&pattern).unwrap();
        result = regex.replace_all(&result, &replacement_value).to_string();
    }

    result
}

fn resolve_interpolation_map<T>(fields: Vec<&str>, values: T) -> Result<Map<String, Value>>
where
    T: Serialize,
{
    let value = serde_json::to_value(&values)?;
    if !value.is_object() {
        return Err(anyhow!(
            "Failed to interpolate template. Provided fields must be an object"
        ));
    }

    let mut interpolation_map = Map::new();

    // Check if provided map contains fields that match those specified
    // in the provided template.
    let replacements = value.as_object().unwrap();
    for field in fields.iter() {
        if !replacements.contains_key(&field.to_string()) {
            return Err(anyhow!(
                "Failed to interpolate template. Missing required key \"{field}\""
            ));
        }

        let value = replacements.get(&field.to_string()).unwrap();
        if !is_interpolatable_value(value) {
            return Err(anyhow!(
        "Failed to interpolate template. Field \"{field}\" is neither a string nor a number"
      ));
        }

        interpolation_map.insert(field.to_string(), value.clone());
    }

    Ok(interpolation_map)
}

#[inline]
fn normalize_replacement_value<T>(value: T) -> String
where
    T: Borrow<Value>,
{
    match value.borrow() {
        Value::String(s) => s.clone(),
        _ => serde_json::to_string(value.borrow()).unwrap(),
    }
}

#[inline]
fn is_interpolatable_value<T>(value: T) -> bool
where
    T: Borrow<Value>,
{
    value.borrow().is_string() || value.borrow().is_number()
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn test_interpolate_template() {
        // Test with no template variables
        let template = "Hello, World!";
        let result = __private_interpolate_template(template, json!({})).unwrap();
        assert_eq!(result, template);

        // Test with valid template variables
        let template = "Hello, $(name)!";
        let values = json!({
            "name": "Alice"
        });
        let result = __private_interpolate_template(template, values).unwrap();
        assert_eq!(result, "Hello, Alice!");

        // Test with multiple template variables
        let template = "$(greeting), $(name)!";
        let values = json!({
            "greeting": "Hi",
            "name": "Alice"
        });
        let result = __private_interpolate_template(template, values).unwrap();
        assert_eq!(result, "Hi, Alice!");

        // Test with escaped characters in the template
        let template = "Escape \\$\\(me\\)!";
        let result = __private_interpolate_template(template, json!({})).unwrap();
        assert_eq!(result, template);

        // Test with missing required key in values
        let template = "$(name) says $(greeting)!";
        let values = json!({
            "name": "Bob"
        });
        assert!(__private_interpolate_template(template, values).is_err());

        // Test with non-string or non-number field value in values
        let template = "$(name) is $(age) years old.";
        let values = json!({
            "name": "Alice",
            "age": true
        });
        assert!(__private_interpolate_template(template, values).is_err());
    }

    #[test]
    fn test_resolve_interpolation_map() {
        // Test with valid input
        let fields = vec!["name", "age"];
        let values = json!({
            "name": "Bob",
            "age": 30
        });
        assert_eq!(
            resolve_interpolation_map(fields, values).unwrap(),
            json!({
                "name": "Bob",
                "age": 30
            })
            .as_object()
            .unwrap()
            .clone()
        );

        // Test with missing required key
        let fields = vec!["name", "age"];
        let values = json!({
            "name": "Bob"
        });
        assert!(resolve_interpolation_map(fields, values).is_err());

        // Test with non-string or non-number field value
        let fields = vec!["name", "age"];
        let values = json!({
            "name": "Bob",
            "age": true
        });
        assert!(resolve_interpolation_map(fields, values).is_err());

        // Test with non-object input
        let fields = vec!["name", "age"];
        let values = json!(["Bob", 30]);
        assert!(resolve_interpolation_map(fields, values).is_err());
    }

    #[test]
    fn test_replace_template_variabless() {
        // Test when there are no replacements
        let template = "Hello, World!";
        let replacements = Map::new();
        assert_eq!(
            replace_template_variables(template, &replacements),
            "Hello, World!"
        );

        // Test with replacements
        let template = "Hello, $(name)!";
        let mut replacements = Map::new();
        replacements.insert("name".to_string(), json!("Alice"));
        assert_eq!(
            replace_template_variables(template, &replacements),
            "Hello, Alice!"
        );

        // Test with multiple replacements
        let template = "$(greeting), $(name)!";
        replacements.insert("greeting".to_string(), json!("Hi"));
        assert_eq!(
            replace_template_variables(template, &replacements),
            "Hi, Alice!"
        );

        // Test with escaped characters in the template
        let template = "Escape \\$\\(me\\)!";
        assert_eq!(
            replace_template_variables(template, &replacements),
            template
        );

        // Test with special characters in the replacement
        let template = "$(name) says $(special)!";
        replacements.insert("special".to_string(), json!("Hello & Goodbye"));
        assert_eq!(
            replace_template_variables(template, &replacements),
            "Alice says Hello & Goodbye!"
        );
    }

    #[test]
    fn extract_template_variables_empty_string() {
        let template = "";
        let vars = extract_template_variables(&template);

        assert!(vars.is_empty());
    }

    #[test]
    fn extract_template_variables_single_variable() {
        let template = "Hello, $(name)!";
        let vars = extract_template_variables(&template);

        assert_eq!(vars, ["name"]);
    }

    #[test]
    fn extract_template_variables_multiple_variables() {
        let template = "Hello, $(name)! How are you, $(age)?";
        let vars = extract_template_variables(&template);

        assert_eq!(vars, ["name", "age"]);
    }

    #[test]
    fn extract_template_variables_no_variables() {
        let template = "Hello!";
        let vars = extract_template_variables(&template);

        assert!(vars.is_empty());
    }
}
