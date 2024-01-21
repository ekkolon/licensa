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

use std::{fs, path::PathBuf, sync::Arc};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::env::templates_dir;

use super::{error::SpdxError, Result};

const LICENSE_METADATA_FILE_FORMAT: &str = "json";

// Initialize reference to license metadata json file.
lazy_static! {
  // TODO: Improve error handling
  static ref LICENSES_METADATA: SpdxLicensesMetadata =
    serde_json::from_slice::<SpdxLicensesMetadata>(include_bytes!(
      "../../licenses.json"
    ))
    .unwrap();
}

pub struct SpdxLicenseStore<'a> {
  http: &'a reqwest::Client,
}

impl<'a> SpdxLicenseStore<'a> {
  /// Fetch the templates content from GitHub repository.
  pub async fn fetch_license_details<T>(&self, spdx_id: T) -> Result<SpdxLicenseDetails>
  where
    T: AsRef<str>,
  {
    self.fetch_local_license_details(&spdx_id).await?;
    let SpdxLicenseMetadata { details_url, .. } = Self::get_license_metadata(&spdx_id)?;
    let res = self.http.get(details_url).send().await?;
    let body = res.bytes().await?;
    let data = serde_json::from_slice::<SpdxLicenseDetails>(&body)?;
    Ok(data)
  }

  /// Fetch the templates content from GitHub repository.
  pub async fn fetch_local_license_details<T>(
    &self,
    spdx_id: T,
  ) -> Result<SpdxLicenseDetails>
  where
    T: AsRef<str>,
  {
    let path = Self::get_license_path(&spdx_id);
    let content = fs::read_to_string(path)?;
    let data = serde_json::from_str::<SpdxLicenseDetails>(&content)?;
    Ok(data)
  }

  /// Fetch the templates content from GitHub repository.
  pub async fn fetch_remote_license_details<T>(
    &self,
    spdx_id: T,
  ) -> Result<SpdxLicenseDetails>
  where
    T: AsRef<str>,
  {
    let SpdxLicenseMetadata { details_url, .. } = Self::get_license_metadata(&spdx_id)?;
    let res = self.http.get(details_url).send().await?;
    let body = res.bytes().await?;
    let data = serde_json::from_slice::<SpdxLicenseDetails>(&body)?;
    Ok(data)
  }

  pub fn get_license_metadata<T>(spdx_id: T) -> Result<SpdxLicenseMetadata>
  where
    T: AsRef<str>,
  {
    let license_id = spdx_id.as_ref().to_string().to_lowercase();
    for license in &LICENSES_METADATA.licenses {
      let item_id_lower = license.item.license_id.to_lowercase();
      if item_id_lower == license_id {
        return Ok(license.clone());
      }
    }

    Err(SpdxError::NotFound(license_id))
  }

  #[inline]
  pub fn new(http_client: &'a reqwest::Client) -> Arc<Self> {
    Arc::new(Self { http: http_client })
  }

  /// Returns the full path of the file in the local store.
  #[inline]
  fn get_license_path<T>(spdx_id: T) -> PathBuf
  where
    T: AsRef<str>,
  {
    templates_dir().join(Self::get_license_filename(&spdx_id))
  }

  /// Returns the filename for the template ref.
  #[inline]
  fn get_license_filename<T>(spdx_id: T) -> String
  where
    T: AsRef<str>,
  {
    format!(
      "{}.{}",
      &spdx_id.as_ref().to_lowercase(),
      LICENSE_METADATA_FILE_FORMAT
    )
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpdxLicensesMetadata {
  pub license_list_version: String,
  pub licenses: Vec<SpdxLicenseMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpdxLicenseDetails {
  #[serde(flatten)]
  pub item: SpdxLicenseItem,
  pub is_fsf_libre: bool,
  pub license_text: String,
  pub license_text_html: String,
  pub license_comments: Option<String>,
  pub standard_license_template: String,
  pub standard_license_header: Option<String>,
  pub standard_license_header_template: Option<String>,
  pub standard_license_header_html: Option<String>,
  pub cross_ref: Vec<SpdxLicenseDetailsCrossRef>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpdxLicenseDetailsCrossRef {
  #[serde(rename = "match")]
  pub matches: String,
  pub url: Url,
  pub is_valid: bool,
  pub is_live: bool,
  pub timestamp: String,
  pub is_way_back_link: bool,
  pub order: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpdxLicenseMetadata {
  #[serde(flatten)]
  pub item: SpdxLicenseItem,
  pub details_url: Url,
  pub reference_number: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpdxLicenseItem {
  pub is_deprecated_license_id: bool,
  pub name: String,
  pub license_id: String,
  pub see_also: Vec<Url>,
  pub is_osi_approved: bool,
}
