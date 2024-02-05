// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::config::Config;
use crate::error;
use crate::ops::scan::{get_path_suffix, Scan, ScanConfig};
use crate::ops::stats::{WorkTreeRunnerStatistics, WorkTreeRunnerStatus};
use crate::ops::work_tree::{FileTaskResponse, WorkTree};
use crate::template::cache::{Cachable, Cache};
use crate::template::copyright::SPDX_COPYRIGHT_NOTICE;
use crate::template::has_copyright_notice;
use crate::template::header::{extract_hash_bang, SourceHeaders};
use crate::workspace::LicensaWorkspace;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use rayon::prelude::*;
use serde::Serialize;

use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug, Serialize, Clone)]
pub struct ApplyArgs {
    #[command(flatten)]
    config: Config,
}

impl ApplyArgs {
    // Merge self with config::Config
    fn to_config(&self) -> Result<LicensaWorkspace> {
        let workspace_root = current_dir()?;
        let config = self.config.clone().with_workspace_config(workspace_root)?;

        // Verify required fields such es `license`, `owner` and `format` are set.
        Self::check_required_fields(&config);

        let args = serde_json::to_value(config);
        if let Err(err) = args.as_ref() {
            error::serialize_args_error("apply", err)
        }

        let config = serde_json::from_value::<LicensaWorkspace>(args.unwrap());
        if let Err(err) = config.as_ref() {
            error::deserialize_args_error("apply", err)
        }

        Ok(config.unwrap())
    }

    fn check_required_fields(config: &Config) {
        if config.license.is_none() {
            error::missing_required_arg_error("-t, --type <LICENSE>")
        }
        if config.owner.is_none() {
            error::missing_required_arg_error("-o, --owner <OWNER>")
        }
    }
}

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

    let template = Arc::new(Mutex::new(SPDX_COPYRIGHT_NOTICE.to_string()));

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

fn apply_license_notice(context: &mut ScanContext, response: &FileTaskResponse) -> Result<()> {
    // Ignore file that already contains a copyright notice
    if has_copyright_notice(response.content.as_bytes()) {
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
