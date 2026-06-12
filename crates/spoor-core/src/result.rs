use crate::detect::Format;
use crate::json_schema::TableEntry;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentResult {
    pub source: String,
    pub format: Format,
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableResult {
    pub tables: Vec<TableEntry>,
    pub serialized_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ParseContent {
    Document(DocumentResult),
    Tables(TableResult),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpoorWarning {
    pub code: WarningCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<WarningLocation>,
}

impl SpoorWarning {
    pub fn new(code: WarningCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            location: None,
        }
    }

    pub fn at_page(code: WarningCode, message: impl Into<String>, number: usize) -> Self {
        Self {
            code,
            message: message.into(),
            location: Some(WarningLocation::Page { number }),
        }
    }

    pub fn at_slide(code: WarningCode, message: impl Into<String>, number: usize) -> Self {
        Self {
            code,
            message: message.into(),
            location: Some(WarningLocation::Slide { number }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningCode {
    PdfPageNoTextLayer,
    PdfPageSuspiciousTextLayer,
    MergedTableStructureNotPreserved,
    EmbeddedVisualsOmitted,
}

impl WarningCode {
    pub const ALL: [WarningCode; 4] = [
        WarningCode::PdfPageNoTextLayer,
        WarningCode::PdfPageSuspiciousTextLayer,
        WarningCode::MergedTableStructureNotPreserved,
        WarningCode::EmbeddedVisualsOmitted,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PdfPageNoTextLayer => "pdf_page_no_text_layer",
            Self::PdfPageSuspiciousTextLayer => "pdf_page_suspicious_text_layer",
            Self::MergedTableStructureNotPreserved => "merged_table_structure_not_preserved",
            Self::EmbeddedVisualsOmitted => "embedded_visuals_omitted",
        }
    }
}

impl fmt::Display for WarningCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WarningLocation {
    Page { number: usize },
    Slide { number: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseStats {
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub format: Format,
}

impl ParseStats {
    pub(crate) fn new(input_bytes: usize, output_bytes: usize, format: Format) -> Self {
        Self {
            input_bytes,
            output_bytes,
            format,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseResult {
    pub content: ParseContent,
    pub warnings: Vec<SpoorWarning>,
    pub stats: ParseStats,
}

#[cfg(test)]
mod tests {
    use super::WarningCode;

    #[test]
    fn warning_code_display_matches_wire_format() {
        for code in WarningCode::ALL {
            assert_eq!(
                serde_json::to_string(&code).unwrap(),
                format!("\"{}\"", code.as_str())
            );
            assert_eq!(code.to_string(), code.as_str());
        }
    }
}
