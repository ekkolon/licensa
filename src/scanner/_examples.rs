use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{
    cache::{Cachable, Cache},
    helpers::channel_duration::ChannelDuration,
    scanner::{header_checker::contains_copyright_notice, scan::get_path_suffix},
    workspace::{self, FileTaskResponse},
};

use rayon::prelude::*;

use super::{
    scan::{Scan, ScanConfig},
    stats::ScanStats,
};

pub fn example_scan_op() -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    let scan_config = ScanConfig {
        limit: 100,
        exclude: None,
        root,
    };
    let scan = Scan::new(scan_config);
    let result = scan.run();

    let candidates = scan.run()?;
    if candidates.is_empty() {
        return Ok(());
    }

    let first_candidate = &candidates[2];
    let dotext = &first_candidate
        .extension
        .clone()
        .map(|e| format!(".{}", &e).to_string());

    println!("Extension {}", &dotext.as_ref().unwrap());

    if dotext.is_some() {
        println!("Extension {}", &dotext.as_ref().unwrap());
        let prefix = super::source::SourceHeaders::find_header_prefix_for_extension(
            dotext.as_ref().unwrap(),
        );
        if prefix.is_some() {
            let p = prefix.as_ref().unwrap().to_owned();

            println!(
                "Prefix for extension {}: {:?}",
                &dotext.as_ref().unwrap(),
                &prefix.as_ref().unwrap()
            );

            let file_content =
                std::fs::read_to_string("./src/main.rs").expect("Failed to read file");

            if super::header_checker::contains_copyright_notice(file_content) {
                println!("File ./example.js has a license header.");
            } else {
                println!("File ./example.js does not contain license header.");
            }
        }
    }

    Ok(())
}

#[derive(Clone)]
struct ScanContext {
    pub root: PathBuf,
    pub stats: Arc<Mutex<ScanStats>>,
    pub cache: Arc<Cache<LicenseHeaderTemplate>>,
}

pub fn example_scan_parallel() -> anyhow::Result<()> {
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
        .run_parallel()
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

    let context = ScanContext {
        root,
        cache: cache.clone(),
        stats: stats.clone(),
    };

    let mut worktree = workspace::FileTree::new();
    let rt = worktree.add_task(context, read_entry);

    worktree.run(candidates);

    // Clear cache
    cache.clear();

    // ========================================================

    // only: DEBUG
    channel_duration.drop_channel();
    let task_duration = &channel_duration.get_duration().as_secs_f32();
    println!("Took {} for {:?} files", task_duration, num_files);

    println!("After run - Cache size: {}", &cache.size());

    let skipped = &stats.lock().unwrap().skipped;
    println!("Modified: {}   Skipped: {}", num_files - skipped, skipped);

    Ok(())
}

fn read_entry(context: &mut ScanContext, response: &FileTaskResponse) {
    if contains_copyright_notice(&response.content) {
        // Skip further processing the file if it already contains a copyright notice
        context.stats.lock().unwrap().skip();
        return;
    }

    let _ = get_context_template(context, &response.path);
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

fn get_context_template<P>(context: &mut ScanContext, path: P)
where
    P: AsRef<Path>,
{
    // FIXME: Compute cache id in FileTree
    let cache_id = get_path_suffix(path.as_ref());

    if context.cache.contains(&cache_id) {
        // Reuse cached template for this candidate
        println!("Cached template found for cache ID: {}", &cache_id);

        let header = &context.cache.get(&cache_id).unwrap();
        let template = &header.template;
    } else {
        // Compile and cache template for this candidate
        println!("Template not in cache for cache ID: {}", &cache_id);

        // TODO: Compile license template based on CLI args
        let compiled_template = "Some random compiled string";

        // Add compiled template to cache
        context.cache.add(LicenseHeaderTemplate {
            extension: cache_id,
            template: compiled_template.into(),
        });
    }
}
