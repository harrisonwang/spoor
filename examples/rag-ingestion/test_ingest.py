from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

import ingest


def namespace(**values):
    return SimpleNamespace(**values)


class DocumentChunkTests(unittest.TestCase):
    def test_paragraphs_are_packed_and_overlap_is_deterministic(self) -> None:
        text = (
            "第一段包含项目背景和必要上下文。\n\n"
            "第二段包含本次评审的核心发现。\n\n"
            "第三段包含下一步行动项。"
        )
        chunks = ingest.document_chunks(text, max_chars=42, overlap_chars=10)

        self.assertGreater(len(chunks), 1)
        self.assertTrue(all(len(chunk) <= 42 for chunk in chunks))
        self.assertEqual(
            chunks,
            ingest.document_chunks(text, max_chars=42, overlap_chars=10),
        )

    def test_oversized_block_is_split(self) -> None:
        chunks = ingest.document_chunks("超长内容 " * 100, max_chars=128, overlap_chars=16)

        self.assertGreater(len(chunks), 2)
        self.assertTrue(all(len(chunk) <= 128 for chunk in chunks))


class ResultDispatchTests(unittest.TestCase):
    def test_document_result_becomes_stable_chunks(self) -> None:
        result = namespace(
            content=namespace(
                kind="document",
                value=namespace(markdown="# 项目报告\n\n发现一项需要处理的重要风险。"),
            ),
            stats=namespace(format="markdown"),
        )
        first = ingest.chunks_from_result(
            "report.md", result, max_chars=256, overlap_chars=20
        )
        second = ingest.chunks_from_result(
            "report.md", result, max_chars=256, overlap_chars=20
        )

        self.assertEqual(first, second)
        self.assertEqual(first[0]["kind"], "document")
        self.assertEqual(len(first[0]["id"]), 24)

    def test_table_result_emits_schema_and_row_chunks(self) -> None:
        table = {
            "source": "数据.csv",
            "sheet": None,
            "range": "A1:B2",
            "column_count": 2,
            "headers": {"区域": {"column_index": 0}, "数值": {"column_index": 1}},
            "rows": [{"row": 2, "区域": "北区", "数值": "42"}],
            "row_range": {"first": 2, "last": 2},
            "truncated": False,
            "warnings": [],
        }
        result = namespace(
            content=namespace(
                kind="tables",
                value=namespace(tables=(table,)),
            ),
            stats=namespace(format="csv"),
        )
        chunks = ingest.chunks_from_result(
            "数据.csv", result, max_chars=256, overlap_chars=20
        )

        self.assertEqual([chunk["kind"] for chunk in chunks], ["table_schema", "table_row"])
        self.assertEqual(chunks[1]["metadata"]["row"], 2)

    def test_truncated_table_is_visible_in_manifest_warnings(self) -> None:
        result = namespace(
            warnings=(),
            content=namespace(
                kind="tables",
                value=namespace(
                    tables=(
                        {
                            "source": "数据.xlsx",
                            "sheet": "明细",
                            "truncated": True,
                        },
                    )
                ),
            ),
        )

        self.assertEqual(
            ingest.result_warnings(result)[0]["code"],
            "table_preview_truncated",
        )

    def test_structured_warning_location_is_preserved(self) -> None:
        warning = {
            "code": "pdf_page_no_text_layer",
            "message": "第 2 页没有可提取文本层。",
            "location": {"kind": "page", "number": 2},
        }
        result = namespace(
            warnings=(warning,),
            content=namespace(kind="document"),
        )

        self.assertEqual(ingest.result_warnings(result), (warning,))


class DiscoveryAndOutputTests(unittest.TestCase):
    def test_discovery_filters_unknown_and_hidden_files(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "report.md").write_text("report", encoding="utf-8")
            (root / "ignored.bin").write_bytes(b"\x00")
            (root / ".secret.txt").write_text("secret", encoding="utf-8")

            discovered = ingest.discover_files([root])

        self.assertEqual([item.source for item in discovered], ["report.md"])

    def test_discovery_keeps_same_named_files_with_unique_sources(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            first = root / "first" / "report.md"
            second = root / "second" / "report.md"
            first.parent.mkdir()
            second.parent.mkdir()
            first.write_text("first", encoding="utf-8")
            second.write_text("second", encoding="utf-8")

            discovered = ingest.discover_files([first, second])

        self.assertEqual(
            [item.source for item in discovered],
            ["report.md", "second/report.md"],
        )

    def test_manifest_and_output_are_deterministic(self) -> None:
        record = ingest.FileRecord(
            source="report.md",
            status="parsed",
            format="markdown",
            input_bytes=8,
            output_bytes=8,
            chunks=1,
            warnings=(),
            error=None,
        )
        chunk = ingest.chunk_record(
            source="report.md",
            format_name="markdown",
            kind="document",
            index=0,
            text="A report",
            metadata={},
        )
        manifest = ingest.build_manifest([record], [chunk])

        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            ingest.write_output(output, [chunk], manifest)
            written_chunk = json.loads((output / "chunks.jsonl").read_text())
            written_manifest = json.loads((output / "manifest.json").read_text())

        self.assertEqual(written_chunk, chunk)
        self.assertEqual(written_manifest["summary"]["formats"], {"markdown": 1})


if __name__ == "__main__":
    unittest.main()
