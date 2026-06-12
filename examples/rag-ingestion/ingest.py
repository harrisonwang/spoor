#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Iterable, Iterator

SUPPORTED_EXTENSIONS = {
    ".csv",
    ".c",
    ".cpp",
    ".docx",
    ".epub",
    ".go",
    ".h",
    ".htm",
    ".html",
    ".ipynb",
    ".java",
    ".js",
    ".json",
    ".log",
    ".md",
    ".markdown",
    ".pdf",
    ".pptx",
    ".py",
    ".rs",
    ".sh",
    ".sql",
    ".tsv",
    ".toml",
    ".ts",
    ".txt",
    ".xml",
    ".xlsm",
    ".xlsx",
    ".yaml",
    ".yml",
}


@dataclass(frozen=True, slots=True)
class SourceFile:
    path: Path
    source: str


@dataclass(frozen=True, slots=True)
class FileRecord:
    source: str
    status: str
    format: str | None
    input_bytes: int
    output_bytes: int | None
    chunks: int
    warnings: tuple[dict[str, str], ...]
    error: dict[str, Any] | None


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build deterministic JSONL chunks from mixed documents with pyspoor."
    )
    parser.add_argument("inputs", nargs="+", type=Path, help="files or directories to ingest")
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("spoor-index"),
        help="directory for chunks.jsonl and manifest.json",
    )
    parser.add_argument(
        "--chunk-chars",
        type=int,
        default=1200,
        help="maximum characters per document chunk (default: 1200)",
    )
    parser.add_argument(
        "--overlap-chars",
        type=int,
        default=120,
        help="tail overlap carried into the next document chunk (default: 120)",
    )
    parser.add_argument(
        "--max-parse-bytes",
        type=int,
        default=64 * 1024 * 1024,
        help="per-file spoor parsing budget (default: 67108864)",
    )
    parser.add_argument(
        "--all-files",
        action="store_true",
        help="inspect every discovered file instead of filtering known extensions",
    )
    args = parser.parse_args(argv)
    if args.chunk_chars < 128:
        parser.error("--chunk-chars must be at least 128")
    if args.overlap_chars < 0 or args.overlap_chars >= args.chunk_chars:
        parser.error("--overlap-chars must be non-negative and smaller than --chunk-chars")
    if args.max_parse_bytes < 1024:
        parser.error("--max-parse-bytes must be at least 1024")
    return args


def discover_files(inputs: Iterable[Path], all_files: bool = False) -> list[SourceFile]:
    discovered: dict[Path, SourceFile] = {}
    used_sources: set[str] = set()
    for raw_input in inputs:
        path = raw_input.expanduser()
        if path.is_file():
            resolved = path.resolve()
            source = unique_source(path.name, path.parent.name, used_sources)
            discovered.setdefault(resolved, SourceFile(path=path, source=source))
            continue
        if not path.is_dir():
            raise FileNotFoundError(f"input does not exist: {raw_input}")

        for candidate in sorted(path.rglob("*")):
            if not candidate.is_file() or any(part.startswith(".") for part in candidate.parts):
                continue
            if not all_files and candidate.suffix.lower() not in SUPPORTED_EXTENSIONS:
                continue
            source = candidate.relative_to(path).as_posix()
            resolved = candidate.resolve()
            if resolved in discovered:
                continue
            source = unique_source(source, path.name, used_sources)
            discovered[resolved] = SourceFile(path=candidate, source=source)
    return sorted(discovered.values(), key=lambda item: item.source)


def unique_source(source: str, prefix: str, used_sources: set[str]) -> str:
    if source not in used_sources:
        used_sources.add(source)
        return source
    candidate = f"{prefix}/{source}"
    counter = 2
    while candidate in used_sources:
        candidate = f"{prefix}-{counter}/{source}"
        counter += 1
    used_sources.add(candidate)
    return candidate


