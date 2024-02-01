use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{
    cache::{Cachable, Cache},
    copyright_notice::{
        contains_copyright_notice, CompactCopyrightNotice, COMPACT_COPYRIGHT_NOTICE,
    },
    header::{extract_hash_bang, SourceHeaders},
    interpolation::interpolate,
    logger::notice,
    utils::to_elapsed_secs,
};

use anyhow::Result;
use colored::Colorize;
use rayon::prelude::*;
use serde::Serialize;

use super::{
    scan::{get_path_suffix, Scan, ScanConfig},
    stats::ScanStats,
    work_tree::FileTaskResponse,
};

#[derive(Clone)]
struct LicenseTemplate<P>
where
    P: Serialize + 'static + ?Sized,
{
    content: String,
    data: P,
}

#[derive(Clone)]
struct ScanContext {
    pub root: PathBuf,
    pub stats: Arc<Mutex<ScanStats>>,
    pub cache: Arc<Cache<LicenseHeaderTemplate>>,
    pub template: Arc<Mutex<String>>,
}

pub fn example_scan_parallel() -> anyhow::Result<()> {
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

    let num_files = candidates.len();

    // ========================================================

    // ========================================================
    // File processing
    // ========================================================

    let cache = Cache::<LicenseHeaderTemplate>::new();
    let stats = Arc::new(Mutex::new(ScanStats::new()));

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

    let mut worktree = super::work_tree::WorkTree::new();

    let rt = worktree.add_task(context, read_entry);

    worktree.run(candidates);

    // Clear cache
    cache.clear();

    // ========================================================

    // only: DEBUG
    let duration = to_elapsed_secs(start_time.elapsed());
    notice!(format!("Process took {}", duration));

    println!("After run - Cache size: {}", &cache.size());

    let skipped = &stats.lock().unwrap().skipped;
    println!("Modified: {}   Skipped: {}", num_files - skipped, skipped);

    Ok(())
}

fn read_entry(context: &mut ScanContext, response: &FileTaskResponse) -> Result<()> {
    if contains_copyright_notice(&response.content) {
        // Skip further processing the file if it already contains a copyright notice
        context.stats.lock().unwrap().skip();
        return Ok(());
    }

    let header = get_context_template(context, response);

    let mut line = extract_hash_bang(response.content.as_bytes()).unwrap_or_default();
    let mut content = response.content.as_bytes().to_vec();
    if !line.is_empty() {
        content = content.split_off(line.len());
        if line[line.len() - 1] != b'\n' {
            line.push(b'\n');
        }
        content = [line, header.template.as_bytes().to_vec(), content].concat();
    } else {
        content = [header.template.as_bytes().to_vec(), content].concat();
    }

    fs::write(&response.path, content)?;
    println!(
        "License applied to: {}",
        &response.path.file_name().unwrap().to_str().unwrap()
    );

    Ok(())
}

#[derive(Debug, Clone)]
pub struct LicenseHeaderTemplate {
    pub extension: String,
    pub template: String,
}

impl Cachable for LicenseHeaderTemplate {
    fn cache_id(&self) -> String {
        self.extension.to_owned()
    }
}

fn get_context_template(
    context: &mut ScanContext,
    task: &FileTaskResponse,
) -> Arc<LicenseHeaderTemplate> {
    // FIXME: Compute cache id in FileTree
    let cache_id = get_path_suffix(&task.path);

    // Reuse cached template for this candidate
    if !context.cache.contains(&cache_id) {
        // Compile and cache template for this candidate
        println!("Template not in cache for cache ID: {}", &cache_id);

        let header = SourceHeaders::find_header_definition_by_extension(&cache_id).unwrap();
        let template = context.template.lock().unwrap();
        let template = template.as_str();
        let compiled_template = header.header_prefix.apply(template).unwrap();

        // Add compiled template to cache
        context.cache.add(LicenseHeaderTemplate {
            extension: cache_id.clone(),
            template: compiled_template,
        });
    }

    context.cache.get(&cache_id).unwrap()
}
