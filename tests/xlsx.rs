//! XLSX integration tests.

mod common;
use common::extract_fixture;
use gist::format::Format;
use insta::assert_snapshot;

#[test]
fn basic_sheet_as_gfm_table() {
    // First row treated as header, separator line emitted.
    // Note: extract-text uses TSV (tab-separated), no header separator.
    // We choose GFM tables — they're more LLM-friendly because column
    // alignment is explicit.
    let out = extract_fixture("xlsx/01_basic.xlsx", Format::Xlsx);
    assert_snapshot!(out);
}

#[test]
fn multiple_sheets_with_empty_one() {
    // Each sheet gets a "## Sheet: <name>" header. Empty sheets still
    // get the header (extract-text behavior we keep).
    let out = extract_fixture("xlsx/02_multi_sheets.xlsx", Format::Xlsx);
    assert_snapshot!(out);
}

#[test]
fn cell_types_numbers_dates_bools_formulas() {
    // IMPORTANT: extract-text outputs Excel's serial-date number for date
    // cells (e.g. 45672.604). This is essentially useless for an LLM.
    // We CHOOSE to format dates as ISO 8601 (e.g. 2025-01-15T14:30:00).
    // calamine returns DateTime types directly so we can format them.
    //
    // Booleans render as TRUE/FALSE (uppercase, like extract-text).
    // Formulas use cached <v> when present.
    // Big floats: 1e-07 (Rust default Display).
    // Trailing-zero floats: 1000000 (no decimal).
    let out = extract_fixture("xlsx/03_types.xlsx", Format::Xlsx);
    assert_snapshot!(out);
}

#[test]
fn sparse_rows_and_merged_cells() {
    // Sparse rows: empty cells render as empty pipes — but since GFM tables
    // require uniform column count, we pad shorter rows with empty cells
    // up to the widest row in the sheet.
    //
    // Merged cells: extract-text outputs the value once in the top-left
    // cell, leaves merged-into cells empty. We do the same (calamine
    // surfaces this for us).
    let out = extract_fixture("xlsx/04_sparse_merged.xlsx", Format::Xlsx);
    assert_snapshot!(out);
}

#[test]
fn formulas_use_cached_value() {
    // <c><f>...</f><v>cached</v></c> → output the cached <v>.
    // If <v> is missing, output empty (we don't evaluate formulas).
    // Error cells like #DIV/0! pass through.
    let out = extract_fixture("xlsx/05_formulas.xlsx", Format::Xlsx);
    assert_snapshot!(out);
}

#[test]
fn empty_workbook() {
    let out = extract_fixture("xlsx/06_empty.xlsx", Format::Xlsx);
    assert_snapshot!(out);
}

#[test]
fn special_characters_safe_for_markdown() {
    // Pipes are escaped to `\|`. Newlines/tabs in cells become spaces.
    // CJK and smart quotes pass through.
    let out = extract_fixture("xlsx/07_special_chars.xlsx", Format::Xlsx);
    assert_snapshot!(out);
}

#[test]
fn shared_strings_resolve_correctly() {
    // Verifies that t="s" cells correctly index into sharedStrings.xml.
    let out = extract_fixture("xlsx/08_shared_strings.xlsx", Format::Xlsx);
    assert_snapshot!(out);
}
