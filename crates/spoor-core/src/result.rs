use crate::detect::Format;
use crate::json_schema::TableEntry;
use serde::{Deserialize, Serialize};

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
    pub code: String,
    pub message: String,
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
