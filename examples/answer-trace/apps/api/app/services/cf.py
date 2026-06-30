"""Cloudflare Workers AI 客户端 —— 用 OpenAI SDK 指向其 OpenAI 兼容端点。

base_url = https://api.cloudflare.com/client/v4/accounts/{id}/ai/v1
api_key  = Cloudflare API Token(有 Workers AI 权限)
"""

import re
from functools import lru_cache
from typing import Any

from openai import OpenAI

from app import config

# qwen3 这类带 thinking 的模型会先吐 <think>…</think>,会污染 JSON,统一剥掉。
_THINK = re.compile(r"<think>.*?</think>", re.DOTALL)


@lru_cache(maxsize=1)
def _client() -> OpenAI:
    return OpenAI(base_url=config.base_url(), api_key=config.CF_API_TOKEN)


def chat(
    model: str,
    messages: list[dict[str, Any]],
    *,
    temperature: float = 0.0,
    json_mode: bool = False,
) -> str:
    """调用一次 chat,返回纯文本(已剥离 thinking)。json_mode 尽力要求 JSON 输出。"""
    kwargs: dict[str, Any] = {"model": model, "messages": messages, "temperature": temperature}
    if json_mode:
        kwargs["response_format"] = {"type": "json_object"}

    try:
        resp = _client().chat.completions.create(**kwargs)
    except Exception:
        # 个别模型/端点不接受 response_format,去掉重试一次(靠 prompt 约束 JSON)。
        if json_mode:
            kwargs.pop("response_format", None)
            resp = _client().chat.completions.create(**kwargs)
        else:
            raise

    text = resp.choices[0].message.content or ""
    return _THINK.sub("", text).strip()
