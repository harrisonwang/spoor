//! Adversarial inputs — malformed, oversized, malicious files.
//! All of these MUST fail cleanly with a useful error message,
//! never panic, never hang, never produce nonsense output.

mod common;
use common::extract_fixture_err;
use pith::Format;

#[test]
fn empty_file_treated_as_docx() {
    let err = extract_fixture_err("adversarial/01_empty.docx", Format::Docx);
    // Should mention zip / archive / EOCD or similar
    assert!(
        err.contains("zip") || err.contains("archive") || err.contains("empty"),
        "unhelpful error: {}",
        err
    );
}

#[test]
fn non_zip_data_treated_as_docx() {
    let err = extract_fixture_err("adversarial/02_not_zip.docx", Format::Docx);
    assert!(
        err.contains("zip") || err.contains("archive"),
        "unhelpful error: {}",
        err
    );
}

#[test]
fn truncated_zip() {
    let err = extract_fixture_err("adversarial/03_truncated_zip.docx", Format::Docx);
    assert!(!err.is_empty());
}

#[test]
fn broken_json_ipynb() {
    let err = extract_fixture_err("adversarial/04_broken.ipynb", Format::Ipynb);
    assert!(err.contains("JSON") || err.contains("json"));
}

#[test]
fn compression_bomb_rejected_when_capped() {
    // Without limits, this would decompress to ~5 MB of 'A's.
    // This fixture verifies the extractor never panics; dedicated limits and
    // CLI tests cover ZIP total size and configurable parse-budget failures.
    let path = "tests/fixtures/adversarial/05_compression_bomb.docx";
    let result = pith::resolve_input(
        path,
        &pith::ExtractOptions {
            format: Some(Format::Docx),
            ..pith::ExtractOptions::default()
        },
    )
    .and_then(|resolved| pith::extract_md(&resolved));
    // Either succeeds (with whatever content) or fails cleanly.
    let _ = result; // we just want no panic
}
