use std::fs;
use std::path::{Path, PathBuf};
use typst::foundations::Bytes;
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst_kit::fonts::{FontSearcher, FontSlot};

pub enum FontEntry {
  Slot(FontSlot),
  Memory(Font),
}

impl FontEntry {
  pub fn get(&self) -> Option<Font> {
    match self {
      Self::Slot(slot) => slot.get(),
      Self::Memory(font) => Some(font.clone()),
    }
  }
}

pub struct FontResolver {
  book: LazyHash<FontBook>,
  fonts: Vec<FontEntry>,
}

impl FontResolver {
  /// Creates a new `FontResolver` initializing embedded fonts (from `typst-kit`)
  /// and optionally scanning system fonts and extra font directories/files.
  pub fn new(font_paths: &[PathBuf], include_system_fonts: bool) -> Self {
    let mut searcher = FontSearcher::new();
    searcher.include_system_fonts(include_system_fonts);
    searcher.include_embedded_fonts(true);

    let fonts_data = searcher.search();

    let mut book = fonts_data.book;
    let mut fonts: Vec<FontEntry> = fonts_data.fonts.into_iter().map(FontEntry::Slot).collect();

    // Load custom fonts from provided paths
    for path in font_paths {
      Self::load_path(path, &mut book, &mut fonts);
    }

    Self {
      book: LazyHash::new(book),
      fonts,
    }
  }

  /// Creates a default resolver containing only bundled embedded fonts.
  pub fn default_embedded() -> Self {
    Self::new(&[], false)
  }

  /// Add an in-memory font buffer.
  pub fn add_font_bytes(&mut self, bytes: Vec<u8>) {
    let mut book = std::mem::take(&mut self.book).into_inner();
    let typst_bytes = Bytes::new(bytes);
    for font in Font::iter(typst_bytes) {
      book.push(font.info().clone());
      self.fonts.push(FontEntry::Memory(font));
    }
    self.book = LazyHash::new(book);
  }

  fn load_path(path: &Path, book: &mut FontBook, fonts: &mut Vec<FontEntry>) {
    if path.is_dir() {
      if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
          let entry_path = entry.path();
          if entry_path.is_file() {
            Self::load_font_file(&entry_path, book, fonts);
          }
        }
      }
    } else if path.is_file() {
      Self::load_font_file(path, book, fonts);
    }
  }

  fn load_font_file(path: &Path, book: &mut FontBook, fonts: &mut Vec<FontEntry>) {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("ttf")
      || ext.eq_ignore_ascii_case("otf")
      || ext.eq_ignore_ascii_case("woff")
      || ext.eq_ignore_ascii_case("woff2")
      || ext.eq_ignore_ascii_case("ttc")
      || ext.eq_ignore_ascii_case("otc")
    {
      if let Ok(data) = fs::read(path) {
        let typst_bytes = Bytes::new(data);
        for font in Font::iter(typst_bytes) {
          book.push(font.info().clone());
          fonts.push(FontEntry::Memory(font));
        }
      }
    }
  }

  pub fn book(&self) -> &LazyHash<FontBook> {
    &self.book
  }

  pub fn font(&self, id: usize) -> Option<Font> {
    self.fonts.get(id).and_then(|entry| entry.get())
  }
}
