// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Borrow;
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use anyhow::Result;
use clap::Args;
use clap::Parser;
use colored::Colorize;
use rayon::prelude::*;
use serde::Serialize;

use crate::cache::{Cachable, Cache};
use crate::config::resolve_workspace_config;
use crate::config::Config;
use crate::config::LicensaConfig;
use crate::copyright_notice::{
    contains_copyright_notice, CompactCopyrightNotice, SpdxCopyrightNotice,
    COMPACT_COPYRIGHT_NOTICE, SPDX_COPYRIGHT_NOTICE,
};
use crate::error;
use crate::header::{extract_hash_bang, SourceHeaders};
use crate::interpolation::interpolate;
use crate::schema::{LicenseId, LicenseNoticeFormat, LicenseYear};
use crate::utils::to_elapsed_secs;
use crate::workspace::scan::{get_path_suffix, Scan, ScanConfig};
use crate::workspace::stats::ScanStats;
use crate::workspace::work_tree::{FileTaskResponse, WorkTree};

pub fn run(args: &ApplyArgs) -> Result<()> {
    // only: DEBUG
    let start_time = Instant::now();

    let root = std::env::current_dir()?;

    // ========================================================
    // Scanning process
    // ========================================================

    let scan_config = ScanConfig {
        limit: 100,
        exclude: None,
        root: root.clone(),
    };

    let scan = Scan::new(scan_config);
    let candidates: Vec<PathBuf> = scan
        .run()
        .into_iter()
        .par_bridge()
        .map(|entry| entry.abspath)
        .collect();

    let num_candidates = candidates.len();

    // ========================================================

    // ========================================================
    // File processing
    // ========================================================

    let cache = Cache::<HeaderTemplate>::new();
    let stats = Arc::new(Mutex::new(ScanStats::new()));

    let workspace_config = args.to_config()?;

    let template = resolve_license_notice_template(&workspace_config)?;
    let template = Arc::new(Mutex::new(template));

    let context = ScanContext {
        root,
        cache: cache.clone(),
        stats: stats.clone(),
        template,
    };

    let mut worktree = WorkTree::new();
    worktree.add_task(context, add_license);
    worktree.run(candidates);

    // Clear cache
    cache.clear();

    // ========================================================

    // only: DEBUG
    let num_skipped = &stats.lock().unwrap().skipped;
    let num_modified = num_candidates - num_skipped;
    let num_failed = 0;
    let result_type = "ok".green();

    let duration = to_elapsed_secs(start_time.elapsed());
    println!("\napply result: {result_type}. {num_modified} modified; {num_failed} failed; {num_skipped} ignored; finished in {duration}");

    Ok(())
}

#[derive(Parser, Debug, Serialize, Clone)]
pub struct ApplyArgs {
    /// License SPDX ID.
    #[arg(short = 't', long = "type")]
    pub license: Option<LicenseId>,

    /// The copyright owner.
    #[arg(short, long)]
    pub owner: Option<String>,

    /// The copyright year.
    #[arg(short, long)]
    pub year: Option<LicenseYear>,

    /// The copyright header format to apply on each file to be licensed.
    #[arg(
        short,
        long,
        value_enum,
        rename_all = "lower",
        requires_if("compact", "compact_info")
    )]
    pub format: Option<LicenseNoticeFormat>,

    #[command(flatten)]
    compact_template_args: CompactLicenseNoticeArgs,
}

#[derive(Debug, Args, Serialize, Clone)]
#[group(id = "compact_info", required = false, multiple = true)]
pub struct CompactLicenseNoticeArgs {
    /// The location where the LICENSE file can be found.
    ///
    /// Only takes effect in conjunction with 'compact' format.
    #[arg(long = "location")]
    #[serde(rename = "location")]
    pub license_location: Option<String>,

    /// The word that appears before the path to the license in a sentence (e.g. "in").
    ///
    /// Only takes effect in conjunction with 'compact' format.
    #[arg(long = "determiner")]
    #[serde(rename = "determiner")]
    pub license_location_determiner: Option<String>,
}

