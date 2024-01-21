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

use std::borrow::Borrow;
use std::{fs, sync::Arc};

use log::info;
use reqwest::Client;

use crate::env::templates_dir;
use crate::spdx::{HasContent, SpdxIdentifier, Template, TemplateRef};

pub struct LicenseStore<'a> {
  http: &'a reqwest::Client,
}

impl<'a> LicenseStore<'a> {
  /// Fetch the templates content.
  ///
  /// This operation attempts to fetch the content from the local template
  /// store first. If the template exists locally it will be returned,
  /// otherwise attempts to fetch it from the remote source.
  pub async fn fetch_template<T>(
    &self,
    license_ref: T,
  ) -> Result<impl TemplateRef, Box<dyn std::error::Error>>
  where
    T: SpdxIdentifier + Clone,
  {
    let spdx_id = &license_ref.spdx_id();

    // Check whether a SPDX license text exists locally
    let local_license = self.fetch_template_from_store(license_ref.clone());
    if let Ok(license) = local_license {
      info!("Fetched {:?} license template from local store", &spdx_id);
      return Ok(license);
    };

    let license = self.fetch_template_from_remote(license_ref.clone()).await?;
    info!("Fetched {:?} license template from remote store", &spdx_id);

    self.save_template(&license)?;
    info!("License template saved at: {:?}", license.path());

    Ok(license)
  }

  /// Fetch the templates content from the local store.
  pub fn fetch_template_from_store<T>(
    &self,
    license_ref: T,
  ) -> Result<Template, Box<dyn std::error::Error>>
  where
    T: SpdxIdentifier + Clone,
  {
    let template_path = &license_ref.path();
    let content = fs::read_to_string(template_path)?;

    Ok(Template {
      content,
      spdx_id: license_ref.spdx_id(),
      title: license_ref.spdx_id(),
    })
  }

  /// Fetch the templates content from GitHub repository.
  pub async fn fetch_template_from_remote<T>(
    &self,
    license_ref: T,
  ) -> Result<Template, Box<dyn std::error::Error>>
  where
    T: SpdxIdentifier + Clone,
  {
    let url = license_ref.remote_url()?;
    let response = self.http.get(url).send().await?;
    let content = response.text().await?;

    Ok(Template {
      title: license_ref.spdx_id(),
      spdx_id: license_ref.spdx_id(),
      content,
    })
  }

  /// Save provided content for this template ref.
  pub fn save_template<T>(&self, template: &T) -> Result<(), Box<dyn std::error::Error>>
  where
    T: Borrow<Template> + HasContent + TemplateRef,
  {
    // Create templates directory if it doesn't exist
    fs::create_dir_all(templates_dir())?;

    // Write raw license template to
    fs::write(template.path(), template.content())?;

    Ok(())
  }

  #[inline]
  pub fn new(http_client: &'a Client) -> Arc<Self> {
    Arc::new(Self { http: http_client })
  }
}

#[derive(Debug, Clone)]
pub struct LicenseRef {
  spdx_id: String,
}

impl LicenseRef {
  #[inline]
  pub fn new<T>(spdx_id: T) -> Self
  where
    T: AsRef<str>,
  {
    Self {
      spdx_id: spdx_id.as_ref().to_string(),
    }
  }
}

impl SpdxIdentifier for LicenseRef {
  #[inline]
  fn spdx_id(&self) -> String {
    self.spdx_id.clone()
  }
}
