"""markdown 预处理:把 spoor 产物的安全 URI 重写为前端可请求的 API 路径。"""

import re

_SPOOR_IMG = re.compile(r"!\[([^\]]*)\]\((spoor://[^)]+)\)")


def rewrite_spoor_images(markdown: str, doc_index: int = 0) -> str:
    """将 markdown 中 ``![alt](spoor://...)`` 重写为指向 ``/api/media`` 的链接。

    ``/api/media?uri=<encoded-uri>&doc=<doc_index>`` 由
    ``routers/media.py`` 处理,通过 ``spoor.extract_media`` 取回原始图片。
    """
    import urllib.parse

    def _replacer(m: re.Match) -> str:
        alt = m.group(1)
        uri = m.group(2)
        encoded = urllib.parse.quote(uri, safe="")
        return f"![{alt}](/api/media?uri={encoded}&doc={doc_index})"

    return _SPOOR_IMG.sub(_replacer, markdown)
