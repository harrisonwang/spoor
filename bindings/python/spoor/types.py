from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Literal


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
    warnings: tuple[dict[str, str], ...]
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
