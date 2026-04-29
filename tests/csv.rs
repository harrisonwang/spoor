//! CSV integration tests.

mod common;
use common::extract_fixture;
use gist::format::Format;
use insta::assert_snapshot;

#[test]
fn basic_comma_separated() {
    // First row → markdown header. All rows padded to max width.
    let out = extract_fixture("csv/01_basic.csv", Format::Csv);
    assert_snapshot!(out);
}

#[test]
fn tab_delimiter_auto_detected() {
    // No need for user to specify; our sniffer counts delimiter occurrences
    // in a sample of the file and picks the most frequent.
    let out = extract_fixture("csv/02_tab_separated.csv", Format::Csv);
    assert_snapshot!(out);
}

#[test]
fn semicolon_delimiter_european_excel() {
    // German/French/etc. Excel uses ';' as the delimiter when the locale
    // uses ',' as decimal separator. Our sniffer must handle this.
    let out = extract_fixture("csv/03_semicolon.csv", Format::Csv);
    assert_snapshot!(out);
}

#[test]
fn gbk_encoding_decoded() {
    // CRITICAL for Chinese users: GBK CSVs are the default Excel export
    // on Windows/Chinese locale. Our chardetng integration handles it.
    let out = extract_fixture("csv/04_gbk.csv", Format::Csv);
    assert_snapshot!(out);
}

#[test]
fn utf8_bom_stripped() {
    // Excel's UTF-8 export adds a BOM. We must not include the BOM
    // character in the first cell of the header.
    let out = extract_fixture("csv/05_utf8_bom.csv", Format::Csv);
    assert_snapshot!(out);
}

#[test]
fn rfc4180_quoted_fields() {
    // Embedded commas, escaped quotes, embedded newlines — handled by
    // the `csv` crate. Newlines inside cells become spaces in our output
    // (markdown tables don't support multi-line cells).
    let out = extract_fixture("csv/06_quoted.csv", Format::Csv);
    assert_snapshot!(out);
}

#[test]
fn empty_file() {
    // Empty CSV → empty output (single trailing newline).
    let out = extract_fixture("csv/07_empty.csv", Format::Csv);
    assert_snapshot!(out);
}

#[test]
fn pipe_delimiter() {
    let out = extract_fixture("csv/08_pipe.csv", Format::Csv);
    assert_snapshot!(out);
}

#[test]
fn ragged_rows_padded() {
    // Rows with fewer cells than the widest row are right-padded with
    // empty cells.
    let out = extract_fixture("csv/09_ragged.csv", Format::Csv);
    assert_snapshot!(out);
}

#[test]
fn large_file_truncated() {
    // 2000-row file should be truncated to MAX_ROWS (1000) with a
    // "_(truncated at N rows)_" footer.
    let out = extract_fixture("csv/10_large.csv", Format::Csv);
    // Don't snapshot the entire 1000-row table; check its summary.
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() > 1000, "expected truncated table");
    assert!(out.contains("truncated"), "expected truncation footer");
}
