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

use serde::{Deserialize, Serialize};

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
struct LicenseMetadata {
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
