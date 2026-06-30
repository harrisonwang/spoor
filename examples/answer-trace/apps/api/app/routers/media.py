"""图片提取端点:将 spoor:// 安全 URI 还原为原始图片字节。

前端渲染 markdown 时遇到 ![](/api/media?uri=...) 即调此接口。
"""

from fastapi import APIRouter, HTTPException, Query
from fastapi.responses import Response
from spoor import extract_media

from app.services import corpus

router = APIRouter()


def _content_type(raw: bytes) -> str:
    """按魔数判定图片类型,无需 URI 后缀。"""
    if raw[:4] == b"\x89PNG":
        return "image/png"
    if raw[:2] == b"\xff\xd8":
        return "image/jpeg"
    if raw[:3] == b"GIF":
        return "image/gif"
    if raw[:4] == b"RIFF" and raw[8:12] == b"WEBP":
        return "image/webp"
    head = raw[:64].decode("utf-8", errors="replace").strip().lower()
    if head.startswith("<?xml") or head.startswith("<svg"):
        return "image/svg+xml"
    return "application/octet-stream"


@router.get("/media")
def get_media(uri: str = Query(...), doc: int = Query(0)):
    """提取 spoor:// 安全 URI 对应的内嵌图片。

    - uri: spoor 在 markdown 中输出的安全资源 URI
    - doc: 文档索引(0 = 内置演示 / 第一个上传文件)
    """
    doc_info = corpus.get_doc(doc)
    if doc_info is None:
        raise HTTPException(404, "document not found")

    try:
        raw = extract_media(
            doc_info["raw_bytes"],
            uri,
            source_name=doc_info["name"],
        )
    except Exception as exc:
        raise HTTPException(422, f"extract media failed: {exc}")

    return Response(content=raw, media_type=_content_type(raw))
