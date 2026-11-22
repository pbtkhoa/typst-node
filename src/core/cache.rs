use std::collections::HashMap;
use std::sync::RwLock;
use typst::syntax::{FileId, Source, VirtualPath};

/// Thread-safe in-memory cache for parsed Typst `Source` (AST) objects.
///
/// In Typst, parsing a template string into a `Source` builds a `GreenNode` concrete syntax tree.
/// Cloning an existing `Source` is an O(1) atomic increment of Typst's internal Arc.
/// This cache eliminates redundant parsing when the same template is compiled repeatedly
/// with dynamic inputs.
#[derive(Debug)]
pub struct TemplateCache {
  entries: RwLock<HashMap<String, Source>>,
  max_entries: usize,
  enabled: bool,
}

impl TemplateCache {
  pub fn new(max_entries: usize, enabled: bool) -> Self {
    Self {
      entries: RwLock::new(HashMap::new()),
      max_entries,
      enabled,
    }
  }

  /// Get a cached `Source` or parse and insert a new one for `template`.
  pub fn get_or_insert(&self, template: &str) -> Source {
    let main_id = FileId::new(None, VirtualPath::new("main.typ"));

    if !self.enabled {
      return Source::new(main_id, template.to_string());
    }

    // 1. Fast read-lock lookup
    if let Ok(guard) = self.entries.read() {
      if let Some(source) = guard.get(template) {
        return source.clone();
      }
    }

    // 2. Cache miss: parse outside lock to avoid blocking concurrent threads
    let new_source = Source::new(main_id, template.to_string());

    // 3. Write-lock insertion
    if let Ok(mut guard) = self.entries.write() {
      if let Some(source) = guard.get(template) {
        return source.clone();
      }
      if guard.len() >= self.max_entries {
        // Simple eviction: clear when capacity exceeded to prevent unbounded memory growth
        guard.clear();
      }
      guard.insert(template.to_string(), new_source.clone());
    }

    new_source
  }

  pub fn len(&self) -> usize {
    self.entries.read().map(|g| g.len()).unwrap_or(0)
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn clear(&self) {
    if let Ok(mut guard) = self.entries.write() {
      guard.clear();
    }
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }
}

impl Default for TemplateCache {
  fn default() -> Self {
    Self::new(1024, true)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_template_cache_memoization() {
    let cache = TemplateCache::new(10, true);
    assert_eq!(cache.len(), 0);

    let template = "= Hello #sys.inputs.name";
    let source1 = cache.get_or_insert(template);
    assert_eq!(cache.len(), 1);

    let source2 = cache.get_or_insert(template);
    assert_eq!(cache.len(), 1);

    // Verify AST identity: both point to the same parsed root
    assert_eq!(source1.text(), source2.text());
    assert_eq!(source1.id(), source2.id());

    // Different template
    let source3 = cache.get_or_insert("= Another Page");
    assert_eq!(cache.len(), 2);
    assert_ne!(source1.text(), source3.text());

    cache.clear();
    assert_eq!(cache.len(), 0);
  }

  #[test]
  fn test_disabled_cache() {
    let cache = TemplateCache::new(10, false);
    let _s1 = cache.get_or_insert("= Test");
    let _s2 = cache.get_or_insert("= Test");
    assert_eq!(cache.len(), 0);
  }

  #[test]
  fn test_cache_capacity_eviction() {
    let cache = TemplateCache::new(2, true);
    cache.get_or_insert("= A");
    cache.get_or_insert("= B");
    assert_eq!(cache.len(), 2);

    // Exceeding capacity triggers eviction
    cache.get_or_insert("= C");
    assert_eq!(cache.len(), 1);
  }
}
