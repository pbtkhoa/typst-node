use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
  Error,
  Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DiagnosticInfo {
  pub severity: DiagnosticSeverity,
  pub message: String,
  pub file: Option<String>,
  pub line: Option<usize>,
  pub column: Option<usize>,
  pub hints: Vec<String>,
}

impl fmt::Display for DiagnosticInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let sev = match self.severity {
      DiagnosticSeverity::Error => "error",
      DiagnosticSeverity::Warning => "warning",
    };

    if let Some(ref file) = self.file {
      let line = self.line.unwrap_or(1);
      let col = self.column.unwrap_or(1);
      write!(f, "{sev}: {}\n  --> {file}:{line}:{col}", self.message)?;
    } else {
      write!(f, "{sev}: {}", self.message)?;
    }

    for hint in &self.hints {
      write!(f, "\n  = hint: {hint}")?;
    }

    Ok(())
  }
}

#[derive(Debug, Error)]
pub enum CoreError {
  #[error("Typst compilation failed with {} diagnostic(s):\n{}", .0.len(), format_diagnostics(.0))]
  CompilationFailed(Vec<DiagnosticInfo>),

  #[error("Typst export failed: {0}")]
  ExportFailed(String),

  #[error("Font resolution failed: {0}")]
  FontError(String),

  #[error("File error: {0}")]
  FileError(String),
}

fn format_diagnostics(diagnostics: &[DiagnosticInfo]) -> String {
  diagnostics
    .iter()
    .enumerate()
    .map(|(i, diag)| {
      if diagnostics.len() == 1 {
        diag.to_string()
      } else {
        format!("{}. {}", i + 1, diag)
      }
    })
    .collect::<Vec<_>>()
    .join("\n\n")
}

impl From<CoreError> for napi::Error {
  fn from(err: CoreError) -> Self {
    napi::Error::from_reason(err.to_string())
  }
}
