use napi::bindgen_prelude::{Buffer, Error, Result, Status};
use napi_derive::napi;
use spoor_core::{Format, ParseLimits, ParseRequest};
use std::str::FromStr;

#[napi(object)]
pub struct ParseOptions {
    pub source_name: Option<String>,
    pub content_type: Option<String>,
    pub format: Option<String>,
    pub max_parse_bytes: Option<i64>,
}

#[napi]
pub fn parse_bytes(data: Buffer, options: Option<ParseOptions>) -> Result<serde_json::Value> {
    let options = options.unwrap_or(ParseOptions {
        source_name: None,
        content_type: None,
        format: None,
        max_parse_bytes: None,
    });
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
    let result = spoor_core::parse(&request).map_err(to_node_error)?;
    serde_json::to_value(result)
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))
}

#[napi]
pub fn detect_format(data: Buffer, source_name: Option<String>) -> Result<String> {
    let mut request = ParseRequest::new(data.as_ref());
    request.source_name = source_name.as_deref();
    spoor_core::detect_format(&request)
        .map(|format| format.to_string())
        .map_err(to_node_error)
}

fn to_node_error(error: spoor_core::SpoorError) -> Error {
    Error::new(Status::GenericFailure, error.to_json())
}
