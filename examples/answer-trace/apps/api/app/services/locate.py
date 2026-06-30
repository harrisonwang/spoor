"""确定性证据定位 —— phase 2 的反幻觉核心。

判定模型给出"支持某条结论的原文 quote";这里在**真实 spoor 产物**里逐字找它:
找到 → 给出 before/hit/after + span + 页码(可渲染、可下钻);
找不到 → 返回 None,上层据此把该条降级为「无法核验」(模型说有、原文却没有 = 杜撰)。

对模型 quote 的空白差异容忍:先精确找,再做空白归一化找。
"""

import re
from functools import lru_cache

_WS = re.compile(r"\s+")
_PAGE = re.compile(r"##\s*Page\s+(\d+)")
_CTX = 30


def page_of(md: str, pos: int) -> int | None:
    page: int | None = None
    for m in _PAGE.finditer(md):
        if m.start() <= pos:
            page = int(m.group(1))
        else:
            break
    return page


@lru_cache(maxsize=4)
def _stripped(md: str) -> tuple[str, tuple[int, ...]]:
    """去掉所有空白,并记录每个保留字符回到原文的下标。

    中文 quote 的空格(数字/标点周围)常和原文不一致,忽略全部空白来匹配最稳,
    再用下标映射切回原文的精确区间。
    """
    chars: list[str] = []
    idx_map: list[int] = []
    for i, ch in enumerate(md):
        if ch.isspace():
            continue
        chars.append(ch)
        idx_map.append(i)
    return "".join(chars), tuple(idx_map)


def _find_span(md: str, quote: str) -> tuple[int, int] | None:
    idx = md.find(quote)
    if idx != -1:
        return idx, idx + len(quote)
    # 兜底:忽略所有空白再找
    stripped, idx_map = _stripped(md)
    qn = "".join(quote.split())
    if not qn:
        return None
    j = stripped.find(qn)
    if j == -1:
        return None
    return idx_map[j], idx_map[j + len(qn) - 1] + 1


def _clean(s: str) -> str:
    return _WS.sub(" ", s).strip()


def locate(md: str, quote: str) -> dict | None:
    """在 md 里定位 quote;返回可直接进 QuoteEvidence 的片段,找不到返回 None。"""
    q = (quote or "").strip()
    if not q:
        return None
    span = _find_span(md, q)
    if span is None:
        return None
    start, end = span
    return {
        "before": _clean(md[max(0, start - _CTX) : start]),
        "hit": _clean(md[start:end]),
        "after": _clean(md[end : end + _CTX]),
        "span": {"start": start, "end": end},
        "page": page_of(md, start),
    }
