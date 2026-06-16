//! PDF integration tests.

mod common;
use common::{extract_fixture, extract_fixture_err, parse_fixture};
use insta::assert_snapshot;
use serde_json::json;
use spoor_core::{Format, WarningCode, WarningLocation};

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
fn no_text_and_no_images_returns_structured_error() {
    // A vector-only page has no text layer and no images to hand off, so there
    // is genuinely nothing to extract — the structured error still fires.
    let error = extract_fixture_err("pdf/06_vector_only.pdf", Format::Pdf);
    let value: serde_json::Value = serde_json::from_str(&error).expect("structured JSON error");

    assert_eq!(
        value,
        json!({
            "is_error": true,
            "code": "image_only_pdf",
            "reason": "纯图片 PDF（无文本层）",
            "hint": "该 PDF 没有文本层，需要 OCR，但 spoor 不执行 OCR。",
            "recoverable": true,
            "stage": "parse"
        })
    );
}

#[test]
fn image_only_pdf_is_surfaced_for_vision_instead_of_failing() {
    // A PDF with no text but with images must NOT hard-fail: it renders the page
    // skeleton plus image markers/handles so a vision-capable agent can read it.
    let markdown = extract_fixture("pdf/04_image_only.pdf", Format::Pdf);
    assert!(markdown.contains("## Page 1"), "{markdown}");
    assert!(markdown.contains("PDF image 1 (p1)"), "{markdown}");

    let result = parse_fixture("pdf/04_image_only.pdf", Format::Pdf);
    let codes: Vec<_> = result.warnings.iter().map(|warning| warning.code).collect();
    assert!(codes.contains(&WarningCode::PdfPageNoTextLayer));
    assert!(codes.contains(&WarningCode::EmbeddedVisualsOmitted));
}

#[test]
fn mixed_pdf_reports_page_level_missing_text_and_image() {
    let result = parse_fixture("pdf/05_mixed_text_and_image.pdf", Format::Pdf);

    // Page 2 has no text layer and carries an image, so it draws both a
    // missing-text warning and an embedded-visual warning, each page-located.
    assert_eq!(result.warnings.len(), 2);
    for warning in &result.warnings {
        assert_eq!(warning.location, Some(WarningLocation::Page { number: 2 }));
    }
    let codes: Vec<_> = result.warnings.iter().map(|warning| warning.code).collect();
    assert!(codes.contains(&WarningCode::PdfPageNoTextLayer));
    assert!(codes.contains(&WarningCode::EmbeddedVisualsOmitted));
}
