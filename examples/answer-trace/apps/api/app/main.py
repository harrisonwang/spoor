"""answer-trace api.

- /api/demo:phase 1 的内置对话(转发 fixture)。
- /api/ask :phase 2,经 Cloudflare Workers AI(REST,有免费额度)真问真答——
  生成 @cf/google/gemma-4-26b-a4b-it,判定 @cf/qwen/qwen3-30b-a3b-fp8,
  产出同一套 AnswerTrace,前端零改动。
"""

from dotenv import load_dotenv

load_dotenv()  # 读取 apps/api/.env(CF_ACCOUNT_ID / CF_API_TOKEN)

from fastapi import FastAPI  # noqa: E402
from fastapi.middleware.cors import CORSMiddleware  # noqa: E402

from app.routers import ask, demo, media, upload  # noqa: E402

app = FastAPI(title="answer-trace api", version="0.1.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173", "http://127.0.0.1:5173"],
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(demo.router, prefix="/api")
app.include_router(ask.router, prefix="/api")
app.include_router(media.router, prefix="/api")
app.include_router(upload.router, prefix="/api")
