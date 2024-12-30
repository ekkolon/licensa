// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::config::Config;
use crate::ops::scan::{get_path_suffix, is_candidate};
use crate::ops::work_tree::{FileTaskResponse, WorkTree};
use crate::template::cache::{Cachable, Cache};
use crate::template::copyright::SPDX_COPYRIGHT_NOTICE;
use crate::template::has_copyright_notice;
use crate::template::header::{extract_hash_bang, SourceHeaders};
use crate::terminal::Step;
use crate::workspace::walker::WalkBuilder;
use crate::workspace::LicensaWorkspace;
use crate::{error, terminal};

use anyhow::Result;
use clap::Parser;
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
    let mut task = terminal::Task::new("Add SPDX license headers");
    task.start()?;

    let workspace_root = std::env::current_dir()?;
    let workspace_config = args.to_config()?;

    // ========================================================
    // Scanning process
    // ========================================================
    let candidates = scan_workspace(&workspace_root, &workspace_config)?;

    // ========================================================
    // File processing
    // ========================================================
    let cache = Cache::<HeaderTemplate>::new();

    let template_engine = handlebars::Handlebars::new();
    let template = template_engine.render_template(SPDX_COPYRIGHT_NOTICE, &workspace_config)?;
    let template = Arc::new(Mutex::new(template));

    let context = ScanContext {
        cache: cache.clone(),
        template,
    };

    let mut worktree = WorkTree::new();
    worktree.add_task(context, apply_license_notice);
    worktree.run(candidates);

    // ========================================================
    // Clear cache
    cache.clear();

    Ok(())
}

#[derive(Clone)]
struct ScanContext {
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
fn scan_workspace<P>(workspace_root: P, config: &LicensaWorkspace) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let mut walk_builder = WalkBuilder::new(&workspace_root);
    walk_builder.exclude(Some(config.exclude.clone()))?;

    let mut walker = walk_builder.build()?;
    walker.quit_while(|res| res.is_err());
    walker.send_while(|res| is_candidate(res.unwrap()));

    let candidates = walker
        .run_task()
        .iter()
        .par_bridge()
        .into_par_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<PathBuf>>();

    Ok(candidates)
}

fn apply_license_notice(context: &mut ScanContext, response: &FileTaskResponse) -> Result<()> {
    // Ignore file that already contains a copyright notice
    if has_copyright_notice(response.content.as_bytes()) {
        return Ok(());
    }

    let header = resolve_header_template(context, response);
    let content = prepend_license_notice(&header.template, &response.content);
    fs::write(&response.path, content)?;

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
