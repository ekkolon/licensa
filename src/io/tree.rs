use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use ignore::DirEntry;
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator,
};

use crate::{
    license::template::has_copyright_notice,
    workspace::{io::prepend_license_notice, walker::WalkBuilder, Error},
    Result,
};

use super::{entry::is_entry_licensable, template_cache::TemplateCache};

pub struct Tree {
    src_root: PathBuf,
    template_cache: TemplateCache,
    dry_run: bool,
}

impl Tree {
    pub fn new<P: AsRef<Path>>(src_root: P) -> Self {
        let src_root = src_root.as_ref().to_path_buf();
        let template_cache = TemplateCache::new();
        Self {
            src_root,
            template_cache,
            dry_run: false,
        }
    }

    // pub fn src_root(&self) -> &Path {
    //     &self.src_root
    // }

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn set_dry_run(&mut self, yes: bool) {
        self.dry_run = yes;
    }

    pub fn add_license<T: AsRef<str>>(&self, template: T, entries: &Vec<DirEntry>) -> Vec<PathBuf> {
        let template = template.as_ref();

        entries
            .into_par_iter()
            .map(|entry| {
                let path = entry.path();
                let content = fs::read_to_string(path)?;

                // Ignore file that already contains a copyright notice
                if has_copyright_notice(content.as_ref()) {
                    return Err(Error::AlreadyLicensed(path.to_path_buf()));
                }

                let header = self.template_cache.compile(path, template);

                if !self.dry_run() {
                    let content = prepend_license_notice(&header.template, content);
                    fs::write(path, content)?;
                }

                Ok(path.to_path_buf())
            })
            .filter_map(|e| e.ok())
            .collect()
    }

    pub fn find_license_candidates(&self, exclude: Option<Vec<String>>) -> Result<Vec<DirEntry>> {
        let mut walk_builder = WalkBuilder::new(&self.src_root);

        if let Some(patterns) = exclude {
            walk_builder.exclude(Some(patterns))?;
        }

        let mut walker = walk_builder.build()?;
        walker
            .quit_while(|res| res.is_err())
            .send_while(|res| is_entry_licensable(res.unwrap()))
            .max_capacity(None);

        let entries: Vec<DirEntry> = walker
            .run_task()
            .iter()
            .par_bridge()
            .into_par_iter()
            .filter_map(|e| e.ok())
            .collect();

        Ok(entries)
    }

    pub fn read_license_info(&self, entries: &Vec<DirEntry>) -> Result<ReadTreeStatistics> {
        let tree = Arc::new(Mutex::new(ReadTreeStatistics::new()));

        entries.par_iter().for_each(|entry: &DirEntry| {
            let mut tree = tree.lock().unwrap();
            let path = entry.path().to_path_buf();
            match fs::read(entry.path()) {
                Ok(content) => {
                    if has_copyright_notice(&content) {
                        tree.passed.push(path);
                    } else {
                        tree.untracked.push(path);
                    }
                }
                Err(_) => {
                    tree.failed.push(path);
                }
            }
        });

        let tree = tree.lock().unwrap().clone();

        Ok(tree)
    }
}

#[derive(Clone)]
pub struct ReadTreeStatistics {
    failed: Vec<PathBuf>,
    untracked: Vec<PathBuf>,
    passed: Vec<PathBuf>,
}

impl ReadTreeStatistics {
    pub fn new() -> Self {
        Self {
            failed: vec![],
            passed: vec![],
            untracked: vec![],
        }
    }
    pub fn count_checked(&self) -> usize {
        self.count_failed() + self.count_passed() + self.count_untracked()
    }
    pub fn count_passed(&self) -> usize {
        self.passed.len()
    }
    pub fn count_failed(&self) -> usize {
        self.failed.len()
    }
    pub fn count_untracked(&self) -> usize {
        self.untracked.len()
    }
    pub fn untracked(&self) -> &Vec<PathBuf> {
        &self.untracked
    }
}
