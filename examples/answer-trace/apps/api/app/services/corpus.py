"""当前语料(内存态)。上传后即为对话的依据;未上传则回退到内置 byd.md。

演示用全局单实例(非多用户)。matcher 与下钻都以此为准。
"""

from app.services import store

_docs: list[dict] = []  # [{name, markdown, raw_bytes}]


def set_docs(docs: list[dict]) -> None:
    global _docs
    _docs = docs


def has_docs() -> bool:
    return bool(_docs)


def names() -> list[str]:
    return [d["name"] for d in _docs]


def get_doc(index: int) -> dict | None:
    """按索引取文档,含原始字节(供 extract_media 使用)。

    - 有上传文档时从 _docs 取
    - 无上传时 index=0 回退到内置演示
    """
    if _docs:
        if 0 <= index < len(_docs):
            return _docs[index]
        return None
    if index == 0:
        raw = store.builtin_raw_bytes()
        if raw is None:
            return None
        return {
            "name": "byd_report.pdf",
            "markdown": store.document_markdown(),
            "raw_bytes": raw,
        }
    return None


def markdown() -> str:
    if not _docs:
        return store.document_markdown()
    return "\n\n".join(f"# 文件:{d['name']}\n\n{d['markdown']}" for d in _docs)


def source_ref() -> dict:
    if not _docs:
        return store.source_ref()
    first = _docs[0]["name"]
    title = first if len(_docs) == 1 else f"{first} 等 {len(_docs)} 个文件"
    return {
        "documentId": "uploaded",
        "title": title,
        "pages": markdown().count("## Page "),
    }
