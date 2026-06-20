from pathlib import Path

import pytest

from spoor import (
    SpoorError,
    detect_format,
    extract_media,
    parse_bytes,
    parse_path,
)

FIXTURES = Path(__file__).resolve().parents[3] / "crates/spoor-cli/tests/fixtures"
ADVERSARIAL = FIXTURES / "adversarial"
CSV_BASIC = FIXTURES / "csv/01_basic.csv"
DOCX_IMAGES = FIXTURES / "docx/16_image_placeholders.docx"


def test_parse_bytes_returns_typed_result() -> None:
    result = parse_bytes(b"hello python\n", source_name="note.txt")
    assert result.content.kind == "document"
    assert result.content.value.markdown == "hello python\n"
    assert result.stats.format == "text"


def test_detect_format() -> None:
    assert detect_format(b"a,b\n1,2\n", source_name="data.csv") == "csv"


def test_parse_path(tmp_path: Path) -> None:
    path = tmp_path / "note.txt"
    path.write_text("hello path\n")
    assert parse_path(path).content.value.markdown == "hello path\n"


def test_warning_code_and_location_match_core_contract() -> None:
    result = parse_path(FIXTURES / "pdf/05_mixed_text_and_image.pdf")

    assert result.warnings[0]["code"] == "pdf_page_no_text_layer"
    assert result.warnings[0]["location"] == {"kind": "page", "number": 2}


def test_table_filter_paginates_and_selects_columns() -> None:
    # 01_basic.csv: Alice(row 2), Bob(row 3), Carol(row 4); cols Name/Score/Note.
    result = parse_bytes(
        CSV_BASIC.read_bytes(),
        source_name="data.csv",
        columns=["Name"],
        limit=1,
        offset=1,
    )
    assert result.content.kind == "tables"
    rows = result.content.value.tables[0]["rows"]
    assert rows == [{"Name": "Bob"}]


def test_table_filter_row_range_matches_cli_semantics() -> None:
    result = parse_bytes(CSV_BASIC.read_bytes(), source_name="data.csv", rows=(3, 3))
    rows = result.content.value.tables[0]["rows"]
    assert [row["Name"] for row in rows] == ["Bob"]


def test_table_filter_rejects_rows_with_limit() -> None:
    with pytest.raises(SpoorError):
        parse_bytes(CSV_BASIC.read_bytes(), source_name="data.csv", rows=(2, 4), limit=1)


def test_extract_media_returns_safe_docx_resource() -> None:
    image = extract_media(
        DOCX_IMAGES.read_bytes(),
        "spoor://docx/part/word/media/image1.png",
        source_name="images.docx",
    )
    assert image == b"first-image"


def test_extract_media_rejects_unsafe_uri() -> None:
    with pytest.raises(SpoorError):
        extract_media(
            DOCX_IMAGES.read_bytes(),
            "word/media/image1.png",
            source_name="images.docx",
        )


def test_pages_filter_limits_pdf_to_requested_pages() -> None:
    # 02_multipage.pdf has 3 pages; --pages 2:2 keeps only page 2.
    result = parse_path(FIXTURES / "pdf/02_multipage.pdf", pages=(2, 2))
    markdown = result.content.value.markdown
    assert "## Page 2" in markdown
    assert "## Page 1" not in markdown
    assert "## Page 3" not in markdown


def test_pages_filter_rejects_invalid_range() -> None:
    with pytest.raises(SpoorError):
        parse_path(FIXTURES / "pdf/02_multipage.pdf", pages=(3, 1))


def test_work_budget_aborts_with_stable_error() -> None:
    with pytest.raises(SpoorError) as exc:
        parse_path(FIXTURES / "pdf/02_multipage.pdf", max_work_units=1)
    assert exc.value.code == "work_budget_exceeded"


def test_page_provenance_maps_output_to_source_pages() -> None:
    # 02_multipage.pdf has 3 pages; page provenance returns one span per page,
    # each output byte range slicing that page's block out of the Markdown.
    result = parse_path(FIXTURES / "pdf/02_multipage.pdf", provenance="page")
    assert result.provenance is not None
    spans = result.provenance.spans
    assert len(spans) == 3
    assert spans[0]["source"] == {"kind": "page", "number": 1}

    markdown = result.content.value.markdown
    start = spans[0]["output"]["start"]
    end = spans[0]["output"]["end"]
    assert markdown.encode("utf-8")[start:end].startswith(b"## Page 1")


def test_provenance_off_by_default() -> None:
    assert parse_path(FIXTURES / "pdf/02_multipage.pdf").provenance is None


def test_error_fields_are_stable() -> None:
    try:
        parse_bytes(b"\x00\x01", source_name="unknown.bin")
    except SpoorError as error:
        assert error.code == "unsupported_format"
        assert error.stage == "detect"
    else:
        raise AssertionError("expected SpoorError")


def test_parse_budget_error_matches_core_contract() -> None:
    try:
        parse_bytes(b"x" * 2048, max_parse_bytes=1024)
    except SpoorError as error:
        assert error.code == "parse_budget_exceeded"
        assert error.stage == "limits"
        assert error.recoverable is True
    else:
        raise AssertionError("expected SpoorError")


def test_invalid_container_error_matches_core_contract() -> None:
    try:
        parse_bytes(b"not a zip", source_name="bad.docx", format="docx")
    except SpoorError as error:
        assert error.code == "invalid_container"
        assert error.stage == "parse"
    else:
        raise AssertionError("expected SpoorError")


def test_compression_bomb_is_rejected_by_shared_budget() -> None:
    try:
        parse_bytes(
            (ADVERSARIAL / "05_compression_bomb.docx").read_bytes(),
            source_name="bomb.docx",
            format="docx",
            max_parse_bytes=1024 * 1024,
        )
    except SpoorError as error:
        assert error.code == "parse_budget_exceeded"
        assert error.stage == "limits"
    else:
        raise AssertionError("expected SpoorError")


def test_cfb_office_container_is_intercepted() -> None:
    try:
        parse_bytes(bytes.fromhex("d0cf11e0a1b11ae1"), source_name="encrypted.docx")
    except SpoorError as error:
        assert error.code == "legacy_or_encrypted_office"
        assert error.stage == "detect"
        assert error.recoverable is False
    else:
        raise AssertionError("expected SpoorError")
