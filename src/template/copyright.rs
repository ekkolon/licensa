// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::interpolation::{interpolate, Interpolate};
use crate::utils::current_year;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Represents a simple SPDX copyright notice.
pub const SPDX_COPYRIGHT_NOTICE: &str = r#"Copyright{{#if year}} {{year}}{{/if}} {{owner}}
SPDX-License-Identifier: {{license}}"#;

/// Holds information for a simple SPDX copyright notice.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SpdxCopyrightNotice {
    /// The full name of the copyright holder.
    pub fullname: String,

    /// The type of license governing the use of the source code.
    pub license: String,

    /// The year(s) to be included in the copyright notice.
    #[serde(default = "current_year")]
    pub year: u16,
}

impl Interpolate for SpdxCopyrightNotice {
    fn interpolate(&self) -> Result<String> {
        interpolate!(SPDX_COPYRIGHT_NOTICE, &self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_spdx_template_without_year() {
        let reg = handlebars::Handlebars::new();

        let data = json!({
            "owner": "Bilbo Baggins",
            "license": "MIT"
        });

        let expected_template = "Copyright Bilbo Baggins\nSPDX-License-Identifier: MIT";

        let template = reg.render_template(SPDX_COPYRIGHT_NOTICE, &data);

        assert!(template.is_ok());
        assert_eq!(template.unwrap(), expected_template.to_string());
    }

    #[test]
    fn test_spdx_template_with_year() {
        let reg = handlebars::Handlebars::new();

        let data = json!({
            "owner": "Gandalf",
            "license": "MIT OR Apache-2.0",
            "year": 2012
        });

        let expected_template =
            "Copyright 2012 Gandalf\nSPDX-License-Identifier: MIT OR Apache-2.0";

        let template = reg.render_template(SPDX_COPYRIGHT_NOTICE, &data);

        assert!(template.is_ok());
        assert_eq!(template.unwrap(), expected_template.to_string());
    }
}
