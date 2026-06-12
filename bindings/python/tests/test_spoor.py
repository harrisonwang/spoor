from pathlib import Path

from spoor import SpoorError, detect_format, parse_bytes, parse_path

ADVERSARIAL = (
    Path(__file__).resolve().parents[3]
    / "crates/spoor-cli/tests/fixtures/adversarial"
)


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
