// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::utils::loadfile;

// Static ref reference to license metadata json file.
lazy_static! {
    static ref LICENSE_MANIFEST: LicensesManifest = loadfile!("../licenses/metadata.json");
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
    pub name: String,

    /// The SPDX identifier of the license.
    pub spdx_id: String,

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
