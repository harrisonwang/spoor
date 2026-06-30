"""token 估算 —— 用 tiktoken(o200k_base)数 token。

用途:上传解析后告诉用户"这份文档大概多少 token、会不会超模型上下文"。
跨模型的**近似**值(对中文偏保守/偏高,正好是安全侧:它说塞得下基本就塞得下)。
首次会下载一份小编码表(~2MB)并缓存;要精确对齐某模型再换该模型的分词器。
"""

from functools import lru_cache

import tiktoken


@lru_cache(maxsize=1)
def _enc() -> "tiktoken.Encoding":
    return tiktoken.get_encoding("o200k_base")


def count(text: str) -> int:
    if not text:
        return 0
    return len(_enc().encode(text))
