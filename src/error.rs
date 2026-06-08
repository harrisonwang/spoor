use serde::Serialize;
use std::fmt;

pub const IMAGE_ONLY_PDF_REASON: &str = "image-only PDF";
pub const IMAGE_ONLY_PDF_HINT: &str =
    "This PDF has no text layer. OCR is required, but pith does not perform OCR.";
pub const PARSE_MEMORY_LIMIT_REASON: &str = "parse memory limit exceeded";

/// A machine-readable extraction failure for agents and other CLI consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StructuredError {
    pub is_error: bool,
    pub reason: String,
    pub hint: String,
    pub recoverable: bool,
}

impl StructuredError {
    pub fn image_only_pdf() -> Self {
        Self {
            is_error: true,
            reason: IMAGE_ONLY_PDF_REASON.to_string(),
            hint: IMAGE_ONLY_PDF_HINT.to_string(),
            recoverable: true,
        }
    }

    pub fn parse_memory_limit(max_bytes: usize, stage: &str) -> Self {
        Self {
            is_error: true,
            reason: PARSE_MEMORY_LIMIT_REASON.to_string(),
            hint: format!(
                "Parsing exceeded the available data-volume budget of {max_bytes} bytes during {stage}. Narrow the input or rerun with --max-parse-bytes <n>."
            ),
            recoverable: true,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialize structured error")
    }
}

impl fmt::Display for StructuredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}

impl std::error::Error for StructuredError {}
