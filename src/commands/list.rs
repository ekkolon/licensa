// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use clap::Args;
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

use crate::license::{LicenseMetadata, LicensesManifest};

#[derive(Args, Debug)]
pub struct ListArgs;

pub fn run(args: &ListArgs) {
    println!("{}", table())
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

#[inline]
fn default_table_item_nickname() -> String {
    "-".to_string()
}

fn table() -> Table {
    let table_config = tabled::settings::Settings::default()
        .with(tabled::settings::Panel::header("Available SPDX Licenses"))
        .with(tabled::settings::Padding::new(1, 1, 0, 0))
        .with(tabled::settings::Style::modern_rounded());

    let table_items: Vec<LicenseMetadataTableItem> = LicensesManifest::licenses()
        .iter()
        .map(to_table_item)
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

#[inline]
fn to_table_item(item: &LicenseMetadata) -> LicenseMetadataTableItem {
    LicenseMetadataTableItem {
        name: item.name.to_string(),
        spdx_id: item.spdx_id.to_string(),
    }
}
