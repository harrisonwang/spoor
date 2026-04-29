//! PPTX integration tests.

mod common;
use common::extract_fixture;
use gist::format::Format;
use insta::assert_snapshot;

#[test]
fn basic_slides_with_titles_and_bullets() {
    // Each slide → "## Slide N" header.
    // Title and body text both extracted from <a:t> nodes.
    // <a:p> within a text frame separates text into lines.
    let out = extract_fixture("pptx/01_basic.pptx", Format::Pptx);
    assert_snapshot!(out);
}

#[test]
fn tables_in_slides() {
    // IMPORTANT: extract-text flattens tables in pptx — it just emits each
    // cell on its own line, no GFM table structure. This is a regression
    // from its docx behavior.
    //
    // We CHOOSE to emit GFM tables for pptx as well, matching docx.
    // It's a few extra lines of code and substantially more useful output.
    let out = extract_fixture("pptx/02_with_table.pptx", Format::Pptx);
    assert_snapshot!(out);
}

#[test]
fn speaker_notes_are_included() {
    // IMPORTANT: extract-text deliberately ignores ppt/notesSlides/*.xml.
    // Speaker notes often contain critical context (talking points, rationale,
    // citations) that are *more* valuable to an LLM than the slide bullets.
    //
    // We CHOOSE to include them, rendered under a "Notes:" sub-section.
    let out = extract_fixture("pptx/03_with_notes.pptx", Format::Pptx);
    assert_snapshot!(out);
}

#[test]
fn empty_deck_with_blank_slide() {
    let out = extract_fixture("pptx/04_empty.pptx", Format::Pptx);
    assert_snapshot!(out);
}

#[test]
fn slide_ordering_handles_double_digits() {
    // slide11.xml must come after slide2.xml. extract-text gets this
    // right by parsing the trailing digits and sorting numerically.
    // Test verifies this for slides 1..12.
    let out = extract_fixture("pptx/05_ordering.pptx", Format::Pptx);
    assert_snapshot!(out);
}
