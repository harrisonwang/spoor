use crate::detect::{self, Format};
use crate::error::{ParseStage, SpoorError};
use crate::limits::{DEFAULT_MAX_PARSE_BYTES, ensure_parse_size};
use crate::parse as parsers;
use crate::result::{DocumentResult, ParseContent, ParseResult, ParseStats, TableResult};
use crate::source::Source;
use serde::{Deserialize, Serialize};

pub type SpoorResult<T> = std::result::Result<T, SpoorError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseLimits {
    pub max_parse_bytes: usize,
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_parse_bytes: DEFAULT_MAX_PARSE_BYTES,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableFilter {
    pub sheet: Option<String>,
    pub row_range: Option<(usize, usize)>,
    pub columns: Vec<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ParseRequest<'a> {
    pub bytes: &'a [u8],
    pub source_name: Option<&'a str>,
    pub content_type: Option<&'a str>,
    pub format_hint: Option<Format>,
    pub table_filter: TableFilter,
    pub limits: ParseLimits,
}

impl<'a> ParseRequest<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            source_name: None,
            content_type: None,
            format_hint: None,
            table_filter: TableFilter::default(),
            limits: ParseLimits::default(),
        }
    }
}

pub fn detect_format(request: &ParseRequest<'_>) -> SpoorResult<Format> {
    catch_boundary(ParseStage::Detect, || detect_format_inner(request))
}

fn detect_format_inner(request: &ParseRequest<'_>) -> SpoorResult<Format> {
    ensure_parse_size(
        request.bytes.len(),
        request.limits.max_parse_bytes,
        "input bytes",
    )
    .map_err(|error| SpoorError::from_anyhow(error, ParseStage::Limits))?;
    if let Some(format) = request.format_hint {
        return Ok(format);
    }
    detect::detect(&source(request))
        .map_err(|error| SpoorError::from_anyhow(error, ParseStage::Detect))
}

pub fn parse(request: &ParseRequest<'_>) -> SpoorResult<ParseResult> {
    catch_boundary(ParseStage::Parse, || parse_inner(request))
}

fn parse_inner(request: &ParseRequest<'_>) -> SpoorResult<ParseResult> {
    let format = detect_format(request)?;
    if format.is_table() {
        let tables = parse_tables_with_format(request, format)?;
        let output_bytes = tables.serialized_bytes;
        Ok(ParseResult {
            content: ParseContent::Tables(tables),
            warnings: Vec::new(),
            stats: ParseStats::new(request.bytes.len(), output_bytes, format),
        })
    } else {
        let document = parse_document_with_format(request, format)?;
        let output_bytes = document.markdown.len();
        Ok(ParseResult {
            content: ParseContent::Document(document),
            warnings: Vec::new(),
            stats: ParseStats::new(request.bytes.len(), output_bytes, format),
        })
    }
}

pub fn parse_document(request: &ParseRequest<'_>) -> SpoorResult<DocumentResult> {
    catch_boundary(ParseStage::Parse, || {
        let format = detect_format(request)?;
        parse_document_with_format(request, format)
    })
}

pub fn parse_tables(request: &ParseRequest<'_>) -> SpoorResult<TableResult> {
    catch_boundary(ParseStage::Parse, || {
        let format = detect_format(request)?;
        parse_tables_with_format(request, format)
    })
}

fn parse_document_with_format(
    request: &ParseRequest<'_>,
    format: Format,
) -> SpoorResult<DocumentResult> {
    let markdown = parsers::extract(&source(request), format, request.limits.max_parse_bytes)
        .map_err(|error| SpoorError::from_anyhow(error, ParseStage::Parse))?;
    ensure_parse_size(
        markdown.len(),
        request.limits.max_parse_bytes,
        "extracted document text",
    )
    .map_err(|error| SpoorError::from_anyhow(error, ParseStage::Limits))?;

    Ok(DocumentResult {
        source: source_label(request).to_string(),
        format,
        markdown,
    })
}

fn parse_tables_with_format(
    request: &ParseRequest<'_>,
    format: Format,
) -> SpoorResult<TableResult> {
    let entries = parsers::extract_table_entries(
        &source(request),
        format,
        source_label(request),
        &request.table_filter,
        request.limits.max_parse_bytes,
    )
    .map_err(|error| SpoorError::from_anyhow(error, ParseStage::Parse))?;
    let serialized_bytes = serialized_size(&entries)
        .map_err(|error| SpoorError::from_anyhow(error, ParseStage::Render))?;
    ensure_parse_size(
        serialized_bytes,
        request.limits.max_parse_bytes,
        "extracted table data",
    )
    .map_err(|error| SpoorError::from_anyhow(error, ParseStage::Limits))?;

    Ok(TableResult {
        tables: entries,
        serialized_bytes,
    })
}

fn source<'a>(request: &'a ParseRequest<'a>) -> Source<'a> {
    Source::new(request.bytes, request.source_name, request.content_type)
}

fn source_label<'a>(request: &'a ParseRequest<'a>) -> &'a str {
    request.source_name.unwrap_or("<bytes>")
}

fn serialized_size(value: &impl serde::Serialize) -> anyhow::Result<usize> {
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

fn catch_boundary<T>(
    stage: ParseStage,
    operation: impl FnOnce() -> SpoorResult<T>,
) -> SpoorResult<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).unwrap_or_else(|payload| {
        Err(SpoorError::parse_failed(
            format!("解析器内部异常：{}", panic_reason(payload.as_ref())),
            stage,
        ))
    })
}

fn panic_reason(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("未知 panic")
}

pub type ExtractedDocument = DocumentResult;
pub type ExtractedTables = TableResult;

#[cfg(test)]
mod tests {
    use super::{ParseStage, catch_boundary};
    use crate::ErrorCode;

    #[test]
    fn public_boundary_normalizes_parser_panics() {
        let error = catch_boundary::<()>(ParseStage::Parse, || {
            panic!("malformed parser input");
        })
        .expect_err("panic must become a structured error");

        assert_eq!(error.code, ErrorCode::ParseFailed);
        assert_eq!(error.stage, Some(ParseStage::Parse));
        assert!(error.reason.contains("malformed parser input"));
    }
}
