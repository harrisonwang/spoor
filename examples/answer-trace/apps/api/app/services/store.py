"""数据出口。

- load_demo():phase 1 的内置对话(AnswerTrace[] + SourceDocument)。
- document_markdown() / source_ref():phase 2 的 matcher 用,真实 spoor 产物 +
  对应的 source 引用。
"""

import json
from functools import lru_cache
from pathlib import Path

_APP_DIR = Path(__file__).resolve().parent.parent  # apps/api/app


def _fixture_path() -> Path:
    for parent in Path(__file__).resolve().parents:
        candidate = parent / "packages" / "protocol" / "fixtures" / "demo.json"
        if candidate.exists():
            return candidate
    raise FileNotFoundError("找不到 packages/protocol/fixtures/demo.json")


@lru_cache(maxsize=1)
def load_demo() -> dict:
    return json.loads(_fixture_path().read_text(encoding="utf-8"))


@lru_cache(maxsize=1)
def document_markdown() -> str:
    """matcher 用于核验/定位的真实 spoor 产物(整篇 markdown)。"""
    return (_APP_DIR / "data" / "byd.md").read_text(encoding="utf-8")


@lru_cache(maxsize=1)
def source_ref() -> dict:
    """写进 AnswerTrace.source 的引用(取自内置 fixture 的 source 元信息)。"""
    src = load_demo()["source"]
    return {
        "documentId": src["documentId"],
        "title": src["title"],
        "pages": len(src["pages"]),
    }


@lru_cache(maxsize=1)
def builtin_raw_bytes() -> bytes | None:
    """内置演示对应的原始 PDF 字节,供 extract_media 提取图片。
    文件不存在时返回 None(图片不可用,但文本展示不受影响)。
    """
    pdf_path = _APP_DIR / "data" / "byd.pdf"
    if not pdf_path.exists():
        return None
    return pdf_path.read_bytes()
