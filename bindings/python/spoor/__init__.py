from __future__ import annotations

from pathlib import Path
from typing import Any

from . import _native
from .exceptions import SpoorError
from .types import (
    ParseResult,
    Provenance,
    ProvenanceSpan,
    SourceAnchor,
    SpoorWarning,
    TextRange,
    WarningCode,
    WarningLocation,
    parse_result,
)

__all__ = [
    "ParseResult",
    "Provenance",
    "ProvenanceSpan",
    "SourceAnchor",
    "SpoorError",
    "SpoorWarning",
    "TextRange",
    "WarningCode",
    "WarningLocation",
    "detect_format",
    "extract_media",
    "parse_bytes",
    "parse_path",
]


def parse_bytes(
    data: bytes,
    *,
    source_name: str | None = None,
    content_type: str | None = None,
    format: str | None = None,
    max_parse_bytes: int | None = None,
    sheet: str | None = None,
    rows: tuple[int, int] | None = None,
    columns: list[str] | None = None,
    limit: int | None = None,
    offset: int | None = None,
    pages: tuple[int, int] | None = None,
    max_work_units: int | None = None,
    provenance: str | None = None,
) -> ParseResult:
    """Parse document/table bytes into a typed result.

    For table formats (CSV/XLSX) the narrowing options mirror the CLI: ``sheet``
    (XLSX only), ``rows`` as an inclusive 1-based ``(first, last)`` pair (mutually
    exclusive with ``limit``/``offset``), ``columns`` to keep, and
    ``limit``/``offset`` for pagination. For page-oriented formats (PDF), ``pages``
    is an inclusive 1-based ``(first, last)`` range that limits which pages are
    parsed. Each option is ignored by formats it does not apply to.

    ``provenance`` (``"page"``) returns an output→source mapping in
    ``result.provenance``; output byte ranges index ``markdown`` as UTF-8, so
    slice with ``markdown.encode("utf-8")[start:end]``.
    """
    try:
        raw: dict[str, Any] = _native.parse_bytes(
            data,
            source_name,
            content_type,
            format,
            max_parse_bytes,
            sheet,
            rows,
            columns,
            limit,
            offset,
            pages,
            max_work_units,
            provenance,
        )
    except _native.SpoorError as error:
        raise SpoorError.from_native(error) from None
    return parse_result(raw)


def parse_path(
    path: str | Path,
    *,
    format: str | None = None,
    max_parse_bytes: int | None = None,
    sheet: str | None = None,
    rows: tuple[int, int] | None = None,
    columns: list[str] | None = None,
    limit: int | None = None,
    offset: int | None = None,
    pages: tuple[int, int] | None = None,
    max_work_units: int | None = None,
    provenance: str | None = None,
) -> ParseResult:
    path = Path(path)
    return parse_bytes(
        path.read_bytes(),
        source_name=str(path),
        format=format,
        max_parse_bytes=max_parse_bytes,
        sheet=sheet,
        rows=rows,
        columns=columns,
        limit=limit,
        offset=offset,
        pages=pages,
        max_work_units=max_work_units,
        provenance=provenance,
    )


def extract_media(
    data: bytes,
    resource: str,
    *,
    source_name: str | None = None,
    content_type: str | None = None,
    format: str | None = None,
    max_parse_bytes: int | None = None,
) -> bytes:
    """Extract one safe embedded media resource referenced by a URI spoor emitted.

    ``resource`` is a safe URI from the parsed output, e.g.
    ``spoor://docx/part/word/media/image1.png``,
    ``spoor://pptx/part/ppt/media/imageN.png``, or
    ``spoor://pdf/obj/{id}/{gen}``.
    Returns the raw bytes; spoor does not decode or interpret them.
    """
    try:
        return _native.extract_media(
            data, resource, source_name, content_type, format, max_parse_bytes
        )
    except _native.SpoorError as error:
        raise SpoorError.from_native(error) from None


def detect_format(
    data: bytes,
    *,
    source_name: str | None = None,
    content_type: str | None = None,
) -> str:
    try:
        return _native.detect_format(data, source_name, content_type)
    except _native.SpoorError as error:
        raise SpoorError.from_native(error) from None
