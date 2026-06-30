from fastapi import APIRouter

from app import config
from app.services import media_urls, store, tokens

router = APIRouter()


@router.get("/health")
def health() -> dict:
    return {"status": "ok"}


@router.get("/demo")
def get_demo() -> dict:
    """内置三轮对话 + 其依据的原文 markdown(供前端就地渲染与下钻)。"""
    d = store.load_demo()
    meta = d["source"]
    raw = store.document_markdown()
    md = media_urls.rewrite_spoor_images(raw, doc_index=0)
    return {
        "source": {
            "documentId": meta["documentId"],
            "title": meta["title"],
            "pages": len(meta["pages"]),
            "markdown": md,
            "tokens": tokens.count(raw),
            "contextLimit": config.CONTEXT_LIMIT,
        },
        "traces": d["traces"],
    }
