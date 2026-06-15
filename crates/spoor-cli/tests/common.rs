#![allow(dead_code)]

//! Common test helpers.
//!
//! We use `insta` for snapshot testing. Each fixture is run through the
//! corresponding extractor and the resulting markdown is snapshotted to
//! `tests/snapshots/<format>__<name>.snap`.
//!
//! On first run, snapshots are created. On subsequent runs, output is
//! diffed against the snapshot. To accept a change:
//!
//!     cargo insta review
//!
//! Or non-interactively:
//!
//!     INSTA_UPDATE=always cargo test

use spoor_core::{Format, ParseRequest, ParseResult, parse, parse_document};
use std::path::Path;

pub fn extract_fixture(rel_path: &str, format: Format) -> String {
    let path = Path::new("tests/fixtures").join(rel_path);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read failed on {rel_path}: {error}"));
    let mut request = ParseRequest::new(&bytes);
    request.source_name = path.to_str();
    request.format_hint = Some(format);
    parse_document(&request)
        .map(|document| document.markdown)
        .unwrap_or_else(|e| panic!("extract failed on {}: {}", rel_path, e))
}

/// Extract a fixture as if it had been fetched from `source_url`, so relative
/// links/images resolve against that URL — the real `spoor https://…` path.
pub fn extract_fixture_from_url(rel_path: &str, format: Format, source_url: &str) -> String {
    let path = Path::new("tests/fixtures").join(rel_path);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read failed on {rel_path}: {error}"));
    let mut request = ParseRequest::new(&bytes);
    request.source_name = Some(source_url);
    request.format_hint = Some(format);
    parse_document(&request)
        .map(|document| document.markdown)
        .unwrap_or_else(|e| panic!("extract failed on {}: {}", rel_path, e))
}

pub fn parse_fixture(rel_path: &str, format: Format) -> ParseResult {
    let path = Path::new("tests/fixtures").join(rel_path);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read failed on {rel_path}: {error}"));
    let mut request = ParseRequest::new(&bytes);
    request.source_name = path.to_str();
    request.format_hint = Some(format);
    parse(&request).unwrap_or_else(|e| panic!("parse failed on {}: {}", rel_path, e))
}

/// Run extractor, expect failure. Returns the formatted error message.
pub fn extract_fixture_err(rel_path: &str, format: Format) -> String {
    let path = Path::new("tests/fixtures").join(rel_path);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => return format!("{error:#}"),
    };
    let mut request = ParseRequest::new(&bytes);
    request.source_name = path.to_str();
    request.format_hint = Some(format);
    match parse_document(&request) {
        Ok(_) => panic!("expected error on {}, got Ok", rel_path),
        Err(e) => format!("{:#}", e),
    }
}
