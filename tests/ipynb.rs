//! IPYNB integration tests.

mod common;
use common::{extract_fixture, extract_fixture_err};
use gist::format::Format;
use insta::assert_snapshot;

#[test]
fn markdown_and_code_cells() {
    // Markdown cells: passthrough.
    // Code cells: wrapped in ```<lang>\n...\n```
    // Outputs: SKIPPED (intentional — they're noisy and often binary).
    let out = extract_fixture("ipynb/01_basic.ipynb", Format::Ipynb);
    assert_snapshot!(out);
}

#[test]
fn source_can_be_string_or_array() {
    // nbformat allows `source` to be a string OR a list of strings (each
    // including its trailing \n). Both must work.
    let out = extract_fixture("ipynb/02_source_formats.ipynb", Format::Ipynb);
    assert_snapshot!(out);
}

#[test]
fn language_hint_from_kernelspec() {
    // We use metadata.kernelspec.language for the fence label
    // (e.g. ```r). Falls back to empty if missing.
    let out = extract_fixture("ipynb/03_language_hint.ipynb", Format::Ipynb);
    assert_snapshot!(out);
}

#[test]
fn raw_cells_skipped() {
    // raw cells are silently dropped (matches extract-text).
    let out = extract_fixture("ipynb/04_raw_cells.ipynb", Format::Ipynb);
    assert_snapshot!(out);
}

#[test]
fn malformed_ipynb_returns_clear_error() {
    let err = extract_fixture_err("ipynb/05_malformed.ipynb", Format::Ipynb);
    assert!(err.contains("cells"), "got: {}", err);
}
