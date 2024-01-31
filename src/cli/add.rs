// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use crate::validator;
use clap::ValueEnum;
use clap::{Args, Parser};

#[derive(Parser, Debug)]
pub struct AddArgs {
    #[command(flatten)]
    info: LicenseInfoArgs,

    /// The copyright header format to apply on each file to be licensed.
    #[arg(
        short,
        long,
        value_enum,
        rename_all = "lower",
        requires_if("compact", "compact_info")
    )]
    pub format: CopyrightFormat,

    #[command(flatten)]
    compact_info: Option<CompactCopyrightNoticeArgs>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CopyrightFormat {
    /// A short, four lines long format that refers users to the
    /// the location at which the full license file is found.
    ///
    /// *Remarks*:
    ///
    /// The location can be either a path to a file within the
    /// repository or an URL.
    Compact,

    /// Render the full license header.
    ///
    /// *Remarks*:
    /// This option only applies to licenses that provide a license header
    /// (e.g. Apache-2.0 or 0BSD). In cases where no license header is available
    /// this fallbacks to the **SPDX** format, or if specified, the `fallback`
    /// format option that can be specified in the *generator* config.
    Full,

    /// Based on the SPDX license format and rendered in two lines.
    Spdx,
}

#[derive(Args, Debug, Clone)]
#[group(required = false, id = "compact_info")]
pub struct CompactCopyrightNoticeArgs {
    /// The word that appears before the path to the license in a sentence (e.g. "in").
    // #[arg(default_value = "in")]
    #[arg(long, required = false)]
    pub determiner: String,

    /// The location where the LICENSE file can be found.
    // #[arg(default_value = "the root of this project")]
    #[arg(long, required = false)]
    pub location: String,
}

#[derive(Args, Debug)]
pub struct LicenseInfoArgs {
    /// License SPDX ID.
    #[arg(short = 't', long = "type")]
    pub license: Option<String>,

    /// The copyright owner.
    #[arg(short, long)]
    pub owner: Option<String>,

    /// The copyright year.
    #[arg(short, long, value_parser = validator::acceptable_year)]
    pub year: Option<u16>,
}
