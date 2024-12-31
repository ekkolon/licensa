use std::{borrow::Cow, path::Path, sync::Arc};

use crate::license::template::{cache::Cache, header::SourceHeaders, HeaderTemplate};

use super::entry::get_path_suffix;

pub struct TemplateCache {
    cache: Cache<HeaderTemplate>,
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateCache {
    pub fn new() -> Self {
        Self {
            cache: Cache::new(),
        }
    }
    pub fn compile<'a, P>(&self, path: P, template: impl Into<Cow<'a, str>>) -> Arc<HeaderTemplate>
    where
        P: AsRef<Path>,
    {
        // FIXME: Compute cache id in FileTree
        let cache_id = get_path_suffix(&path);

        // Reuse cached template for this candidate
        if self.cache.contains(&cache_id) {
            return self.cache.get(&cache_id).unwrap();
        }

        // Compile and cache template for this candidate
        let header = SourceHeaders::find_header_definition_by_extension(&cache_id).unwrap();
        let compiled_template = header.header_prefix.apply(template.into()).unwrap();

        // FIXME: Use unique cache_id for header prefixes to prevent compiling
        // that use the same format.
        self.cache.add(HeaderTemplate {
            extension: cache_id.clone(),
            template: compiled_template,
        });

        self.cache.get(&cache_id).unwrap()
    }
}
