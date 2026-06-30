"""phase 2:真问真答。POST /api/ask {question} → AnswerTrace(经 Workers AI 产出)。"""

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from app import config
from app.services import corpus, matcher

router = APIRouter()


class AskBody(BaseModel):
    question: str


@router.post("/ask")
def ask(body: AskBody) -> dict:
    question = body.question.strip()
    if not question:
        raise HTTPException(status_code=400, detail="问题不能为空")
    if not config.llm_enabled():
        raise HTTPException(
            status_code=503,
            detail="未配置模型后端:在 apps/api/.env 设 AT_BASE_URL + AT_API_KEY(OpenRouter/DeepSeek/z.ai 等),或 CF_ACCOUNT_ID + CF_API_TOKEN。",
        )
    try:
        return matcher.build_trace(question, corpus.markdown(), corpus.source_ref())
    except Exception as exc:  # 模型调用 / JSON 解析失败 → 502,带原因
        raise HTTPException(status_code=502, detail=f"模型调用或解析失败:{exc}") from exc
