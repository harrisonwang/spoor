use spoor_core::{Format, ParseLimits, ParseRequest, TableFilter};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn detect_format(
    bytes: &[u8],
    source_name: Option<String>,
    content_type: Option<String>,
) -> Result<String, JsValue> {
    let request = request(
        bytes,
        source_name.as_deref(),
        content_type.as_deref(),
        None,
        None,
    )?;
    spoor_core::detect_format(&request)
        .map(|format| format.to_string())
        .map_err(error_value)
}

/// Parse document/table bytes into a typed `ParseResult`.
///
/// For table formats (CSV/XLSX) the trailing options mirror the CLI and the
/// other bindings: `sheet` (XLSX only), `rows` as an inclusive 1-based
/// `[first, last]` pair (mutually exclusive with `limit`/`offset`), `columns`
/// to keep, and `limit`/`offset` for pagination. They are ignored for document
/// formats. All are optional, so existing 5-argument calls keep working.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn parse_bytes(
    bytes: &[u8],
    source_name: Option<String>,
    content_type: Option<String>,
    format: Option<String>,
    max_parse_bytes: Option<usize>,
    sheet: Option<String>,
    rows: Option<Vec<u32>>,
    columns: Option<Vec<String>>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<JsValue, JsValue> {
    let mut request = request(
        bytes,
        source_name.as_deref(),
        content_type.as_deref(),
        format.as_deref(),
        max_parse_bytes,
    )?;
    let rows = match rows {
        Some(pair) => {
            let [first, last] = pair.as_slice() else {
                return Err(JsValue::from_str("rows must be a [first, last] pair"));
            };
            Some((*first as usize, *last as usize))
        }
        None => None,
    };
    request.table_filter =
        TableFilter::build(sheet, rows, columns.unwrap_or_default(), limit, offset)
            .map_err(error_value)?;
    let result = spoor_core::parse(&request).map_err(error_value)?;
    serde_wasm_bindgen::to_value(&result).map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Extract one safe embedded media resource referenced by a URI emitted in the
/// parsed output (`spoor-docx://word/media/*` or `spoor-pdf://obj/{id}/{gen}`).
/// Returns the raw resource bytes as a `Uint8Array`. Lets browser and edge
/// callers resolve image placeholders without filesystem access. spoor does not
/// decode or interpret the bytes.
#[wasm_bindgen]
pub fn extract_media(
    bytes: &[u8],
    resource: String,
    source_name: Option<String>,
    content_type: Option<String>,
    format: Option<String>,
    max_parse_bytes: Option<usize>,
) -> Result<Vec<u8>, JsValue> {
    let request = request(
        bytes,
        source_name.as_deref(),
        content_type.as_deref(),
        format.as_deref(),
        max_parse_bytes,
    )?;
    spoor_core::extract_media(&request, &resource).map_err(error_value)
}

fn request<'a>(
    bytes: &'a [u8],
    source_name: Option<&'a str>,
    content_type: Option<&'a str>,
    format: Option<&str>,
    max_parse_bytes: Option<usize>,
) -> Result<ParseRequest<'a>, JsValue> {
    let mut request = ParseRequest::new(bytes);
    request.source_name = source_name;
    request.content_type = content_type;
    request.format_hint = format
        .map(Format::from_str)
        .transpose()
        .map_err(error_value)?;
    if let Some(max_parse_bytes) = max_parse_bytes {
        request.limits = ParseLimits { max_parse_bytes };
    }
    Ok(request)
}

fn error_value(error: spoor_core::SpoorError) -> JsValue {
    serde_wasm_bindgen::to_value(&error).unwrap_or_else(|_| JsValue::from_str(&error.to_json()))
}
