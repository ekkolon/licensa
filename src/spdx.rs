// Copyright 2024 Nelson Dominguez
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use reqwest::Client;
use std::{
  fs,
  io::{self, BufRead},
  path::PathBuf,
  process,
  string::ParseError,
};

use crate::env::templates_dir;

const TEMPLATE_FILE_FORMAT: &str = "txt";

/// Verify whether a license template exists in local Data directory
/// for the given SPDX identifier.
pub fn confirm_remote_fetch<T>(spdx_id: T) -> io::Result<String>
where
  T: AsRef<str>,
{
  let template = fetch_local_template(spdx_id);
  match template {
    Err(err) => {
      println!("License template does not exist locally, would you like to download it? \n[y/N]");

      let mut input = String::new();
      match io::stdin().lock().lines().next() {
        Some(line) => input = line.unwrap(),
        None => println!("\n"),
      };

      if input.to_lowercase() != "y" {
        println!("Confirmation not received. Exiting.");
        process::exit(1)
      } else {
        Ok(input)
      }
    }
    Ok(template) => Ok(template),
  }
}

// !!!DO NOT EDIT!
const SPDX_LICENSE_DATA_REMOTE_URL: &str =
  "https://raw.githubusercontent.com/github/choosealicense.com/gh-pages/_licenses";

pub async fn fetch_remote_template<T>(spdx_id: T) -> Result<(), Box<dyn std::error::Error>>
where
  T: AsRef<str>,
{
  // Check whether a SPDX license text exists locally
  let license_text = fetch_local_template(&spdx_id);
  if let Ok(license_text) = license_text {
    // License template from local store
    println!(
      "Fetched license template from local store:\n\n{}",
      license_text
    );

    return Ok(());
  };

  // Fetch template from Github repo
  let url = get_remote_template_url(&spdx_id);
  let client = Client::new();
  let response = client.get(url).send().await.unwrap();
  let body = response.text().await.unwrap();
  let license_text = extract_license_text(&body);

  println!(
    "Fetched license template from remote store:\n\n{}",
    license_text
  );

  // Save template to local store
  save_license_text(&spdx_id, &license_text)?;

  Ok(())
}

// FIXME: Don't allocate result
fn extract_license_text(input: &str) -> String {
  if let Some(last_separator) = input.rfind("---") {
    // Remove the newline after the last occurrence of ---
    let result = input[last_separator + 3..].trim_start();
    String::from(result)
  } else {
    // If --- is not found, return the entire input
    String::from(input)
  }
}

fn get_remote_template_url<T>(spdx_id: T) -> String
where
  T: AsRef<str>,
{
  let filename = make_template_filename(&spdx_id);
  format!("{SPDX_LICENSE_DATA_REMOTE_URL}/{}", filename)
}

/// Verify whether a license template exists in local Data directory
/// for the given SPDX identifier.
pub fn fetch_local_template<T>(spdx_id: T) -> io::Result<String>
where
  T: AsRef<str>,
{
  let template_path = get_template_path(spdx_id);
  fs::read_to_string(template_path)
}

#[derive(Debug, Clone)]
pub struct Template<'a> {
  content: &'a str,
  spdx_id: String,
  title: String,
  nickname: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SpdxTemplateRef {
  pub spdx_id: String,
}

impl TemplateRef for SpdxTemplateRef {
  fn spdx_id(&self) -> String {
    self.spdx_id.clone()
  }
}

impl<'a> TemplateRef for Template<'a> {
  fn spdx_id(&self) -> String {
    self.spdx_id.clone()
  }
}

impl<'a> Template<'a> {}

#[derive(Debug, Clone)]
pub struct RemoteTemplateSnapshot<'a> {
  content: &'a str,
  spdx_id: String,
  title: String,
  nickname: Option<String>,
}

impl TemplateRef for RemoteTemplateSnapshot<'_> {
  fn spdx_id(&self) -> String {
    self.spdx_id.clone()
  }
}

impl<'a> RemoteTemplateSnapshot<'a> {
  /// Save provided content for this template ref.
  pub fn save(&self) -> io::Result<()> {
    let template_path = &self.path();
    // Create templates directory if it doesn't exist
    fs::create_dir_all(templates_dir())?;
    fs::write(template_path.as_path(), self.content.as_bytes()).unwrap_or_else(|err| {
      panic!(
        "\nFailed to write License template for the {} license: {}",
        &self.spdx_id, err
      )
    });
    println!(
      "\nSuccessfully saved \"{}\" license",
      &template_path.display()
    );

    Ok(())
  }

