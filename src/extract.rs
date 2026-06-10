use crate::error::StructuredError;
use crate::extractors;
use crate::format::{self, Format};
use crate::json_schema::TableEntry;
use crate::limits::{DEFAULT_MAX_PARSE_BYTES, ensure_parse_size};
use crate::source::{Source, SourceInput};
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct ExtractOptions {
    pub format: Option<Format>,
    pub max_parse_bytes: usize,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            format: None,
            max_parse_bytes: DEFAULT_MAX_PARSE_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedDocument {
    pub source: String,
    pub format: Format,
    pub markdown: String,
}

/// Table entries plus the JSON size that was measured to enforce the parse
/// budget. The size is returned so batch callers can track a cumulative budget
/// without serializing the same entries a second time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedTables {
    pub entries: Vec<TableEntry>,
    pub serialized_bytes: usize,
}

/// Narrowing filter for table JSON output. Empty/None fields mean "no
/// filter, return the default preview". Constructed by the CLI from
/// `--sheet`, `--rows`, `--columns`, `--limit`, `--offset`.
#[derive(Debug, Clone, Default)]
pub struct TableFilter {
    /// XLSX: keep only the sheet with this name; error if not present.
    /// CSV: no-op (CSV has no sheet concept).
    pub sheet: Option<String>,
    /// Inclusive Excel row range (1-based file row numbers). Filters data
    /// rows whose row number falls in `[first, last]`. Header / title /
    /// preamble rows are unaffected.
    pub row_range: Option<(usize, usize)>,
    /// Keep only these column names. Error if any name is missing from a
    /// table that this filter applies to.
    pub columns: Vec<String>,
    /// Max data rows per table (after `offset`). Default 100 when None.
    pub limit: Option<usize>,
    /// Skip this many data rows before counting `limit`. Default 0.
    pub offset: Option<usize>,
}

pub struct ResolvedInput {
    pub label: String,
    pub source: Source,
    pub format: Format,
    pub max_parse_bytes: usize,
}

pub fn resolve_input(
    input: impl Into<SourceInput>,
    options: &ExtractOptions,
) -> Result<ResolvedInput> {
    let input = input.into();
    let label = input.as_str().to_string();
    let source = Source::resolve_with_limit(input.as_str(), options.max_parse_bytes)
        .with_context(|| format!("failed to resolve input: {label}"))?;

    let format = match options.format {
        Some(format) => format,
        // Preserve a structured `unsupported_format` error so the agent can
        // branch on `code`; only wrap unstructured detection failures.
        None => match format::detect(&source) {
            Ok(format) => format,
            Err(error) if error.downcast_ref::<StructuredError>().is_some() => return Err(error),
            Err(error) => {
                return Err(error.context(format!("could not detect format for: {label}")));
            }
        },
    };

    Ok(ResolvedInput {
        label,
        source,
        format,
        max_parse_bytes: options.max_parse_bytes,
    })
}

pub fn extract_md(resolved: &ResolvedInput) -> Result<ExtractedDocument> {
    let markdown =
        match extractors::extract(&resolved.source, resolved.format, resolved.max_parse_bytes) {
            Ok(markdown) => markdown,
            Err(error) if error.downcast_ref::<StructuredError>().is_some() => return Err(error),
            Err(error) => {
                return Err(error.context(format!("extraction failed ({})", resolved.format)));
            }
        };
    ensure_parse_size(
        markdown.len(),
        resolved.max_parse_bytes,
        "extracted document text",
    )?;

    Ok(ExtractedDocument {
        source: resolved.label.clone(),
        format: resolved.format,
        markdown,
    })
}

pub fn extract_table_entries(
    resolved: &ResolvedInput,
    filter: &TableFilter,
) -> Result<ExtractedTables> {
    let entries = extractors::extract_table_entries(
        &resolved.source,
        resolved.format,
        &resolved.label,
        filter,
        resolved.max_parse_bytes,
    )
    .with_context(|| format!("table JSON extraction failed ({})", resolved.format))?;
    let serialized_bytes = serialized_size(&entries)?;
    ensure_parse_size(
        serialized_bytes,
        resolved.max_parse_bytes,
        "extracted table data",
    )?;
    Ok(ExtractedTables {
        entries,
        serialized_bytes,
    })
}

fn serialized_size(value: &impl serde::Serialize) -> Result<usize> {
    struct Counter(usize);

    impl std::io::Write for Counter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.saturating_add(buf.len());
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, value)?;
    Ok(counter.0)
}
