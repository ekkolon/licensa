// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::BorrowMut, fmt::Debug, path::PathBuf, str::Lines};

use crate::env::templates_dir;

const TEMPLATE_FILE_FORMAT: &str = "txt";

// !!!DO NOT EDIT!
const SPDX_LICENSE_DATA_REMOTE_URL: &str =
  "https://raw.githubusercontent.com/github/choosealicense.com/gh-pages/_licenses/";

#[derive(Debug, Clone)]
pub struct Template {
  pub content: String,
  pub spdx_id: String,
  pub title: String,
}

impl SpdxIdentifier for Template {
  #[inline]
  fn spdx_id(&self) -> String {
    self.spdx_id.clone()
  }
}

impl HasContent for Template {
  #[inline]
  fn content(&self) -> String {
    self.content.clone()
  }
}

pub trait TemplateRef: SpdxIdentifier {
  /// Returns the remote GitHub URL to the template file's content.
  #[inline]
  fn remote_url(&self) -> Result<url::Url, Box<dyn std::error::Error>> {
    url::Url::parse(SPDX_LICENSE_DATA_REMOTE_URL)?
      .join(&self.filename())
      .map(Result::Ok)?
  }

  /// Returns the full path of the file in the local store.
  #[inline]
  fn path(&self) -> PathBuf {
    templates_dir().join(self.filename())
  }

  /// Returns the filename for the template ref.
  #[inline]
  fn filename(&self) -> String {
    format!("{}.{}", &self.spdx_id_lower(), TEMPLATE_FILE_FORMAT)
  }
}

impl<T> TemplateRef for T where T: SpdxIdentifier {}

pub trait SpdxIdentifier {
  /// Returns the SPDX identifier for objects that implement this trait.
  fn spdx_id(&self) -> String;

  /// Returns the lowercase version SPDX identifier for objects
  /// that implement this trait.
  fn spdx_id_lower(&self) -> String {
    self.spdx_id().to_lowercase()
  }
}

impl<T> Extractor for T
where
  T: HasContent,
{
  fn extract<'a>(&self) -> Result<String, Box<dyn std::error::Error>> {
    let content = self.content();
    let mut lines = content.lines();
    let license_header = extract_license_header(lines.borrow_mut())?;
    let license_text = extract_license_text(&content)?;
    Ok(license_text)
  }
}

pub trait Interpolate {
  // FIXME: Don't allocate result
  /// Remove the newline after the last occurrence of "---".
  ///
  /// If "---" is not found, return the entire input
  fn interpolate(&self) -> Result<String, Box<dyn std::error::Error>>;
}

pub trait Extractor {
  // FIXME: Don't allocate result
  /// Remove the newline after the last occurrence of "---".
  ///
  /// If "---" is not found, return the entire input
  fn extract(&self) -> Result<String, Box<dyn std::error::Error>>;
}

pub trait HasContent {
  fn content(&self) -> String;
}

#[derive(Debug)]
pub struct LicenseHeader {
  pub title: String,
  pub spdx_id: String,
}

// Extract license metadata
fn extract_license_header(
  lines: &mut Lines,
) -> Result<LicenseHeader, Box<dyn std::error::Error>> {
  let metadata_lines: Vec<&str> = lines
    .enumerate()
    .skip(1)
    .filter(|&(i, _)| i < 3)
    .map(|(_, line)| {
      line
        .split(": ")
        .last()
        .expect("Failed to determine license header field")
    })
    .collect();

  let title = metadata_lines
    .first()
    .unwrap_or_else(|| panic!("{}", invalid_field_error("title")))
    .to_string();

  let spdx_id = metadata_lines
    .last()
    .unwrap_or_else(|| panic!("{}", invalid_field_error("spdx_id")))
    .to_string();

  Ok(LicenseHeader { title, spdx_id })
}

fn extract_license_text<T>(content: T) -> Result<String, Box<dyn std::error::Error>>
where
  T: AsRef<str>,
{
  let slice = content.as_ref();
  if let Some(last_separator) = &slice.rfind("---") {
    // NOTE: Leave whitespaces untouched.
    // Don't use methods like `.trim_start()` or `.trim_end()`.
    let result = &slice[last_separator + 5..];
    Ok(result.to_string())
  } else {
    Ok(slice.to_string())
  }
}

#[inline]
fn invalid_field_error<F>(field: F) -> String
where
  F: AsRef<str> + Debug,
{
  format!("Invalid value found for license header field {:?}", field)
}
