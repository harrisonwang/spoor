use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const TABLE_SCHEMA_VERSION: &str = "spoor-table-json-v2";

/// Self-describing usage hint for consumers. Mentions every narrowing flag
/// so an LLM seeing the JSON knows how to compose the next call without
/// reading `--help`.
pub const TABLE_USAGE: &str = "收窄输出：--sheet <name>、--rows <first:last>（Excel 行号，含两端）、--columns <a,b,c>、--limit <n>、--offset <n>。默认预览 = 每个 table 前 100 条数据行。--rows 与 --limit/--offset 互斥。";

/// Top-level JSON output. Wraps all tables across all input files in one
/// envelope, with self-describing `usage` and `schema_version`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonOutput {
    pub schema_version: String,
    pub usage: String,
    pub tables: Vec<TableEntry>,
    pub truncated: bool,
    pub warnings: Vec<String>,
}

impl JsonOutput {
    pub fn new(tables: Vec<TableEntry>) -> Self {
        Self {
            schema_version: TABLE_SCHEMA_VERSION.to_string(),
            usage: TABLE_USAGE.to_string(),
            tables,
            truncated: false,
            warnings: Vec::new(),
        }
    }
}

/// One table = one CSV file, or one sheet of an XLSX workbook.
/// Self-contained: includes its own source/format so multi-file output
/// flattens into a single `tables[]` array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableEntry {
    pub source: String,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheet: Option<String>,
    /// XLSX only: all sheet names in the workbook (including empty ones).
    /// Repeated on every entry from the same workbook so each entry is
    /// self-describing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workbook_sheets: Option<Vec<String>>,
    /// CSV only: the detected delimiter as a 1-char string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    pub column_count: usize,
    /// XLSX only: 1-based row number of the detected header. CSV always has
    /// header at row 1; we omit the field for CSV to keep the schema tight.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_row: Option<usize>,
    pub headers: BTreeMap<String, HeaderInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preamble: Option<PreambleInfo>,
    pub rows: Vec<BTreeMap<String, String>>,
    pub row_range: RowRange,
    pub truncated: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderInfo {
    pub column_index: usize,
}

impl HeaderInfo {
    pub fn new(column_index: usize) -> Self {
        Self { column_index }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreambleInfo {
    pub row: usize,
    pub content: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowRange {
    pub first: usize,
    pub last: usize,
}

impl RowRange {
    pub fn new(first: usize, last: usize) -> Self {
        Self { first, last }
    }
}

pub fn a1_range(start_row: usize, start_col: usize, end_row: usize, end_col: usize) -> String {
    format!(
        "{}{}:{}{}",
        column_name(start_col),
        start_row,
        column_name(end_col),
        end_row
    )
}

fn column_name(mut index: usize) -> String {
    debug_assert!(index >= 1);
    let mut name = Vec::new();

    while index > 0 {
        index -= 1;
        name.push((b'A' + (index % 26) as u8) as char);
        index /= 26;
    }

    name.iter().rev().collect()
}

/// Convert a row of cells + header list into a field→value BTreeMap.
/// Duplicate header names get `_2`, `_3` suffixes; missing headers fall
/// back to `column_N`. Deterministic key order is via BTreeMap.
pub fn cells_to_values(cells: &[String], headers: &[String]) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    let mut seen = std::collections::HashMap::new();

    for (idx, cell) in cells.iter().enumerate() {
        let base = headers
            .get(idx)
            .map(|h| h.trim())
            .filter(|h| !h.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("column_{}", idx + 1));

        let count = seen.entry(base.clone()).or_insert(0usize);
        *count += 1;

        let key = if *count == 1 {
            base
        } else {
            format!("{base}_{}", *count)
        };

        values.insert(key, cell.clone());
    }

    values
}
