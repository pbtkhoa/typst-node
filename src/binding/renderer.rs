use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::Env;
use napi_derive::napi;

use crate::core::cache::TemplateCache;
use crate::core::font::FontResolver;

use super::options::{JsRenderOptions, JsRenderPngOptions, JsTypstRendererOptions};
use super::task::{RenderPdfTask, RenderPngTask, RenderSvgMergedTask, RenderSvgTask};

fn convert_assets(assets: Option<HashMap<String, Buffer>>) -> HashMap<PathBuf, Vec<u8>> {
  assets
    .map(|map| {
      map
        .into_iter()
        .map(|(k, v)| (PathBuf::from(k), v.to_vec()))
        .collect()
    })
    .unwrap_or_default()
}

#[napi]
pub struct TypstRenderer {
  font_resolver: Arc<FontResolver>,
  template_cache: Arc<TemplateCache>,
}

#[napi]
impl TypstRenderer {
  #[napi(constructor)]
  pub fn new(options: Option<JsTypstRendererOptions>) -> napi::Result<Self> {
    let font_paths = options
      .as_ref()
      .and_then(|o| o.font_paths.as_ref())
      .map(|paths| paths.iter().map(PathBuf::from).collect::<Vec<_>>())
      .unwrap_or_default();
    let include_system = options
      .as_ref()
      .and_then(|o| o.include_system_fonts)
      .unwrap_or(false);

    let cache_capacity = options
      .as_ref()
      .and_then(|o| o.cache_capacity)
      .map(|c| c as usize)
      .unwrap_or(1024);
    let enable_cache = options
      .as_ref()
      .and_then(|o| o.enable_cache)
      .unwrap_or(true);

    let resolver = FontResolver::new(&font_paths, include_system);
    let cache = TemplateCache::new(cache_capacity, enable_cache);

    Ok(Self {
      font_resolver: Arc::new(resolver),
      template_cache: Arc::new(cache),
    })
  }

  #[napi]
  pub fn cache_size(&self) -> u32 {
    self.template_cache.len() as u32
  }

  #[napi]
  pub fn clear_cache(&self) {
    self.template_cache.clear();
  }

  #[napi(ts_return_type = "Promise<Buffer>")]
  pub fn render(&self, options: JsRenderOptions) -> AsyncTask<RenderPdfTask> {
    self.render_pdf(options)
  }

  #[napi(ts_return_type = "Promise<Buffer>")]
  pub fn render_pdf(&self, options: JsRenderOptions) -> AsyncTask<RenderPdfTask> {
    let assets = convert_assets(options.assets);
    AsyncTask::new(RenderPdfTask::new(
      Arc::clone(&self.font_resolver),
      Arc::clone(&self.template_cache),
      options.template,
      options.data,
      assets,
    ))
  }

  #[napi(ts_return_type = "Promise<Array<string>>")]
  pub fn render_svg(&self, options: JsRenderOptions) -> AsyncTask<RenderSvgTask> {
    let assets = convert_assets(options.assets);
    AsyncTask::new(RenderSvgTask::new(
      Arc::clone(&self.font_resolver),
      Arc::clone(&self.template_cache),
      options.template,
      options.data,
      assets,
    ))
  }

  #[napi(ts_return_type = "Promise<string>")]
  pub fn render_svg_merged(&self, options: JsRenderOptions) -> AsyncTask<RenderSvgMergedTask> {
    let assets = convert_assets(options.assets);
    AsyncTask::new(RenderSvgMergedTask::new(
      Arc::clone(&self.font_resolver),
      Arc::clone(&self.template_cache),
      options.template,
      options.data,
      assets,
    ))
  }

  #[napi(ts_return_type = "Promise<Array<Buffer>>")]
  pub fn render_png(&self, options: JsRenderPngOptions) -> AsyncTask<RenderPngTask> {
    let assets = convert_assets(options.assets);
    let ppi = options.ppi.unwrap_or(144.0);
    AsyncTask::new(RenderPngTask::new(
      Arc::clone(&self.font_resolver),
      Arc::clone(&self.template_cache),
      options.template,
      options.data,
      assets,
      ppi,
      options.pages,
    ))
  }

  #[napi(ts_return_type = "import('node:stream').Readable")]
  pub fn render_pdf_stream(
    &self,
    env: Env,
    options: JsRenderOptions,
    chunk_size: Option<u32>,
  ) -> napi::Result<Object<'_>> {
    let assets = convert_assets(options.assets);
    let task = RenderPdfTask::new(
      Arc::clone(&self.font_resolver),
      Arc::clone(&self.template_cache),
      options.template,
      options.data,
      assets,
    );
    let promise = env.spawn(task)?;
    let promise_raw = promise.promise_object();
    let chunk_size = chunk_size.unwrap_or(65536);

