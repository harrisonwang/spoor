//! PPTX integration tests.

mod common;
use common::{extract_fixture, parse_fixture};
use insta::assert_snapshot;
use spoor_core::{Format, WarningCode, WarningLocation};

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

#[test]
fn merged_table_and_visual_omissions_are_located_by_slide() {
    let merged = parse_fixture("pptx/06_merged_table.pptx", Format::Pptx);
    let visual = parse_fixture("pptx/07_embedded_visual.pptx", Format::Pptx);

    assert_eq!(
        merged.warnings[0].code,
        WarningCode::MergedTableStructureNotPreserved
    );
    assert_eq!(
        merged.warnings[0].location,
        Some(WarningLocation::Slide { number: 1 })
    );
    assert_eq!(visual.warnings[0].code, WarningCode::EmbeddedVisualsOmitted);
    assert_eq!(
        visual.warnings[0].location,
        Some(WarningLocation::Slide { number: 1 })
    );
}

#[test]
fn image_placeholders_follow_slide_order_and_only_reference_safe_entries() {
    let out = extract_fixture("pptx/08_image_placeholders.pptx", Format::Pptx);
    assert_snapshot!(out);

    // image_number runs across slides: 1 on slide 1, 2 + 3 on slide 2, none
    // on slide 3. python-pptx dedups by content hash, so slide 1 and slide 2's
    // first image share `ppt/media/image1.png` — verifies that the same OPC
    // part referenced from two slides still gets distinct image numbers.
    assert_eq!(
        out.matches("![PPTX image 1 (slide 1)](spoor://pptx/part/ppt/media/image1.png)")
            .count(),
        1
    );
    assert_eq!(
        out.matches("![PPTX image 2 (slide 2)](spoor://pptx/part/ppt/media/image1.png)")
            .count(),
        1
    );
    assert_eq!(
        out.matches("![PPTX image 3 (slide 2)](spoor://pptx/part/ppt/media/image2.png)")
            .count(),
        1
    );
    // Slide 3 has no images: no `PPTX image 4` placeholder is emitted.
    assert!(!out.contains("PPTX image 4"));

    // Every emitted handle uses the unified scheme; nothing escapes the
    // `spoor://pptx/part/ppt/media/` sandbox.
    let total = out.matches("spoor://pptx/part/ppt/media/").count();
    assert_eq!(total, 3);
    assert!(!out.contains("spoor-pptx://"));
}

#[test]
fn slide_with_images_carries_extract_wording_in_warning() {
    let parsed = parse_fixture("pptx/08_image_placeholders.pptx", Format::Pptx);
    // Slide 1 and 2 carry visuals; slide 3 does not.
    let visual_warnings: Vec<_> = parsed
        .warnings
        .iter()
        .filter(|w| w.code == WarningCode::EmbeddedVisualsOmitted)
        .collect();
    assert_eq!(visual_warnings.len(), 2);
    for warning in visual_warnings {
        assert!(
            warning.message.contains("spoor://pptx/part/") && warning.message.contains("--extract"),
            "expected extract wording, got: {}",
            warning.message,
        );
    }
}
