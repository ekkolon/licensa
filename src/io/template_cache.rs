use std::{borrow::Cow, sync::Arc};

use lazy_static::lazy_static;
use serde::Serialize;

use crate::{
    license::template::{cache::Cache, header::SourceHeaders, HeaderTemplate},
    Result,
};

pub struct TemplateCache {
    engine: handlebars::Handlebars<'static>,
    cache: Cache<HeaderTemplate>,
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    static ref TEMPLATE_CACHE: TemplateCache = TemplateCache::new();
}

impl TemplateCache {
    fn new() -> Self {
        let engine = handlebars::Handlebars::new();
        Self {
            cache: Cache::new(),
            engine,
        }
    }

    pub fn compile<K, C, D>(template_key: K, content: C, data: D) -> Result<Arc<HeaderTemplate>>
    where
        K: AsRef<str>,
        C: AsRef<str>,
        D: Serialize,
    {
        let cache_id = template_key.as_ref();
        // Reuse cached template for this candidate
        if TEMPLATE_CACHE.cache.contains(cache_id) {
            let template = TEMPLATE_CACHE.cache.get(cache_id).unwrap();
            return Ok(template);
        }

        // Compile and cache template for this candidate
        let header = SourceHeaders::find_header_definition_by_extension(cache_id).unwrap();
        let template = TemplateCache::render(content, data)?;
        let template = header.header_prefix.apply(template).unwrap();

        // FIXME: Use unique cache_id for header prefixes to prevent compiling
        // that use the same format.
        TEMPLATE_CACHE.cache.add(HeaderTemplate {
            extension: cache_id.to_string(),
            template,
        });

        let template = TEMPLATE_CACHE.cache.get(cache_id).unwrap();
        Ok(template)
    }

    pub fn render<'a, T: AsRef<str>, D: Serialize>(template: T, data: D) -> Result<Cow<'a, str>> {
        let template = TEMPLATE_CACHE
            .engine
            .render_template(template.as_ref(), &data)?;

        Ok(Cow::Owned(template))
    }
}
