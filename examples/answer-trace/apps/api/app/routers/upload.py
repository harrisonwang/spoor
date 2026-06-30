"""多文件上传 → pyspoor 解析 → 设为当前语料。"""

from fastapi import APIRouter, File, UploadFile

from app.services import corpus, media_urls, parsing

router = APIRouter()


@router.post("/upload")
async def upload(files: list[UploadFile] = File(...)) -> dict:
    parsed: list[dict] = []
    results: list[dict] = []
    for f in files:
        name = f.filename or "file"
        data = await f.read()
        try:
            md, raw_bytes = parsing.parse_to_markdown(name, data)
            parsed.append({"name": name, "markdown": md, "raw_bytes": raw_bytes})
            results.append({"name": name, "chars": len(md), "ok": True})
        except Exception as exc:  # 单个文件失败不连累其它
            results.append({"name": name, "ok": False, "error": str(exc)})

    if parsed:
        corpus.set_docs(parsed)

    raw_md = corpus.markdown()
    md = media_urls.rewrite_spoor_images(raw_md, doc_index=0)
    return {
        "files": results,
        "source": corpus.source_ref(),
        "markdown": md,
    }
