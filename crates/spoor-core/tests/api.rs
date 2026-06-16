use spoor_core::{
    ErrorCode, Format, ParseContent, ParseLimits, ParseRequest, detect_format, extract_media, parse,
};
#[cfg(feature = "pdf")]
use spoor_core::{WarningCode, WarningLocation, parse_document_result};

#[test]
fn bytes_only_document_api_returns_typed_result() {
    let mut request = ParseRequest::new(b"hello from core\n");
    request.source_name = Some("note.txt");

    assert_eq!(detect_format(&request).unwrap(), Format::PlainText);
    let result = parse(&request).unwrap();
    assert_eq!(result.stats.input_bytes, 16);
    match result.content {
        ParseContent::Document(document) => {
            assert_eq!(document.source, "note.txt");
            assert_eq!(document.markdown, "hello from core\n");
        }
        ParseContent::Tables(_) => panic!("expected document result"),
    }
}

#[test]
#[cfg(feature = "tables")]
fn bytes_only_table_api_returns_native_tables() {
    let bytes = include_bytes!("../../spoor-cli/tests/fixtures/csv/01_basic.csv");
    let mut request = ParseRequest::new(bytes);
    request.source_name = Some("data.csv");

    let result = parse(&request).unwrap();
    match result.content {
        ParseContent::Tables(tables) => {
            assert_eq!(tables.tables.len(), 1);
            assert_eq!(tables.tables[0].format, "csv");
        }
        ParseContent::Document(_) => panic!("expected table result"),
    }
}

#[test]
fn public_boundary_normalizes_unstructured_parser_errors() {
    let mut request = ParseRequest::new(br#"{"not":"a notebook"}"#);
    request.source_name = Some("bad.ipynb");
    request.format_hint = Some(Format::Ipynb);

    let error = parse(&request).unwrap_err();
    assert_eq!(error.code, ErrorCode::ParseFailed);
    assert_eq!(error.stage, Some(spoor_core::ParseStage::Parse));
}

#[test]
fn parse_budget_is_enforced_before_detection() {
    let mut request = ParseRequest::new(&[b'x'; 2048]);
    request.limits = ParseLimits {
        max_parse_bytes: 1024,
    };

    let error = parse(&request).unwrap_err();
    assert_eq!(error.code, ErrorCode::ParseBudgetExceeded);
}

#[test]
#[cfg(feature = "office")]
fn extract_media_uses_safe_format_specific_resource_uris() {
    let bytes = include_bytes!("../../spoor-cli/tests/fixtures/docx/16_image_placeholders.docx");
    let mut request = ParseRequest::new(bytes);
    request.source_name = Some("images.docx");

    let image = extract_media(&request, "spoor-docx://word/media/image1.png").unwrap();
    assert_eq!(image, b"first-image");

    let error = extract_media(&request, "word/media/image1.png").unwrap_err();
    assert_eq!(error.code, ErrorCode::ParseFailed);
}

#[test]
#[cfg(feature = "pdf")]
fn document_result_api_preserves_structured_warning_locations() {
    let bytes = include_bytes!("../../spoor-cli/tests/fixtures/pdf/05_mixed_text_and_image.pdf");
    let mut request = ParseRequest::new(bytes);
    request.source_name = Some("mixed.pdf");

    let result = parse_document_result(&request).unwrap();

    // Page 2 lacks a text layer and carries an image: a missing-text warning
    // followed by an embedded-visual warning, both page-located.
    assert_eq!(result.warnings.len(), 2);
    assert_eq!(result.warnings[0].code, WarningCode::PdfPageNoTextLayer);
    assert_eq!(result.warnings[1].code, WarningCode::EmbeddedVisualsOmitted);
    for warning in &result.warnings {
        assert_eq!(warning.location, Some(WarningLocation::Page { number: 2 }));
    }
    let serialized = serde_json::to_value(result).unwrap();
    assert_eq!(serialized["warnings"][0]["location"]["kind"], "page");
    assert_eq!(serialized["warnings"][0]["location"]["number"], 2);
}
