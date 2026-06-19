use napi::bindgen_prelude::{Buffer, Error, Result, Status};
use napi_derive::napi;
use spoor_core::{DocumentFilter, Format, ParseLimits, ParseRequest, TableFilter};
use std::str::FromStr;

#[derive(Default)]
#[napi(object)]
pub struct ParseOptions {
    pub source_name: Option<String>,
    pub content_type: Option<String>,
    pub format: Option<String>,
    pub max_parse_bytes: Option<i64>,
    /// XLSX only: restrict output to one sheet by name.
    pub sheet: Option<String>,
    /// Inclusive 1-based `[first, last]` row range (Excel rows for XLSX, line
    /// numbers for CSV). Mutually exclusive with `limit`/`offset`.
    pub rows: Option<Vec<u32>>,
    /// Keep only these columns, by header name.
    pub columns: Option<Vec<String>>,
    /// Max data rows per table (default 100).
    pub limit: Option<u32>,
    /// Skip this many data rows before applying `limit`.
    pub offset: Option<u32>,
    /// PDF only: inclusive 1-based `[first, last]` page range to parse.
    pub pages: Option<Vec<u32>>,
}

#[napi]
pub fn parse_bytes(data: Buffer, options: Option<ParseOptions>) -> Result<serde_json::Value> {
    let options = options.unwrap_or_default();
    let mut request = build_request(&data, &options)?;
    request.table_filter = table_filter(&options)?;
    request.document_filter =
        DocumentFilter::build_from_page_slice(options.pages.as_deref()).map_err(to_node_error)?;
    let result = spoor_core::parse(&request).map_err(to_node_error)?;
    serde_json::to_value(result)
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))
}

/// Extract one safe embedded media resource referenced by a URI emitted in the
/// parsed output (e.g. `spoor-docx://word/media/image1.png` or
/// `spoor-pdf://obj/{id}/{gen}`). Returns the raw resource bytes. spoor does not
/// decode or interpret the bytes.
#[napi]
pub fn extract_media(
    data: Buffer,
    resource: String,
    options: Option<ParseOptions>,
) -> Result<Buffer> {
    let options = options.unwrap_or_default();
    let request = build_request(&data, &options)?;
    let bytes = spoor_core::extract_media(&request, &resource).map_err(to_node_error)?;
    Ok(Buffer::from(bytes))
}

#[napi]
pub fn detect_format(data: Buffer, source_name: Option<String>) -> Result<String> {
    let mut request = ParseRequest::new(data.as_ref());
    request.source_name = source_name.as_deref();
    spoor_core::detect_format(&request)
        .map(|format| format.to_string())
        .map_err(to_node_error)
}

fn build_request<'a>(data: &'a Buffer, options: &'a ParseOptions) -> Result<ParseRequest<'a>> {
    let mut request = ParseRequest::new(data.as_ref());
    request.source_name = options.source_name.as_deref();
    request.content_type = options.content_type.as_deref();
    request.format_hint = options
        .format
        .as_deref()
        .map(Format::from_str)
        .transpose()
        .map_err(to_node_error)?;
    if let Some(max_parse_bytes) = options.max_parse_bytes {
        request.limits = ParseLimits {
            max_parse_bytes: usize::try_from(max_parse_bytes)
                .map_err(|_| Error::new(Status::InvalidArg, "max_parse_bytes must be positive"))?,
        };
    }
    Ok(request)
}

fn table_filter(options: &ParseOptions) -> Result<TableFilter> {
    TableFilter::build_from_row_slice(
        options.sheet.clone(),
        options.rows.as_deref(),
        options.columns.clone().unwrap_or_default(),
        options.limit.map(|n| n as usize),
        options.offset.map(|n| n as usize),
    )
    .map_err(to_node_error)
}

fn to_node_error(error: spoor_core::SpoorError) -> Error {
    Error::new(Status::GenericFailure, error.to_json())
}
