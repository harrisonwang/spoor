use spoor_core::{
    ErrorCode, Format, ParseContent, ParseLimits, ParseRequest, detect_format, parse,
};

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
