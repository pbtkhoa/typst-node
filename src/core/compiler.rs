use typst::diag::SourceDiagnostic;
use typst::foundations::{Array, Dict, Str, Value};
use typst::layout::PagedDocument;
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

use crate::error::{CoreError, DiagnosticInfo, DiagnosticSeverity};

pub fn json_to_typst_value(val: serde_json::Value) -> Value {
  match val {
    serde_json::Value::Null => Value::None,
    serde_json::Value::Bool(b) => Value::Bool(b),
    serde_json::Value::Number(num) => {
      if let Some(i) = num.as_i64() {
        Value::Int(i)
      } else if let Some(u) = num.as_u64() {
        if let Ok(i) = i64::try_from(u) {
          Value::Int(i)
        } else {
          Value::Float(u as f64)
        }
      } else if let Some(f) = num.as_f64() {
        Value::Float(f)
      } else {
        Value::None
      }
    }
    serde_json::Value::String(s) => Value::Str(Str::from(s)),
    serde_json::Value::Array(arr) => {
      let items: Vec<Value> = arr.into_iter().map(json_to_typst_value).collect();
      Value::Array(Array::from_iter(items))
    }
    serde_json::Value::Object(map) => {
      let mut dict = Dict::new();
      for (k, v) in map {
        dict.insert(Str::from(k), json_to_typst_value(v));
      }
      Value::Dict(dict)
    }
  }
}

pub fn create_library(data: Option<serde_json::Value>) -> LazyHash<Library> {
  let mut builder = Library::builder();

  if let Some(data) = data {
    let typst_value = json_to_typst_value(data);
    if let Value::Dict(dict) = typst_value {
      builder = builder.with_inputs(dict);
    } else {
      let mut dict = Dict::new();
      dict.insert(Str::from("data"), typst_value);
      builder = builder.with_inputs(dict);
    }
  }

  LazyHash::new(builder.build())
}

pub fn compile(world: &dyn World) -> Result<PagedDocument, CoreError> {
  let warned = typst::compile::<PagedDocument>(world);
  match warned.output {
    Ok(doc) => Ok(doc),
    Err(diags) => {
      let infos = diags
        .into_iter()
        .map(|diag| extract_diagnostic(world, diag))
        .collect();
      Err(CoreError::CompilationFailed(infos))
    }
  }
}

fn extract_diagnostic(world: &dyn World, diag: SourceDiagnostic) -> DiagnosticInfo {
  let severity = match diag.severity {
    typst::diag::Severity::Error => DiagnosticSeverity::Error,
    typst::diag::Severity::Warning => DiagnosticSeverity::Warning,
  };

  let message = diag.message.to_string();
  let hints = diag.hints.into_iter().map(|h| h.to_string()).collect();

  let mut file = None;
  let mut line = None;
  let mut column = None;

  if let Some(id) = diag.span.id() {
    let path = id.vpath().as_rootless_path();
    file = Some(path.to_string_lossy().to_string());

    if let Ok(src) = world.source(id) {
      if let Some(range) = src.range(diag.span) {
        if let Some((l, c)) = src.lines().byte_to_line_column(range.start) {
          line = Some(l + 1);
          column = Some(c + 1);
        }
      }
    }
  }

  DiagnosticInfo {
    severity,
    message,
    file,
    line,
    column,
    hints,
  }
}
