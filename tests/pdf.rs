//! PDF integration tests.

mod common;
use common::{extract_fixture, extract_fixture_err};
use insta::assert_snapshot;
use pith::Format;
use serde_json::json;

#[test]
fn basic_text_layer() {
    // Single page, plain text. pdf-extract gives us the text in
    // approximately reading order.
    let out = extract_fixture("pdf/01_basic.pdf", Format::Pdf);
    assert_snapshot!(out);
}

#[test]
fn multipage_has_page_boundaries() {
    let out = extract_fixture("pdf/02_multipage.pdf", Format::Pdf);
    assert_eq!(out.matches("## Page ").count(), 3);
    assert!(out.starts_with("## Page 1\n\nPage 1 content begins here."));
    assert!(out.contains("\n\n## Page 2\n\nPage 2 content begins here."));
    assert!(out.contains("\n\n## Page 3\n\nPage 3 content begins here."));
}

#[test]
fn ascii_baseline() {
    let out = extract_fixture("pdf/03_ascii_only.pdf", Format::Pdf);
    assert!(out.contains("ASCII only"));
}

#[test]
fn image_only_pdf_returns_structured_error() {
    let error = extract_fixture_err("pdf/04_image_only.pdf", Format::Pdf);
    let value: serde_json::Value = serde_json::from_str(&error).expect("structured JSON error");

    assert_eq!(
        value,
        json!({
            "is_error": true,
            "code": "image_only_pdf",
            "reason": "纯图片 PDF（无文本层）",
            "hint": "该 PDF 没有文本层，需要 OCR，但 pith 不执行 OCR。",
            "recoverable": true
        })
    );
}
