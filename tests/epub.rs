//! EPUB integration tests.

mod common;
use common::extract_fixture;
use gist::format::Format;
use insta::assert_snapshot;

#[test]
fn basic_book_chapters_in_spine_order() {
    // The OPF <spine> defines reading order. Files in the manifest
    // not in the spine (cover images, css, etc.) must be ignored.
    //
    // IMPORTANT: extract-text iterates xhtml files alphabetically rather
    // than by spine order. For most books this happens to be correct
    // (ch01.xhtml < ch02.xhtml) but for any book using non-numeric
    // filenames, output is in the wrong order.
    //
    // We CHOOSE to parse the OPF and follow spine order properly.
    let out = extract_fixture("epub/01_basic.epub", Format::Epub);
    assert_snapshot!(out);
}
