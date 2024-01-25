// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tabled::{settings::Settings, Table, Tabled};

// Static ref reference to license metadata json file.
lazy_static! {
  static ref LICENSE_MANIFEST: LicensesManifest =
    serde_json::from_slice::<LicensesManifest>(include_bytes!(
      "../licenses/metadata.json"
    ))
    .unwrap();
}

/// Represents license metadata.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicensesManifest {
  /// The SPDX identifier of the license.
  ids: Vec<String>,

  /// The SPDX identifier of the license in lowercase.
  licenses: Vec<LicenseMetadata>,
}

impl LicensesManifest {
  pub fn ids<'a>() -> &'a Vec<String> {
    &LICENSE_MANIFEST.ids
  }

  pub fn licenses<'a>() -> &'a Vec<LicenseMetadata> {
    &LICENSE_MANIFEST.licenses
  }

  pub fn spdx_ids() -> Vec<String> {
    LicensesManifest::licenses()
      .iter()
      .map(|license| license.spdx_id.to_string())
      .collect::<Vec<String>>()
  }

  pub fn print_license_table() {
    println!("{}", LicensesManifest::table())
  }

  fn table() -> Table {
    let table_config = Settings::default()
      .with(tabled::settings::Panel::header("Available SPDX Licenses"))
      .with(tabled::settings::Padding::new(1, 1, 0, 0))
      .with(tabled::settings::Style::modern_rounded());

    let table_items: Vec<LicenseMetadataTableItem> = LicensesManifest::licenses()
      .iter()
      .map(|license| license.to_table_item())
      .collect();

    let mut table = Table::new(table_items);

    table.with(table_config);

    table.modify(
      tabled::settings::object::Rows::first(),
      tabled::settings::Height::increase(2),
    );

    table
      .modify((0, 0), tabled::settings::Span::column(2))
      .modify(
        tabled::settings::object::Rows::first(),
        tabled::settings::Alignment::center(),
      )
      .modify(
        tabled::settings::object::Columns::last(),
        tabled::settings::Alignment::center(),
      )
      .to_owned()
  }
}

/// Represents license metadata.
#[derive(Debug, Serialize, Deserialize)]
struct License {
  /// License metadata
  #[serde(flatten)]
  pub metadata: LicenseMetadata,

  /// The URL to the license template.
  pub template_text: String,

  /// The URL to the license template.
  pub template_header: String,
}

/// Represents license metadata.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseMetadata {
  /// The name of the license.
  name: String,

  /// The SPDX identifier of the license.
  spdx_id: String,

  /// The SPDX identifier of the license in lowercase.
  spdx_id_lower: String,

  /// An optional nickname for the license.
  nickname: Option<String>,

  /// Indicates whether the license has a header.
  has_header: bool,

  /// The URL to the license template.
  template_url: String,

  /// Additional fields associated with the license.
  fields: Vec<String>,
}

impl LicenseMetadata {
  pub fn to_table_item(&self) -> LicenseMetadataTableItem {
    // let nickname = self.nickname.clone().unwrap_or("-".to_string()).to_string();

    LicenseMetadataTableItem {
      name: self.name.to_string(),
      spdx_id: self.spdx_id.to_string(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
#[serde(rename_all = "camelCase")]
#[tabled(rename_all = "UPPERCASE")]
pub struct LicenseMetadataTableItem {
  /// The name of the license.
  name: String,

  /// The SPDX identifier of the license.
  #[tabled(rename = "SPDX ID")]
  spdx_id: String,
}

fn default_table_item_nickname() -> String {
  "-".to_string()
}