def document_chunks(
    markdown: str,
    *,
    max_chars: int,
    overlap_chars: int,
) -> list[str]:
    blocks = [block.strip() for block in markdown.split("\n\n") if block.strip()]
    chunks: list[str] = []
    current = ""

    for block in blocks:
        if len(block) > max_chars:
            if current:
                chunks.append(current)
                current = overlap_tail(current, overlap_chars)
            for window in sliding_windows(block, max_chars, overlap_chars):
                if current:
                    candidate = f"{current}\n\n{window}"
                    if len(candidate) <= max_chars:
                        chunks.append(candidate)
                        current = overlap_tail(candidate, overlap_chars)
                        continue
                chunks.append(window)
                current = overlap_tail(window, overlap_chars)
            current = ""
            continue

        candidate = block if not current else f"{current}\n\n{block}"
        if len(candidate) <= max_chars:
            current = candidate
            continue

        chunks.append(current)
        tail = overlap_tail(current, overlap_chars)
        candidate = block if not tail else f"{tail}\n\n{block}"
        current = candidate if len(candidate) <= max_chars else block

    if current and (not chunks or current != chunks[-1]):
        chunks.append(current)
    return chunks


def sliding_windows(text: str, max_chars: int, overlap_chars: int) -> Iterator[str]:
    start = 0
    while start < len(text):
        end = min(start + max_chars, len(text))
        if end < len(text):
            boundary = text.rfind(" ", start + max_chars // 2, end)
            if boundary > start:
                end = boundary
        yield text[start:end].strip()
        if end >= len(text):
            break
        start = max(start + 1, end - overlap_chars)


def overlap_tail(text: str, overlap_chars: int) -> str:
    if overlap_chars == 0:
        return ""
    tail = text[-overlap_chars:]
    first_space = tail.find(" ")
    return tail[first_space + 1 :].strip() if first_space >= 0 else tail.strip()


def chunks_from_result(
    source: str,
    result: Any,
    *,
    max_chars: int,
    overlap_chars: int,
) -> list[dict[str, Any]]:
    if result.content.kind == "document":
        chunks = document_chunks(
            result.content.value.markdown,
            max_chars=max_chars,
            overlap_chars=overlap_chars,
        )
        return [
            chunk_record(
                source=source,
                format_name=result.stats.format,
                kind="document",
                index=index,
                text=text,
                metadata={},
            )
            for index, text in enumerate(chunks)
        ]

    records: list[dict[str, Any]] = []
    for table_index, table in enumerate(result.content.value.tables):
        table_meta = {
            "table_index": table_index,
            "sheet": table.get("sheet"),
            "range": table.get("range"),
            "column_count": table["column_count"],
            "headers": table["headers"],
            "truncated": table["truncated"],
            "warnings": table["warnings"],
        }
        records.append(
            chunk_record(
                source=source,
                format_name=result.stats.format,
                kind="table_schema",
                index=table_index,
                text=json.dumps(table_meta, ensure_ascii=False, sort_keys=True),
                metadata=table_meta,
            )
        )
        for row_index, row in enumerate(table["rows"]):
            first_row = table.get("row_range", {}).get("first")
            row_number = row.get("row") or (
                first_row + row_index if first_row is not None else None
            )
            records.append(
                chunk_record(
                    source=source,
                    format_name=result.stats.format,
                    kind="table_row",
                    index=row_index,
                    text=json.dumps(row, ensure_ascii=False, sort_keys=True),
                    metadata={
                        "table_index": table_index,
                        "sheet": table.get("sheet"),
                        "row": row_number,
                    },
                )
            )
    return records


def result_warnings(result: Any) -> tuple[dict[str, str], ...]:
    warnings = list(result.warnings)
    if result.content.kind == "tables":
        for table in result.content.value.tables:
            if table["truncated"]:
                warnings.append(
                    {
                        "code": "table_preview_truncated",
                        "message": (
                            f"{table.get('sheet') or table.get('source') or 'table'} "
                            "只摄取了 spoor 默认表格预览；请用 CLI 分页读取完整数据。"
                        ),
                    }
                )
    return tuple(warnings)


def chunk_record(
    *,
    source: str,
    format_name: str,
    kind: str,
    index: int,
    text: str,
    metadata: dict[str, Any],
) -> dict[str, Any]:
    identity = json.dumps(
        [source, format_name, kind, index, metadata, text],
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode()
    return {
        "id": hashlib.sha256(identity).hexdigest()[:24],
        "source": source,
        "format": format_name,
        "kind": kind,
        "index": index,
        "text": text,
        "metadata": metadata,
    }


def ingest(
    source_files: Iterable[SourceFile],
    *,
    max_chars: int,
    overlap_chars: int,
    max_parse_bytes: int,
) -> tuple[list[dict[str, Any]], list[FileRecord]]:
    from spoor import SpoorError, parse_bytes

    chunks: list[dict[str, Any]] = []
    files: list[FileRecord] = []
    for source_file in source_files:
        try:
            data = source_file.path.read_bytes()
            result = parse_bytes(
                data,
                source_name=source_file.source,
                max_parse_bytes=max_parse_bytes,
            )
            file_chunks = chunks_from_result(
                source_file.source,
                result,
                max_chars=max_chars,
                overlap_chars=overlap_chars,
            )
            chunks.extend(file_chunks)
            files.append(
                FileRecord(
                    source=source_file.source,
                    status="parsed",
                    format=result.stats.format,
                    input_bytes=result.stats.input_bytes,
                    output_bytes=result.stats.output_bytes,
                    chunks=len(file_chunks),
                    warnings=result_warnings(result),
                    error=None,
                )
            )
            print(
                f"parsed {source_file.source}: {result.stats.format}, "
                f"{len(file_chunks)} chunks",
                file=sys.stderr,
            )
        except SpoorError as error:
            files.append(
                FileRecord(
                    source=source_file.source,
                    status="error",
                    format=None,
                    input_bytes=source_file.path.stat().st_size,
                    output_bytes=None,
                    chunks=0,
                    warnings=(),
                    error={
                        "code": error.code,
                        "reason": error.reason,
                        "hint": error.hint,
                        "recoverable": error.recoverable,
                        "stage": error.stage,
                    },
                )
            )
            print(f"skipped {source_file.source}: {error}", file=sys.stderr)
        except OSError as error:
            files.append(
                FileRecord(
                    source=source_file.source,
                    status="error",
                    format=None,
                    input_bytes=0,
                    output_bytes=None,
                    chunks=0,
                    warnings=(),
                    error={
                        "code": "read_failed",
                        "reason": str(error),
                        "hint": "Check the path and file permissions.",
                        "recoverable": True,
                        "stage": "read",
                    },
                )
            )
            print(f"skipped {source_file.source}: {error}", file=sys.stderr)
    return chunks, files


def build_manifest(files: list[FileRecord], chunks: list[dict[str, Any]]) -> dict[str, Any]:
    parsed = [record for record in files if record.status == "parsed"]
    failed = [record for record in files if record.status == "error"]
    formats = Counter(record.format for record in parsed if record.format is not None)
    kinds = Counter(chunk["kind"] for chunk in chunks)
    return {
        "schema_version": "spoor-ingestion-manifest-v1",
        "summary": {
            "files": len(files),
            "parsed": len(parsed),
            "failed": len(failed),
            "chunks": len(chunks),
            "formats": dict(sorted(formats.items())),
            "chunk_kinds": dict(sorted(kinds.items())),
        },
        "files": [asdict(record) for record in files],
    }


def write_output(
    output_dir: Path,
    chunks: list[dict[str, Any]],
    manifest: dict[str, Any],
) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    chunks_path = output_dir / "chunks.jsonl"
    manifest_path = output_dir / "manifest.json"
    chunks_path.write_text(
        "".join(
            f"{json.dumps(chunk, ensure_ascii=False, sort_keys=True)}\n"
            for chunk in chunks
        ),
        encoding="utf-8",
    )
    manifest_path.write_text(
        f"{json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True)}\n",
        encoding="utf-8",
    )


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        source_files = discover_files(args.inputs, all_files=args.all_files)
    except FileNotFoundError as error:
        print(error, file=sys.stderr)
        return 2
    if not source_files:
        print("no input files discovered", file=sys.stderr)
        return 2

    chunks, files = ingest(
        source_files,
        max_chars=args.chunk_chars,
        overlap_chars=args.overlap_chars,
        max_parse_bytes=args.max_parse_bytes,
    )
    manifest = build_manifest(files, chunks)
    write_output(args.output_dir, chunks, manifest)
    print(
        f"wrote {len(chunks)} chunks from {manifest['summary']['parsed']} files "
        f"to {args.output_dir}",
        file=sys.stderr,
    )
    return 0 if manifest["summary"]["parsed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
