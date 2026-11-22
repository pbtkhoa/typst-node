use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};

use chrono::{Datelike, Utc};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::Library;

use super::font::FontResolver;

/// Normalize a virtual path into a canonical rootless PathBuf:
/// - Strips leading root slashes/prefixes (so `/foo/bar` -> `foo/bar`)
/// - Strips `.` (current directory components)
/// - Resolves `..` (parent directory components) cleanly without escaping virtual root
/// - Normalizes Windows `\` separators to standard path components
pub fn normalize_virtual_path(path: &Path) -> PathBuf {
  let path_str = path.to_string_lossy();
  let normalized_str = path_str.replace('\\', "/");
  let clean_path = Path::new(&normalized_str);

  let mut normalized = PathBuf::new();
  for component in clean_path.components() {
    match component {
      Component::Prefix(_) | Component::RootDir | Component::CurDir => {
        // Ignore root, prefix, and current directory (.)
      }
      Component::ParentDir => {
        // Pop parent component if possible, but do not escape root
        normalized.pop();
      }
      Component::Normal(c) => {
        normalized.push(c);
      }
    }
  }
  normalized
}

pub struct MemoryWorld {
  library: LazyHash<Library>,
  font_resolver: Arc<FontResolver>,
  main_id: FileId,
  source: Source,
  assets: HashMap<PathBuf, Bytes>,
  subfile_sources: RwLock<HashMap<PathBuf, Source>>,
  now: chrono::DateTime<Utc>,
}

impl MemoryWorld {
  pub fn new_with_source(
    source: Source,
    font_resolver: Arc<FontResolver>,
    library: LazyHash<Library>,
    assets: HashMap<PathBuf, Vec<u8>>,
  ) -> Self {
    let main_id = source.id();
    let converted_assets: HashMap<PathBuf, Bytes> = assets
      .into_iter()
      .map(|(p, buf)| (normalize_virtual_path(&p), Bytes::new(buf)))
      .collect();

    Self {
      library,
      font_resolver,
      main_id,
      source,
      assets: converted_assets,
      subfile_sources: RwLock::new(HashMap::new()),
      now: Utc::now(),
    }
  }

  pub fn new(
    template: String,
    font_resolver: Arc<FontResolver>,
    library: LazyHash<Library>,
    assets: HashMap<PathBuf, Vec<u8>>,
  ) -> Self {
    let main_id = FileId::new(None, VirtualPath::new("main.typ"));
    let source = Source::new(main_id, template);
    Self::new_with_source(source, font_resolver, library, assets)
  }
}

impl typst::World for MemoryWorld {
  fn library(&self) -> &LazyHash<Library> {
    &self.library
  }

  fn book(&self) -> &LazyHash<FontBook> {
    self.font_resolver.book()
  }

  fn main(&self) -> FileId {
    self.main_id
  }

  fn source(&self, id: FileId) -> FileResult<Source> {
    if id == self.main_id {
      Ok(self.source.clone())
    } else {
      let rootless = id.vpath().as_rootless_path();
      let normalized = normalize_virtual_path(rootless);

      if let Ok(guard) = self.subfile_sources.read() {
        if let Some(src) = guard.get(&normalized) {
          return Ok(src.clone());
        }
      }

      if let Some(bytes) = self.assets.get(&normalized) {
        let text = std::str::from_utf8(bytes.as_slice()).map_err(|_| FileError::InvalidUtf8)?;
        let src = Source::new(id, text.to_string());
        if let Ok(mut guard) = self.subfile_sources.write() {
          guard.insert(normalized, src.clone());
        }
        Ok(src)
      } else {
        Err(FileError::NotFound(normalized))
      }
    }
  }

  fn file(&self, id: FileId) -> FileResult<Bytes> {
    let rootless = id.vpath().as_rootless_path();
    let normalized = normalize_virtual_path(rootless);
    if let Some(bytes) = self.assets.get(&normalized) {
      Ok(bytes.clone())
    } else {
      Err(FileError::NotFound(normalized))
    }
  }

  fn font(&self, index: usize) -> Option<Font> {
    self.font_resolver.font(index)
  }

