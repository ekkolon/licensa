// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use clap::Args;
use colored::Colorize;
use rayon::prelude::*;

use crate::cache::{Cachable, Cache};
use crate::copyright_notice::{
    contains_copyright_notice, CompactCopyrightNotice, COMPACT_COPYRIGHT_NOTICE,
};
use crate::header::{extract_hash_bang, SourceHeaders};
use crate::helpers::channel_duration::ChannelDuration;
use crate::interpolation::interpolate;
use crate::logger::{notice, success};
use crate::utils;
use crate::validator;
use crate::workspace::scan::{get_path_suffix, Scan, ScanConfig};
use crate::workspace::stats::ScanStats;
use crate::workspace::work_tree::{FileTaskResponse, WorkTree};

#[derive(Args, Debug)]
pub struct ApplyArgs {
    /// License type as SPDX id.
    #[arg(short, long)]
    pub license: String,

    /// The copyright owner.
    #[arg(short, long)]
    pub author: String,

    /// The copyright year.
    #[arg(short, long, value_parser = validator::acceptable_year)]
    #[arg(default_value_t = utils::current_year())]
    pub year: u16,
}

pub fn run(args: &ApplyArgs) -> Result<()> {
    // only: DEBUG
    let mut channel_duration = ChannelDuration::new();

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

    // FIXME: Allow user to provide a function that returns the template
    let template = interpolate!(
        COMPACT_COPYRIGHT_NOTICE,
        CompactCopyrightNotice {
            year: 2024,
            fullname: "John Doe".into(),
            license: "Apache-2.0".into(),
            determiner: "in".into(),
            location: "the root of this project".into(),
        }
    )?;

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
    notice!(format!(
        "Modified: {}   Skipped: {}",
        num_modified, num_skipped
    ));

    channel_duration.drop_channel();
    let task_duration = &channel_duration.as_secs_rounded();
    notice!(format!("Process took {}", task_duration));

    Ok(())
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

fn add_license(context: &mut ScanContext, response: &FileTaskResponse) -> Result<()> {
    if contains_copyright_notice(&response.content) {
        // Skip further processing the file if it already contains a copyright notice
        context.stats.lock().unwrap().skip();

        return Ok(());
    }

    let header = get_header_template(context, response);
    let content = prepend_license(&header.template, &response.content);
    fs::write(&response.path, content)?;

    success!(format!(
        "Apply license to {}",
        &response
            .path
            .strip_prefix(&context.root)
            .unwrap()
            .to_str()
            .unwrap()
    ));

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
