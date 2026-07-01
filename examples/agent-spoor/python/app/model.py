"""OpenAI 兼容 LLM 层（异步）。"""

from __future__ import annotations

import os

from openai import AsyncOpenAI


def _require(name: str) -> str:
    value = os.environ.get(name)
    if not value:
        raise RuntimeError(f"缺少环境变量 {name}，请在 .env 中设置")
    return value


_client: AsyncOpenAI | None = None


def _get_client() -> AsyncOpenAI:
    global _client
    if _client is None:
        _client = AsyncOpenAI(base_url=_require("BASE_URL"), api_key=_require("OPENAI_API_KEY"))
    return _client


async def call_model(messages: list[dict], tools: list[dict]):
    """调用一次模型，返回 (message, usage)。"""
    response = await _get_client().chat.completions.create(
        model=_require("OPENAI_MODEL"),
        messages=messages,
        tools=tools,
        tool_choice="auto",
    )
    choice = response.choices[0]
    if choice.message is None:
        raise RuntimeError("模型返回的消息为空")
    return choice.message, response.usage
