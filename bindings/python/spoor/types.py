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


@dataclass(frozen=True, slots=True)
class ParseResult:
    content: ParseContent
    warnings: tuple[SpoorWarning, ...]
    stats: ParseStats


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
    return ParseResult(
        content=ParseContent(kind=content["kind"], value=value),
        warnings=tuple(raw["warnings"]),
        stats=ParseStats(**raw["stats"]),
    )
