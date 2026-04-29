//! PDF integration tests.

mod common;
use common::extract_fixture;
use gist::format::Format;
use insta::assert_snapshot;

#[test]
fn basic_text_layer() {
    // Single page, plain text. pdf-extract gives us the text in
    // approximately reading order.
    let out = extract_fixture("pdf/01_basic.pdf", Format::Pdf);
    // Snapshot the trimmed output to avoid platform whitespace differences.
    assert_snapshot!(out.trim());
}

#[test]
fn multipage_concatenated() {
    let out = extract_fixture("pdf/02_multipage.pdf", Format::Pdf);
    assert!(out.contains("Page 1"));
    assert!(out.contains("Page 2"));
    assert!(out.contains("Page 3"));
}

#[test]
fn ascii_baseline() {
    let out = extract_fixture("pdf/03_ascii_only.pdf", Format::Pdf);
    assert!(out.contains("ASCII only"));
}

// IMPORTANT: We deliberately do NOT have a test for image-only PDFs here.
// That's the OCR territory we explicitly excluded. If a user passes an
// image-only PDF, our extractor returns a clear error pointing them at
// pdf-extract returning empty; that's the correct behavior.
