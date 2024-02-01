// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use crate::cache::{Cachable, Cache};
use crate::config::{resolve_workspace_config, Config, LicensaConfig};
use crate::copyright_notice::{
    contains_copyright_notice, CompactCopyrightNotice, SpdxCopyrightNotice,
    COMPACT_COPYRIGHT_NOTICE, SPDX_COPYRIGHT_NOTICE,
};
use crate::error;
use crate::header::{extract_hash_bang, SourceHeaders};
use crate::interpolation::interpolate;
use crate::schema::{LicenseId, LicenseNoticeFormat, LicenseYear};
use crate::workspace::scan::{get_path_suffix, Scan, ScanConfig};
use crate::workspace::stats::{WorkTreeRunnerStatistics, WorkTreeRunnerStatus};
use crate::workspace::work_tree::{FileTaskResponse, WorkTree};

use anyhow::Result;
use clap::{Args, Parser};
use colored::Colorize;
use rayon::prelude::*;
use serde::Serialize;

use std::borrow::Borrow;
use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub fn run(args: &ApplyArgs) -> Result<()> {
    let mut runner_stats = WorkTreeRunnerStatistics::new("apply", "modified");

    let workspace_root = std::env::current_dir()?;
    let workspace_config = args.to_config()?;

    // ========================================================
    // Scanning process
    // ========================================================
    let candidates = scan_workspace(&workspace_root)?;

    runner_stats.set_items(candidates.len());

    // ========================================================
    // File processing
    // ========================================================
    let runner_stats = Arc::new(Mutex::new(runner_stats));
    let cache = Cache::<HeaderTemplate>::new();

    let template = resolve_license_notice_template(workspace_config)?;
    let template = Arc::new(Mutex::new(template));

    let context = ScanContext {
        root: workspace_root,
        cache: cache.clone(),
        runner_stats: runner_stats.clone(),
        template,
    };

    let mut worktree = WorkTree::new();
    worktree.add_task(context, apply_license_notice);
    worktree.run(candidates);

    // ========================================================
    // Clear cache
    cache.clear();

    // Print output statistics
    let mut runner_stats = runner_stats.lock().unwrap();
    runner_stats.set_status(WorkTreeRunnerStatus::Ok);
    runner_stats.print(true);

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
    pub runner_stats: Arc<Mutex<WorkTreeRunnerStatistics>>,
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

// FIXME: Refactor to more generic, re-usable fn
fn scan_workspace<P>(workspace_root: P) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let scan_config = ScanConfig {
        // FIXME: Add limit to workspace config
        limit: 100,
        // FIXME: Use exclude from workspace config
        exclude: None,
        root: workspace_root.as_ref().to_path_buf(),
    };

    let scan = Scan::new(scan_config);

    let candidates: Vec<PathBuf> = scan
        .run()
        .into_iter()
        .par_bridge()
        .map(|entry| entry.abspath)
        .collect();

    Ok(candidates)
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

fn apply_license_notice(context: &mut ScanContext, response: &FileTaskResponse) -> Result<()> {
    // Ignore file that already contains a copyright notice
    if contains_copyright_notice(&response.content) {
        context.runner_stats.lock().unwrap().add_ignore();
        return Ok(());
    }

    let header = resolve_header_template(context, response);
    let content = prepend_license_notice(&header.template, &response.content);
    fs::write(&response.path, content)?;

    let file_path = &response
        .path
        .strip_prefix(&context.root)
        .unwrap()
        .to_str()
        .unwrap();

    // Capture task success
    context.runner_stats.lock().unwrap().add_action_count();

    print_task_success(file_path);

    Ok(())
}

fn prepend_license_notice<H, F>(header: H, file_content: F) -> Vec<u8>
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

fn resolve_header_template(
    context: &mut ScanContext,
    task: &FileTaskResponse,
) -> Arc<HeaderTemplate> {
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

fn print_task_success<P>(path: P)
where
    P: AsRef<Path>,
{
    let result_type = "ok".green();
    println!("apply {} ... {result_type}", path.as_ref().display())
}
