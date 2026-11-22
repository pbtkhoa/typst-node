use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;

#[napi(object)]
pub struct JsTypstRendererOptions {
  pub font_paths: Option<Vec<String>>,
  pub include_system_fonts: Option<bool>,
  pub enable_cache: Option<bool>,
  pub cache_capacity: Option<u32>,
}

#[napi(object)]
pub struct JsRenderOptions {
  pub template: String,
  pub data: Option<serde_json::Value>,
  pub assets: Option<HashMap<String, Buffer>>,
}

#[napi(object)]
pub struct JsRenderPngOptions {
  pub template: String,
  pub data: Option<serde_json::Value>,
  pub assets: Option<HashMap<String, Buffer>>,
  pub ppi: Option<f64>,
  pub pages: Option<Vec<u32>>,
}
