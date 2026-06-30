from fastapi import APIRouter

from app.services import media_urls, store

router = APIRouter()


@router.get("/health")
def health() -> dict:
    return {"status": "ok"}


@router.get("/demo")
def get_demo() -> dict:
    """内置三轮对话 + 其依据的原文 markdown(供前端就地渲染与下钻)。"""
    d = store.load_demo()
    meta = d["source"]
    md = media_urls.rewrite_spoor_images(store.document_markdown(), doc_index=0)
    return {
        "source": {
            "documentId": meta["documentId"],
            "title": meta["title"],
            "pages": len(meta["pages"]),
            "markdown": md,
        },
        "traces": d["traces"],
    }
