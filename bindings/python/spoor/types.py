from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Literal, TypedDict

WarningCode = Literal[
    "pdf_page_no_text_layer",
    "pdf_page_suspicious_text_layer",
    "pdf_multi_column_reading_order",
    "merged_table_structure_not_preserved",
    "embedded_visuals_omitted",
]


class WarningLocation(TypedDict):
    kind: Literal["page", "slide"]
    number: int


class _SpoorWarningRequired(TypedDict):
    code: WarningCode
    message: str


class SpoorWarning(_SpoorWarningRequired, total=False):
    location: WarningLocation


@dataclass(frozen=True, slots=True)
class DocumentResult:
    source: str
    format: str
    markdown: str


@dataclass(frozen=True, slots=True)
class TableResult:
    tables: tuple[dict[str, Any], ...]
    serialized_bytes: int


@dataclass(frozen=True, slots=True)
class ParseContent:
    kind: Literal["document", "tables"]
    value: DocumentResult | TableResult


@dataclass(frozen=True, slots=True)
class ParseStats:
    input_bytes: int
    output_bytes: int
    format: str
    page_count: int | None = None


class TextRange(TypedDict):
    """Half-open ``[start, end)`` byte range into the returned ``markdown``."""

    start: int
    end: int


class SourceAnchor(TypedDict):
    """Where a span of output came from. Currently page-oriented (PDF)."""

    kind: Literal["page"]
    number: int


class ProvenanceSpan(TypedDict):
    output: TextRange
    source: SourceAnchor


@dataclass(frozen=True, slots=True)
class Provenance:
    spans: tuple[ProvenanceSpan, ...]


@dataclass(frozen=True, slots=True)
class ParseResult:
    content: ParseContent
    warnings: tuple[SpoorWarning, ...]
    stats: ParseStats
    provenance: Provenance | None = None


def parse_result(raw: dict[str, Any]) -> ParseResult:
    content = raw["content"]
    if content["kind"] == "document":
        value: DocumentResult | TableResult = DocumentResult(**content["value"])
    else:
        table_value = content["value"]
        value = TableResult(
            tables=tuple(table_value["tables"]),
            serialized_bytes=table_value["serialized_bytes"],
        )
    provenance_raw = raw.get("provenance")
    provenance = (
        Provenance(spans=tuple(provenance_raw["spans"]))
        if provenance_raw is not None
        else None
    )
    return ParseResult(
        content=ParseContent(kind=content["kind"], value=value),
        warnings=tuple(raw["warnings"]),
        stats=ParseStats(**raw["stats"]),
        provenance=provenance,
    )
