use spoor_core::{
    DocumentFilter, ErrorCode, Format, ParseContent, ParseLimits, ParseRequest, TableFilter,
    detect_format, extract_media, parse,
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
#[cfg(feature = "tables")]
fn table_filter_narrows_rows_and_columns_through_parse() {
    // 01_basic.csv has 3 data rows: Alice(row 2), Bob(row 3), Carol(row 4),
    // columns Name/Score/Note. The filter all bindings now set must flow
    // through `parse()` and select the same slice the CLI's flags do.
    let bytes = include_bytes!("../../spoor-cli/tests/fixtures/csv/01_basic.csv");

    let mut request = ParseRequest::new(bytes);
    request.source_name = Some("data.csv");
    request.table_filter =
        TableFilter::build(None, None, vec!["Name".to_string()], Some(1), Some(1)).unwrap();

    let ParseContent::Tables(tables) = parse(&request).unwrap().content else {
        panic!("expected table result");
    };
    let rows = &tables.tables[0].rows;
    assert_eq!(rows.len(), 1, "offset 1 + limit 1 keeps a single row");
    assert_eq!(rows[0]["Name"], "Bob");
    assert!(
        !rows[0].contains_key("Score"),
        "column filter drops unselected fields"
    );

    // Excel-style row range selects the same row by its 1-based number.
    let mut ranged = ParseRequest::new(bytes);
    ranged.source_name = Some("data.csv");
    ranged.table_filter = TableFilter::build(None, Some((3, 3)), Vec::new(), None, None).unwrap();
    let ParseContent::Tables(tables) = parse(&ranged).unwrap().content else {
        panic!("expected table result");
    };
    assert_eq!(tables.tables[0].rows.len(), 1);
    assert_eq!(tables.tables[0].rows[0]["Name"], "Bob");
}

#[test]
#[cfg(feature = "pdf")]
fn pdf_stats_report_total_page_count_even_when_sliced() {
    // 02_multipage.pdf has 3 pages. A one-page peek must still report the full
    // count, so a caller can learn the document size cheaply, then widen --pages.
    let bytes = include_bytes!("../../spoor-cli/tests/fixtures/pdf/02_multipage.pdf");
    let mut request = ParseRequest::new(bytes);
    request.source_name = Some("doc.pdf");
    request.document_filter = DocumentFilter {
        page_range: Some((1, 1)),
    };

    let result = parse(&request).unwrap();
    assert_eq!(result.stats.page_count, Some(3));
}

#[test]
fn non_paged_formats_report_no_page_count() {
    let mut request = ParseRequest::new(b"hello\n");
    request.source_name = Some("note.txt");
    assert_eq!(parse(&request).unwrap().stats.page_count, None);
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