  // FIXME: Don't allocate result
  pub fn text(&self) -> &'_ str {
    if let Some(last_separator) = &self.content.rfind("---") {
      // Remove the newline after the last occurrence of ---
      let result = &self.content[last_separator + 3..].trim_start();
      result
    } else {
      // If --- is not found, return the entire input
      self.content
    }
  }
}

pub struct TemplateStore<'a> {
  http: &'a Client,
}

impl<'a> TemplateStore<'a> {
  /// Fetch the templates content.
  ///
  /// This operation attempts to fetch the content from the local template
  /// store first. If the template exists locally it will be returned,
  /// otherwise attempts to fetch it from the remote source.
  pub async fn fetch<T>(
    &self,
    template_ref: T,
  ) -> Result<Box<dyn TemplateRef>, Box<dyn std::error::Error>>
  where
    T: TemplateRef + Clone,
  {
    let spdx_id = &template_ref.spdx_id();
    // Check whether a SPDX license text exists locally
    let local_license = self.fetch_from_store(template_ref.clone());
    if let Ok(license_text) = local_license {
      println!("Fetched \"{}\" license from local store", &spdx_id);

      return Ok::<Template>(Template {
        content: &license_text,
        nickname: None,
        title: spdx_id.to_string(),
        spdx_id: spdx_id.to_string(),
      });
    };

    let remote_license = self.fetch_from_remote(template_ref.clone()).await?;
    println!("Fetched \"{}\" license from remote store", &spdx_id);

    Ok(RemoteTemplateSnapshot {
      content: &remote_license,
      nickname: None,
      title: spdx_id.to_string(),
      spdx_id: spdx_id.to_string(),
    })
  }

  /// Fetch the templates content from the local store.
  pub fn fetch_from_store<T>(&self, template_ref: T) -> io::Result<String>
  where
    T: TemplateRef + Clone,
  {
    let template_path = &template_ref.path();
    fs::read_to_string(template_path)
  }

  /// Fetch the templates content from GitHub repository.
  pub async fn fetch_from_remote<T>(
    &self,
    template_ref: T,
  ) -> Result<String, Box<dyn std::error::Error>>
  where
    T: TemplateRef + Clone,
  {
    let url = template_ref.remote_url()?;
    let response = self.http.get(url).send().await?;
    let content = response.text().await?;
    Ok(content)
  }

  pub fn new(http_client: &'a Client) -> Self {
    Self { http: http_client }
  }
}

pub fn save_license_text<I, T>(spdx_id: I, contents: T) -> io::Result<()>
where
  I: AsRef<str>,
  T: AsRef<str>,
{
  let template_path = get_template_path(&spdx_id);
  // Create templates directory if it doesn't exist
  fs::create_dir_all(templates_dir())?;
  fs::write(template_path.as_path(), contents.as_ref().as_bytes()).unwrap_or_else(|err| {
    panic!(
      "\nFailed to write License template for the {} license: {}",
      &spdx_id.as_ref().to_string(),
      err
    )
  });
  println!(
    "\nSuccessfully saved \"{}\" license",
    &template_path.display()
  );

  Ok(())
}

pub fn get_template_path<T>(spdx_id: T) -> PathBuf
where
  T: AsRef<str>,
{
  let filename = make_template_filename(&spdx_id);
  templates_dir().join(filename)
}

fn make_template_filename<T>(spdx_id: T) -> String
where
  T: AsRef<str>,
{
  format!("{}.txt", &spdx_id.as_ref().to_lowercase())
}

pub trait TemplateRef {
  /// Returns the filename for the template ref.
  fn spdx_id(&self) -> String;

  /// Returns the remote GitHub URL to the template file's content.
  fn remote_url(&self) -> Result<url::Url, Box<dyn std::error::Error>> {
    let url = url::Url::parse(SPDX_LICENSE_DATA_REMOTE_URL)?.join(&self.filename())?;
    Ok(url)
  }

  /// Returns the full path of the file in the local store.
  fn path(&self) -> PathBuf {
    let filename = self.filename();
    templates_dir().join(filename)
  }

  /// Returns the filename for the template ref.
  fn filename(&self) -> String {
    let filename = &self.spdx_id().to_lowercase();
    format!("{}.{}", filename, TEMPLATE_FILE_FORMAT)
  }
}
