"""配置:模型后端(任意 OpenAI 兼容端点)+ 模型 + token 上限。全走环境变量。"""

import os

GEN_MODEL = os.getenv("AT_GEN_MODEL", "@cf/google/gemma-4-26b-a4b-it")
JUDGE_MODEL = os.getenv("AT_JUDGE_MODEL", "@cf/qwen/qwen3-30b-a3b-fp8")

# 直接指定任意 OpenAI 兼容端点(OpenRouter / DeepSeek / z.ai …)。设了就优先用。
AT_BASE_URL = os.getenv("AT_BASE_URL", "")
AT_API_KEY = os.getenv("AT_API_KEY", "")

# 便捷后端:Cloudflare Workers AI(未设 AT_BASE_URL 时回退到它,用账号 ID 拼 base_url)。
CF_ACCOUNT_ID = os.getenv("CF_ACCOUNT_ID", "")
CF_API_TOKEN = os.getenv("CF_API_TOKEN", "")

# 模型上下文上限(token),仅用于前端"是否超限"提示;按所选模型设。
CONTEXT_LIMIT = int(os.getenv("AT_CONTEXT_LIMIT", "32768"))


def base_url() -> str:
    if AT_BASE_URL:
        return AT_BASE_URL
    return f"https://api.cloudflare.com/client/v4/accounts/{CF_ACCOUNT_ID}/ai/v1"


def api_key() -> str:
    return AT_API_KEY or CF_API_TOKEN


def llm_enabled() -> bool:
    """配齐任一后端即可:自定义端点(AT_BASE_URL+AT_API_KEY)或 Cloudflare(账号 ID + Token)。"""
    if AT_BASE_URL and AT_API_KEY:
        return True
    return bool(CF_ACCOUNT_ID and CF_API_TOKEN)
