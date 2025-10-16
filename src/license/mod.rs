// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

mod error;
mod id;
mod lookup;
mod period;
pub mod template;

pub use error::*;

pub use id::LicenseId;
pub use lookup::{id_from_license_fullname, list_spdx_license_names};
pub use period::LicensePeriod;

/// Resolved licensa configuration
pub struct License {
    pub id: LicenseId,
    pub owner: String,
    pub period: LicensePeriod,
}