    let helper: Function<(PromiseRaw<Buffer>, u32), Object> = env.run_script(
      r#"
      (function(promiseOrTuple, maybeChunkSize) {
        let promise = promiseOrTuple;
        let chunkSize = maybeChunkSize;
        if (Array.isArray(promiseOrTuple)) {
          promise = promiseOrTuple[0];
          chunkSize = promiseOrTuple[1];
        }

        const streamMod = (typeof process !== 'undefined' && process.getBuiltinModule)
          ? process.getBuiltinModule('node:stream')
          : (typeof require !== 'undefined' ? require('node:stream') : process.mainModule.require('node:stream'));
        const { Readable } = streamMod;

        // Catch promise early to avoid Node.js unhandledRejection events before stream consumer reads
        if (promise && typeof promise.catch === 'function') {
          promise.catch(() => {});
        } else if (promise && typeof promise.then === 'function') {
          Promise.resolve(promise).catch(() => {});
        }

        let buf = null;
        let offset = 0;
        let reading = false;

        function pushNext(stream) {
          if (!buf || reading) return;
          reading = true;
          while (offset < buf.length) {
            const sliceSize = chunkSize || 65536;
            const end = Math.min(offset + sliceSize, buf.length);
            const chunk = buf.subarray(offset, end);
            offset = end;
            const canContinue = stream.push(chunk);
            if (!canContinue) {
              reading = false;
              return;
            }
          }
          stream.push(null);
          reading = false;
        }

        return new Readable({
          highWaterMark: chunkSize || 65536,
          read() {
            if (!buf) {
              promise.then(
                (res) => {
                  buf = res;
                  pushNext(this);
                },
                (err) => this.destroy(err)
              );
              return;
            }
            pushNext(this);
          }
        });
      })
      "#,
    )?;

    helper.call((promise_raw, chunk_size))
  }

  #[napi(ts_return_type = "import('node:stream').Readable")]
  pub fn render_png_stream(
    &self,
    env: Env,
    options: JsRenderPngOptions,
  ) -> napi::Result<Object<'_>> {
    let assets = convert_assets(options.assets);
    let ppi = options.ppi.unwrap_or(144.0);
    let task = RenderPngTask::new(
      Arc::clone(&self.font_resolver),
      Arc::clone(&self.template_cache),
      options.template,
      options.data,
      assets,
      ppi,
      options.pages,
    );
    let promise = env.spawn(task)?;
    let promise_raw = promise.promise_object();

    let helper: Function<PromiseRaw<Vec<Buffer>>, Object> = env.run_script(
      r#"
      (function(promise) {
        const streamMod = (typeof process !== 'undefined' && process.getBuiltinModule)
          ? process.getBuiltinModule('node:stream')
          : (typeof require !== 'undefined' ? require('node:stream') : process.mainModule.require('node:stream'));
        const { Readable } = streamMod;
        promise.catch(() => {});
        return new Readable({
          objectMode: true,
          async read() {
            if (this._started) return;
            this._started = true;
            try {
              const pages = await promise;
              for (let i = 0; i < pages.length; i++) {
                this.push({ page: i + 1, data: pages[i] });
              }
              this.push(null);
            } catch (err) {
              this.destroy(err);
            }
          }
        });
      })
      "#,
    )?;

    helper.call(promise_raw)
  }

  #[napi(ts_return_type = "import('node:stream').Readable")]
  pub fn render_svg_stream(&self, env: Env, options: JsRenderOptions) -> napi::Result<Object<'_>> {
    let assets = convert_assets(options.assets);
    let task = RenderSvgTask::new(
      Arc::clone(&self.font_resolver),
      Arc::clone(&self.template_cache),
      options.template,
      options.data,
      assets,
    );
    let promise = env.spawn(task)?;
    let promise_raw = promise.promise_object();

    let helper: Function<PromiseRaw<Vec<String>>, Object> = env.run_script(
      r#"
      (function(promise) {
        const streamMod = (typeof process !== 'undefined' && process.getBuiltinModule)
          ? process.getBuiltinModule('node:stream')
          : (typeof require !== 'undefined' ? require('node:stream') : process.mainModule.require('node:stream'));
        const { Readable } = streamMod;
        promise.catch(() => {});
        return new Readable({
          objectMode: true,
          async read() {
            if (this._started) return;
            this._started = true;
            try {
              const pages = await promise;
              for (let i = 0; i < pages.length; i++) {
                this.push({ page: i + 1, svg: pages[i] });
              }
              this.push(null);
            } catch (err) {
              this.destroy(err);
            }
          }
        });
      })
      "#,
    )?;

    helper.call(promise_raw)
  }
}