impl ApplyArgs {
    // Merge self with config::Config
    fn to_config(&self) -> Result<LicensaConfig> {
        let workspace_root = current_dir()?;
        let mut config = resolve_workspace_config(workspace_root)?;

        config.update(Config {
            license: self.license.clone(),
            owner: self.owner.clone(),
            format: self.format.clone(),
            year: self.year.clone(),
            license_location_determiner: self
                .compact_template_args
                .license_location_determiner
                .clone(),
            license_location: self.compact_template_args.license_location.clone(),
            ..Default::default()
        });

        // Verify required fields such es `license`, `owner` and `format` are set.
        config.check_required_fields();

        let args = serde_json::to_value(config);
        if let Err(err) = args.as_ref() {
            error::serialize_args_error("add", err)
        }

        let config = serde_json::from_value::<LicensaConfig>(args.unwrap());
        if let Err(err) = config.as_ref() {
            error::deserialize_args_error("add", err)
        }

        Ok(config.unwrap())
    }
}

#[derive(Clone)]
struct ScanContext {
    pub root: PathBuf,
    pub stats: Arc<Mutex<ScanStats>>,
    pub cache: Arc<Cache<HeaderTemplate>>,
    pub template: Arc<Mutex<String>>,
}

#[derive(Debug, Clone)]
struct HeaderTemplate {
    pub extension: String,
    pub template: String,
}

impl Cachable for HeaderTemplate {
    fn cache_id(&self) -> String {
        self.extension.to_owned()
    }
}

fn resolve_license_notice_template<C>(config: C) -> Result<String>
where
    C: Borrow<LicensaConfig>,
{
    let config = config.borrow() as &LicensaConfig;

    match config.format {
        LicenseNoticeFormat::Compact => interpolate!(
            COMPACT_COPYRIGHT_NOTICE,
            CompactCopyrightNotice {
                year: 2024,
                fullname: config.owner.to_string(),
                license: config.license.to_string(),
                determiner: config.license_location_determiner.clone().unwrap(),
                location: config.license_location.clone().unwrap(),
            }
        ),
        LicenseNoticeFormat::Full | LicenseNoticeFormat::Spdx => {
            interpolate!(
                SPDX_COPYRIGHT_NOTICE,
                SpdxCopyrightNotice {
                    year: 2024,
                    fullname: config.owner.to_string(),
                    license: config.license.to_string(),
                }
            )
        }
    }
}

fn add_license(context: &mut ScanContext, response: &FileTaskResponse) -> Result<()> {
    if contains_copyright_notice(&response.content) {
        // Skip further processing the file if it already contains a copyright notice
        context.stats.lock().unwrap().skip();

        return Ok(());
    }

    let header = get_header_template(context, response);
    let content = prepend_license(&header.template, &response.content);
    fs::write(&response.path, content)?;

    let file_path = &response
        .path
        .strip_prefix(&context.root)
        .unwrap()
        .to_str()
        .unwrap();

    let result_type = "ok".green();

    println!("apply {file_path} ... {result_type}");

    Ok(())
}

fn prepend_license<H, F>(header: H, file_content: F) -> Vec<u8>
where
    H: AsRef<str>,
    F: AsRef<str>,
{
    let template = header.as_ref().as_bytes().to_vec();
    let file_content = file_content.as_ref().as_bytes();
    let mut line = extract_hash_bang(file_content).unwrap_or_default();
    let mut content = file_content.to_vec();

    let line_break = b'\n';

    if !line.is_empty() {
        content = content.split_off(line.len());
        if line[line.len() - 1] != line_break {
            line.push(line_break);
        }
        content = [line, template, content].concat();
    } else {
        content = [template, content].concat();
    }

    content
}

fn get_header_template(context: &mut ScanContext, task: &FileTaskResponse) -> Arc<HeaderTemplate> {
    // FIXME: Compute cache id in FileTree
    let cache_id = get_path_suffix(&task.path);

    // Reuse cached template for this candidate
    if !context.cache.contains(&cache_id) {
        // Compile and cache template for this candidate

        let header = SourceHeaders::find_header_definition_by_extension(&cache_id).unwrap();
        let template = context.template.lock().unwrap();
        let template = template.as_str();
        let compiled_template = header.header_prefix.apply(template).unwrap();

        // FIXME: Use unique cache_id for header prefixes to prevent compiling
        // that use the same format.
        context.cache.add(HeaderTemplate {
            extension: cache_id.clone(),
            template: compiled_template,
        });
    }

    context.cache.get(&cache_id).unwrap()
}