  fn today(&self, offset: Option<i64>) -> Option<Datetime> {
    let date = match offset {
      Some(hours) => {
        let duration = chrono::Duration::hours(hours);
        self.now.checked_add_signed(duration)?.date_naive()
      }
      None => self.now.date_naive(),
    };

    Datetime::from_ymd(date.year(), date.month() as u8, date.day() as u8)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::compiler;

  #[test]
  fn test_normalize_virtual_path() {
    assert_eq!(
      normalize_virtual_path(Path::new("a/b/c")),
      PathBuf::from("a/b/c")
    );
    assert_eq!(
      normalize_virtual_path(Path::new("/a/b/c")),
      PathBuf::from("a/b/c")
    );
    assert_eq!(
      normalize_virtual_path(Path::new("./a/./b/../c")),
      PathBuf::from("a/c")
    );
    assert_eq!(
      normalize_virtual_path(Path::new("../../a")),
      PathBuf::from("a")
    );
    assert_eq!(
      normalize_virtual_path(Path::new("a/b/../../c/d/")),
      PathBuf::from("c/d")
    );
    assert_eq!(
      normalize_virtual_path(Path::new("a\\b\\c")),
      PathBuf::from("a/b/c")
    );
    assert_eq!(
      normalize_virtual_path(Path::new(".\\a\\..\\b\\c.png")),
      PathBuf::from("b/c.png")
    );
  }

  #[test]
  fn test_nested_virtual_paths_and_includes() {
    let resolver = Arc::new(FontResolver::new(&[], false));
    let library = compiler::create_library(None);

    let mut assets = HashMap::new();
    assets.insert(
      PathBuf::from("chapters/intro.typ"),
      b"#include \"../shared/header.typ\"\n= Chapter 1\n#include \"sections/sec1.typ\"".to_vec(),
    );
    assets.insert(PathBuf::from("shared/header.typ"), b"Header text".to_vec());
    assets.insert(
      PathBuf::from("chapters/sections/sec1.typ"),
      b"== Section 1 content".to_vec(),
    );

    let world = MemoryWorld::new(
      "#include \"chapters/intro.typ\"".to_string(),
      resolver,
      library,
      assets,
    );

    let doc = compiler::compile(&world);
    assert!(doc.is_ok(), "Compilation failed: {:?}", doc.err());
  }

  #[test]
  fn test_virtual_path_variations_and_normalization() {
    let resolver = Arc::new(FontResolver::new(&[], false));
    let library = compiler::create_library(None);

    // A minimal valid 1x1 PNG
    let minimal_png: Vec<u8> = vec![
      0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, // PNG header
      0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, // IHDR
      0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15,
      0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, // IDAT
      0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4,
      0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, // IEND
      0x42, 0x60, 0x82,
    ];

    let mut assets = HashMap::new();
    // Asset provided with "./" prefix
    assets.insert(PathBuf::from("./images/logo.png"), minimal_png.clone());
    // Asset provided with leading "/"
    assets.insert(
      PathBuf::from("/styles/header.typ"),
      b"== Header Content\n#image(\"/images/logo.png\")".to_vec(),
    );
    // Nested relative subfile
    assets.insert(
      PathBuf::from("docs/reports/deep/item.typ"),
      b"#image(\"../../../images/logo.png\")\n#include \"/styles/header.typ\"".to_vec(),
    );
    // File read via #read()
    assets.insert(
      PathBuf::from("data/config.json"),
      b"{\"title\": \"In-Memory Config\"}".to_vec(),
    );
    // Local asset relative to chapters/
    assets.insert(PathBuf::from("chapters/chart.png"), minimal_png.clone());
    assets.insert(PathBuf::from("chapters/page.typ"), b"= Page\n#image(\"chart.png\")\n#let raw = json(\"../data/config.json\")\nConfig title: #raw.title".to_vec());

    let world = MemoryWorld::new(
            "#include \"styles/header.typ\"\n#image(\"images/logo.png\")\n#include \"docs/reports/deep/item.typ\"\n#include \"chapters/page.typ\"".to_string(),
            resolver,
            library,
            assets,
        );

    let doc = compiler::compile(&world);
    assert!(doc.is_ok(), "Failed: {:?}", doc.err());
  }
}
