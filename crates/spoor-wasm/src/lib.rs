use spoor_core::{Format, ParseLimits, ParseRequest};
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

#[wasm_bindgen]
pub fn parse_bytes(
    bytes: &[u8],
    source_name: Option<String>,
    content_type: Option<String>,
    format: Option<String>,
    max_parse_bytes: Option<usize>,
) -> Result<JsValue, JsValue> {
    let request = request(
        bytes,
        source_name.as_deref(),
        content_type.as_deref(),
        format.as_deref(),
        max_parse_bytes,
    )?;
    let result = spoor_core::parse(&request).map_err(error_value)?;
    serde_wasm_bindgen::to_value(&result).map_err(|error| JsValue::from_str(&error.to_string()))
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
