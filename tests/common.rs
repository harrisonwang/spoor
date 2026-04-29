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

use gist::{extractors, format::Format, source::Source};
use std::path::Path;

pub fn extract_fixture(rel_path: &str, format: Format) -> String {
    let path = Path::new("tests/fixtures").join(rel_path);
    let source = Source::resolve(path.to_str().unwrap())
        .unwrap_or_else(|e| panic!("failed to read {}: {}", rel_path, e));
    extractors::extract(&source, format)
        .unwrap_or_else(|e| panic!("extract failed on {}: {}", rel_path, e))
}

/// Run extractor, expect failure. Returns the formatted error message.
pub fn extract_fixture_err(rel_path: &str, format: Format) -> String {
    let path = Path::new("tests/fixtures").join(rel_path);
    let source = match Source::resolve(path.to_str().unwrap()) {
        Ok(s) => s,
        Err(e) => return format!("{:#}", e),
    };
    match extractors::extract(&source, format) {
        Ok(_) => panic!("expected error on {}, got Ok", rel_path),
        Err(e) => format!("{:#}", e),
    }
}
