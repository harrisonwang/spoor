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

use pith::{ExtractOptions, Format, SourceInput, extract_md, resolve_input};
use std::path::Path;

pub fn extract_fixture(rel_path: &str, format: Format) -> String {
    let path = Path::new("tests/fixtures").join(rel_path);
    let options = ExtractOptions {
        format: Some(format),
        ..ExtractOptions::default()
    };
    let resolved = resolve_input(
        SourceInput::from(path.to_string_lossy().into_owned()),
        &options,
    )
    .unwrap_or_else(|e| panic!("resolve failed on {}: {}", rel_path, e));
    extract_md(&resolved)
        .map(|document| document.markdown)
        .unwrap_or_else(|e| panic!("extract failed on {}: {}", rel_path, e))
}

/// Run extractor, expect failure. Returns the formatted error message.
pub fn extract_fixture_err(rel_path: &str, format: Format) -> String {
    let path = Path::new("tests/fixtures").join(rel_path);
    let options = ExtractOptions {
        format: Some(format),
        ..ExtractOptions::default()
    };
    let resolved = match resolve_input(
        SourceInput::from(path.to_string_lossy().into_owned()),
        &options,
    ) {
        Ok(r) => r,
        Err(e) => return format!("{:#}", e),
    };
    match extract_md(&resolved) {
        Ok(_) => panic!("expected error on {}, got Ok", rel_path),
        Err(e) => format!("{:#}", e),
    }
}
