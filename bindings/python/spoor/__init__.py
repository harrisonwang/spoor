from __future__ import annotations

from pathlib import Path
from typing import Any

from . import _native
from .exceptions import SpoorError
from .types import ParseResult, SpoorWarning, WarningCode, WarningLocation, parse_result

__all__ = [
    "ParseResult",
    "SpoorError",
    "SpoorWarning",
    "WarningCode",
    "WarningLocation",
    "detect_format",
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
) -> ParseResult:
    try:
        raw: dict[str, Any] = _native.parse_bytes(
            data, source_name, content_type, format, max_parse_bytes
        )
    except _native.SpoorError as error:
        raise SpoorError.from_native(error) from None
    return parse_result(raw)


def parse_path(
    path: str | Path,
    *,
    format: str | None = None,
    max_parse_bytes: int | None = None,
) -> ParseResult:
    path = Path(path)
    return parse_bytes(
        path.read_bytes(),
        source_name=str(path),
        format=format,
        max_parse_bytes=max_parse_bytes,
    )


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
