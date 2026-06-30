"""pyspoor 解析:上传文件 → markdown(供 matcher 核验/定位)。

文档类(PDF/DOCX/PPTX…)直接拿 markdown;表格类(CSV/XLSX)把结构化表渲染成
markdown 表格,这样 matcher 的定位器一视同仁地在文本里找证据。
"""

import spoor


def _tables_to_markdown(tables) -> str:
    out: list[str] = []
    for t in tables:
        name = t.get("sheet") or t.get("title") or t.get("source") or "表"
        headers_meta = t.get("headers", {})
        headers = sorted(headers_meta, key=lambda h: headers_meta[h]["column_index"])
        out.append(f"## {name}\n")
        if headers:
            out.append("| " + " | ".join(headers) + " |")
            out.append("| " + " | ".join("---" for _ in headers) + " |")
            for row in t.get("rows", []):
                out.append(
                    "| " + " | ".join(str(row.get(h, "")) for h in headers) + " |"
                )
        out.append("")
    return "\n".join(out)


def parse_to_markdown(name: str, data: bytes) -> tuple[str, bytes]:
    """解析文档,返回 (markdown, raw_bytes)。

    raw_bytes 用于后续 extract_media 提取内嵌图片。
    """
    res = spoor.parse_bytes(data, source_name=name)
    if res.content.kind == "document":
        return res.content.value.markdown, data
    return _tables_to_markdown(res.content.value.tables), data
