// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait Cachable {
    /// Generates a unique cache identifier for the implementor.
    fn cache_id(&self) -> String;
}

type CacheInner<T> = Arc<Mutex<HashMap<String, Arc<T>>>>;

/// A simple caching system for arbitrary items.
pub struct Cache<T>
where
    T: Clone + Cachable,
{
    inner: CacheInner<T>,
}

impl<T> Cache<T>
where
    T: Clone + Cachable,
{
    /// Creates a new instance of `Cache`.
    pub fn new() -> Arc<Self> {
        Arc::new(Cache {
            inner: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Adds or updates the cache with the provided item.
    ///
    /// # Arguments
    ///
    /// * `item` - An item implementing the `Cachable` trait.  
    pub fn set(&mut self, item: T) {
        let mut cache = self.inner.lock().unwrap();

        let cache_id = item.cache_id();

        cache.entry(cache_id).or_insert_with(|| Arc::new(item));
    }

    /// Adds or updates the cache with the provided item.
    ///
    /// # Arguments
    ///
    /// * `item` - An item implementing the `Cachable` trait.  
    pub fn add(&self, item: T) {
        let mut cache = self.inner.lock().unwrap();

        let cache_id = item.cache_id();

        cache.entry(cache_id).or_insert_with(|| Arc::new(item));
    }

    /// Retrieves the cached item for the given cache identifier.
    ///
    /// # Arguments
    ///
    /// * `cache_id` - A string slice representing the cache identifier.
    ///
    /// # Returns
    ///
    /// An `Option` containing a cloned `Arc<T>` if the item is found, otherwise `None`.
    pub fn get<I>(&self, cache_id: I) -> Option<Arc<T>>
    where
        I: AsRef<str>,
    {
        let cache = self.inner.lock().unwrap();
        let id = cache_id.as_ref();
        cache.get(id).cloned()
    }

    pub fn value(&mut self, item: T) -> Arc<T> {
        let mut cache = self.inner.lock().unwrap();
        let entry = cache.entry(item.cache_id());
        entry.or_insert_with(|| item.into()).to_owned()
    }

    /// Clears all items from the cache.
    pub fn clear(&self) {
        let mut cache = self.inner.lock().unwrap();
        cache.clear();
    }

    /// Removes a specific item from the cache.
    ///
    /// # Arguments
    ///
    /// * `cache_id` - The unique identifier for the item to be removed.
    pub fn remove<I>(&self, cache_id: I)
    where
        I: AsRef<str>,
    {
        let mut cache = self.inner.lock().unwrap();
        let id = cache_id.as_ref();
        cache.remove(id);
    }

    /// Retrieves all items in the cache.
    ///
    /// # Returns
    ///
    /// A `Vec` containing cloned `Arc<T>` items in the cache.
    pub fn get_all(&self) -> Vec<Arc<T>> {
        let cache = self.inner.lock().unwrap();
        cache.values().cloned().collect()
    }

    /// Checks if the cache is empty.
    ///
    /// # Returns
    ///
    /// `true` if the cache is empty, otherwise `false`.
    pub fn is_empty(&self) -> bool {
        let cache = self.inner.lock().unwrap();
        cache.is_empty()
    }

    /// Returns the number of items in the cache.
    ///
    /// # Returns
    ///
    /// The number of items in the cache.
    pub fn size(&self) -> usize {
        let cache = self.inner.lock().unwrap();
        cache.len()
    }

    // /// Provides an iterator over the items in the cache.
    // ///
    // /// # Returns
    // ///
    // /// An iterator over cloned `Arc<T>` items in the cache.
    // pub fn iter(&self) -> CacheIter<T> {
    //   CacheIter { cache: &self.inner }
    // }

    /// Checks if a specific item exists in the cache.
    ///
    /// # Arguments
    ///
    /// * `cache_id` - The unique identifier for the item.
    ///
    /// # Returns
    ///
    /// `true` if the item exists in the cache, otherwise `false`.
    pub fn contains<I>(&self, cache_id: I) -> bool
    where
        I: AsRef<str>,
    {
        let cache = self.inner.lock().unwrap();
        let id = cache_id.as_ref();
        cache.contains_key(id)
    }
}

// /// Iterator over items in the cache.
// pub struct CacheIter<'a, T>
// where
//   T: 'a,
// {
//   cache: &'a Mutex<HashMap<String, Arc<T>>>,
// }

// impl<'a, T> Iterator for CacheIter<'a, T>
// where
//   T: 'a,
// {
//   type Item = &'a Arc<T>;

//   fn next(&mut self) -> Option<Self::Item> {
//     let cache = self.cache.lock().unwrap();
//     cache.values().next()
//   }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TemplateItem {
        pub template: String,
        pub extension: String,
    }
    impl Cachable for TemplateItem {
        fn cache_id(&self) -> String {
            self.extension.clone()
        }
    }

    #[test]
    fn test_cache_add_and_get_template() {
        let cache = Cache::<TemplateItem>::new();
        let extension = ".rs";
        let template = "Your license template here";

        cache.add(TemplateItem {
            extension: extension.into(),
            template: template.into(),
        });

        let cached_template = cache.get(extension);
        assert!(cached_template.is_some());
        assert_eq!(cached_template.unwrap().template, template);
    }

    #[test]
    fn test_cache_get_template_not_found() {
        let cache = Cache::<TemplateItem>::new();
        let extension = ".rs";

        let cached_template = cache.get(extension);
        assert!(cached_template.is_none());
    }

    #[test]
    fn test_cache_remove_template() {
        let cache = Cache::<TemplateItem>::new();
        let extension = ".rs";
        let template = "Your license template here";

        cache.add(TemplateItem {
            extension: extension.into(),
            template: template.into(),
        });

        cache.remove(extension);
        let cached_template = cache.get(extension);
        assert!(cached_template.is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::<TemplateItem>::new();
        let extension = ".rs";
        let template = "Your license template here";

        cache.add(TemplateItem {
            extension: extension.into(),
            template: template.into(),
        });

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_get_all() {
        let cache = Cache::<TemplateItem>::new();
        let ext1 = ".rs";
        let ext2 = ".toml";
        let template1 = "Your license template for Rust";
        let template2 = "Your license template for TOML";

        cache.add(TemplateItem {
            extension: ext1.into(),
            template: template1.into(),
        });

        cache.add(TemplateItem {
            extension: ext2.into(),
            template: template2.into(),
        });

        let all_templates = cache.get_all();
        assert_eq!(all_templates.len(), 2);
    }

    #[test]
    fn test_cache_size() {
        let cache = Cache::<TemplateItem>::new();
        let ext1 = ".rs";
        let ext2 = ".toml";
        let template1 = "Your license template for Rust";
        let template2 = "Your license template for TOML";

        cache.add(TemplateItem {
            extension: ext1.into(),
            template: template1.into(),
        });

        cache.add(TemplateItem {
            extension: ext2.into(),
            template: template2.into(),
        });

        let size = cache.size();
        assert_eq!(size, 2);
    }

    // #[test]
    // fn test_cache_iter() {
    //   let cache = Cache::<TemplateItem>::new();
    //   let ext1 = ".rs";
    //   let ext2 = ".toml";
    //   let template1 = "Your license template for Rust";
    //   let template2 = "Your license template for TOML";

    //   cache.add(TemplateItem {
    //     extension: ext1.into(),
    //     template: template1.into(),
    //   });

    //   cache.add(TemplateItem {
    //     extension: ext2.into(),
    //     template: template2.into(),
    //   });

    //   let mut iter = cache.iter();
    //   assert_eq!(iter.next().unwrap().template, template1);
    //   assert_eq!(iter.next().unwrap().template, template2);
    //   assert!(iter.next().is_none());
    // }

    #[test]
    fn test_cache_contains() {
        let cache = Cache::<TemplateItem>::new();
        let ext1 = ".rs";
        let template1 = "Your license template for Rust";

        cache.add(TemplateItem {
            extension: ext1.into(),
            template: template1.into(),
        });

        assert!(cache.contains(ext1));
        assert!(!cache.contains(".toml"));
    }
}
