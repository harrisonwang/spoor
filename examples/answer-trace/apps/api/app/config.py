"""配置:Cloudflare Workers AI(OpenAI 兼容端点)+ 模型选择,全走环境变量。"""

import os

# 生成模型(出答案)与判定模型(逐条核验)。可用环境变量覆盖。
GEN_MODEL = os.getenv("AT_GEN_MODEL", "@cf/google/gemma-4-26b-a4b-it")
JUDGE_MODEL = os.getenv("AT_JUDGE_MODEL", "@cf/qwen/qwen3-30b-a3b-fp8")

CF_ACCOUNT_ID = os.getenv("CF_ACCOUNT_ID", "")
CF_API_TOKEN = os.getenv("CF_API_TOKEN", "")


def cf_enabled() -> bool:
    """两个密钥都在,才允许真正调用 Workers AI。"""
    return bool(CF_ACCOUNT_ID and CF_API_TOKEN)


def base_url() -> str:
    """Workers AI 的 OpenAI 兼容 base_url。"""
    return f"https://api.cloudflare.com/client/v4/accounts/{CF_ACCOUNT_ID}/ai/v1"
