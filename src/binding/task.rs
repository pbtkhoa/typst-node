use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use napi::bindgen_prelude::Buffer;
use napi::{Env, Result, Task};

use crate::core::cache::TemplateCache;
use crate::core::compiler;
use crate::core::exporter;
use crate::core::font::FontResolver;
use crate::core::world::MemoryWorld;

pub struct RenderPdfTask {
  font_resolver: Arc<FontResolver>,
  template_cache: Arc<TemplateCache>,
  template: String,
  data: Option<serde_json::Value>,
  assets: HashMap<PathBuf, Vec<u8>>,
}

impl RenderPdfTask {
  pub fn new(
    font_resolver: Arc<FontResolver>,
    template_cache: Arc<TemplateCache>,
    template: String,
    data: Option<serde_json::Value>,
    assets: HashMap<PathBuf, Vec<u8>>,
  ) -> Self {
    Self {
      font_resolver,
      template_cache,
      template,
      data,
      assets,
    }
  }
}

impl Task for RenderPdfTask {
  type Output = Vec<u8>;
  type JsValue = Buffer;

  fn compute(&mut self) -> Result<Self::Output> {
    let source = self.template_cache.get_or_insert(&self.template);
    let library = compiler::create_library(self.data.take());
    let world = MemoryWorld::new_with_source(
      source,
      Arc::clone(&self.font_resolver),
      library,
      std::mem::take(&mut self.assets),
    );

    let doc = compiler::compile(&world)?;
    let pdf = exporter::export_pdf(&doc)?;
    Ok(pdf)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output.into())
  }
}

pub struct RenderSvgTask {
  font_resolver: Arc<FontResolver>,
  template_cache: Arc<TemplateCache>,
  template: String,
  data: Option<serde_json::Value>,
  assets: HashMap<PathBuf, Vec<u8>>,
}

impl RenderSvgTask {
  pub fn new(
    font_resolver: Arc<FontResolver>,
    template_cache: Arc<TemplateCache>,
    template: String,
    data: Option<serde_json::Value>,
    assets: HashMap<PathBuf, Vec<u8>>,
  ) -> Self {
    Self {
      font_resolver,
      template_cache,
      template,
      data,
      assets,
    }
  }
}

impl Task for RenderSvgTask {
  type Output = Vec<String>;
  type JsValue = Vec<String>;

  fn compute(&mut self) -> Result<Self::Output> {
    let source = self.template_cache.get_or_insert(&self.template);
    let library = compiler::create_library(self.data.take());
    let world = MemoryWorld::new_with_source(
      source,
      Arc::clone(&self.font_resolver),
      library,
      std::mem::take(&mut self.assets),
    );

    let doc = compiler::compile(&world)?;
    let svgs = exporter::export_svg(&doc)?;
    Ok(svgs)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

pub struct RenderSvgMergedTask {
  font_resolver: Arc<FontResolver>,
  template_cache: Arc<TemplateCache>,
  template: String,
  data: Option<serde_json::Value>,
  assets: HashMap<PathBuf, Vec<u8>>,
}

impl RenderSvgMergedTask {
  pub fn new(
    font_resolver: Arc<FontResolver>,
    template_cache: Arc<TemplateCache>,
    template: String,
    data: Option<serde_json::Value>,
    assets: HashMap<PathBuf, Vec<u8>>,
  ) -> Self {
    Self {
      font_resolver,
      template_cache,
      template,
      data,
      assets,
    }
  }
}

impl Task for RenderSvgMergedTask {
  type Output = String;
  type JsValue = String;

  fn compute(&mut self) -> Result<Self::Output> {
    let source = self.template_cache.get_or_insert(&self.template);
    let library = compiler::create_library(self.data.take());
    let world = MemoryWorld::new_with_source(
      source,
      Arc::clone(&self.font_resolver),
      library,
      std::mem::take(&mut self.assets),
    );

    let doc = compiler::compile(&world)?;
    let merged = exporter::export_svg_merged(&doc)?;
    Ok(merged)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

pub struct RenderPngTask {
  font_resolver: Arc<FontResolver>,
  template_cache: Arc<TemplateCache>,
  template: String,
  data: Option<serde_json::Value>,
  assets: HashMap<PathBuf, Vec<u8>>,
  ppi: f64,
  pages: Option<Vec<u32>>,
}

impl RenderPngTask {
  pub fn new(
    font_resolver: Arc<FontResolver>,
    template_cache: Arc<TemplateCache>,
    template: String,
    data: Option<serde_json::Value>,
    assets: HashMap<PathBuf, Vec<u8>>,
    ppi: f64,
    pages: Option<Vec<u32>>,
  ) -> Self {
    Self {
      font_resolver,
      template_cache,
      template,
      data,
      assets,
      ppi,
      pages,
    }
  }
}

impl Task for RenderPngTask {
  type Output = Vec<Vec<u8>>;
  type JsValue = Vec<Buffer>;

  fn compute(&mut self) -> Result<Self::Output> {
    let source = self.template_cache.get_or_insert(&self.template);
    let library = compiler::create_library(self.data.take());
    let world = MemoryWorld::new_with_source(
      source,
      Arc::clone(&self.font_resolver),
      library,
      std::mem::take(&mut self.assets),
    );

    let doc = compiler::compile(&world)?;
    let pngs = exporter::export_png(&doc, self.ppi, self.pages.as_deref())?;
    Ok(pngs)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    let buffers = output.into_iter().map(Buffer::from).collect();
    Ok(buffers)
  }
}
