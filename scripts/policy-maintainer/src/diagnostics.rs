use std::{
    fmt,
    io::{self, Write},
};

use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputMode {
    Text,
    Json,
}

impl OutputMode {
    pub const fn from_json(json: bool) -> Self {
        if json { Self::Json } else { Self::Text }
    }
}

#[derive(Debug, Error)]
pub enum DiagnosticError {
    #[error("failed to serialize diagnostic as JSON")]
    Serialize(#[from] serde_json::Error),
    #[error("failed to write diagnostic")]
    Write(#[from] io::Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Info,
    Warn,
    Error,
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => f.write_str("info"),
            Self::Warn => f.write_str("warn"),
            Self::Error => f.write_str("error"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: String,
    pub message: String,
}

impl Diagnostic {
    pub fn new(
        level: DiagnosticLevel,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            level,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(DiagnosticLevel::Info, code, message)
    }

    pub fn warn(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(DiagnosticLevel::Warn, code, message)
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(DiagnosticLevel::Error, code, message)
    }

    pub fn render(&self, output_mode: OutputMode) -> Result<String, DiagnosticError> {
        match output_mode {
            OutputMode::Text => Ok(self.to_string()),
            OutputMode::Json => serde_json::to_string(self).map_err(DiagnosticError::from),
        }
    }

    pub fn emit(
        &self,
        output_mode: OutputMode,
        writer: &mut impl Write,
    ) -> Result<(), DiagnosticError> {
        writeln!(writer, "{}", self.render(output_mode)?)?;
        Ok(())
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.level, self.code, self.message)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{Diagnostic, OutputMode};

    #[test]
    fn text_render_includes_level_code_and_message() {
        let diagnostic = Diagnostic::info("PM0001", "policy-maintainer skeleton operational");

        assert_eq!(
            diagnostic.render(OutputMode::Text).unwrap(),
            "info PM0001: policy-maintainer skeleton operational"
        );
    }

    #[test]
    fn json_render_emits_structured_single_line_diagnostic() {
        let diagnostic = Diagnostic::warn("PM0002", "manifest is missing an optional entry");
        let rendered = diagnostic.render(OutputMode::Json).unwrap();
        let value: Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(value["level"], "warn");
        assert_eq!(value["code"], "PM0002");
        assert_eq!(value["message"], "manifest is missing an optional entry");
        assert!(!rendered.contains('\n'));
    }

    #[test]
    fn emit_writes_one_diagnostic_line() {
        let diagnostic = Diagnostic::error("PM0003", "manifest is invalid");
        let mut output = Vec::new();

        diagnostic.emit(OutputMode::Text, &mut output).unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "error PM0003: manifest is invalid\n"
        );
    }
}
