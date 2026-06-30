"""确定性证据定位 —— phase 2 的反幻觉核心。

判定模型给出"支持某条结论的原文 quote";这里在**真实 spoor 产物**里逐字找它:
找到 → 给出 before/hit/after + span + 页码(可渲染、可下钻);
找不到 → 返回 None,上层据此把该条降级为「无法核验」(模型说有、原文却没有 = 杜撰)。

定位分三档,逐档放宽:① 精确子串;② 忽略全部空白再找;③ 表格单元格兜底——
判定模型对表格数据给的 quote 常是『列名 行名 数值』的坐标重组,在 markdown 表格里
这三段跨表头行/数据行、并不连续,前两档必然落空;第三档以 quote 里最具辨识度的数字
为锚点定位,再用标签词校验命中行,把整张表格行作证据(否则正确的表格事实会被误判为杜撰)。
"""

import re
from functools import lru_cache

_WS = re.compile(r"\s+")
_PAGE = re.compile(r"##\s*Page\s+(\d+)")
_CTX = 30

# 数字型 token:带千分位逗号/小数/百分号的数,或纯数字串。作表格证据的"锚点"。
_NUM = re.compile(r"\d[\d,]*(?:\.\d+)?%?")
# 标签 token:连续 CJK 或 ≥2 字母的词,用来校验锚点命中的是不是 quote 指的那一行。
_LABEL = re.compile(r"[一-鿿]+|[A-Za-z][A-Za-z]+")


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


def _all_occurrences(md: str, needle: str) -> list[int]:
    """needle 在 md 中全部出现的起始下标(忽略空白),返回原文坐标。"""
    nn = "".join(needle.split())
    if not nn:
        return []
    stripped, idx_map = _stripped(md)
    out: list[int] = []
    i = stripped.find(nn)
    while i != -1:
        out.append(idx_map[i])
        i = stripped.find(nn, i + 1)
    return out


def _line_bounds(md: str, pos: int) -> tuple[int, int]:
    ls = md.rfind("\n", 0, pos) + 1
    le = md.find("\n", pos)
    return ls, len(md) if le == -1 else le


def _anchored_span(md: str, quote: str) -> tuple[int, int] | None:
    """表格单元格兜底:以 quote 里最具辨识度的数字为锚点定位(见模块 docstring 第③档)。

    ① 取锚点:优先带分隔符(逗号/小数/百分号)的数,再取最长——金融数值常带分隔符,
       这样能避开年份(如 2024A)误当锚点。② 在原文找锚点全部出现;
    ③ 用 quote 的标签词(行名/列名)校验命中行,防数字撞车;命中则把整张表格行作证据。
    """
    nums = _NUM.findall(quote)
    if not nums:
        return None
    sep_nums = [n for n in nums if any(c in n for c in ",.%")]
    anchor = max(sep_nums or nums, key=len)
    has_sep = any(c in anchor for c in ",.%")
    if len(anchor) < 3 and not has_sep:  # 太短(如个位数)不足以辨识,放弃
        return None

    occ = _all_occurrences(md, anchor)
    if not occ:
        return None

    labels = [w for w in _LABEL.findall(quote) if len(w) >= 2]
    scored = []
    for start in occ:
        ls, le = _line_bounds(md, start)
        line = md[ls:le]
        score = sum(1 for w in labels if w in line)
        scored.append((score, start, ls, le, line))
    scored.sort(key=lambda t: t[0], reverse=True)
    best_score, start, ls, le, line = scored[0]

    # 接受:有标签词时必须至少命中一个(防杜撰标签蹭到巧合数字);
    # 无标签词时仅当锚点全文唯一且够辨识。
    if labels:
        if best_score < 1:
            return None
    elif not (len(occ) == 1 and (len(anchor) >= 4 or has_sep)):
        return None

    # 命中在 markdown 表格行里 → 整行作证据(行名/各列值都可见);否则只圈锚点本身。
    if line.lstrip().startswith("|"):
        ts = ls + (len(line) - len(line.lstrip()))
        te = le - (len(line) - len(line.rstrip()))
        return ts, te
    return start, start + len(anchor)


def _clean(s: str) -> str:
    return _WS.sub(" ", s).strip()


def locate(md: str, quote: str) -> dict | None:
    """在 md 里定位 quote;返回可直接进 QuoteEvidence 的片段,找不到返回 None。"""
    q = (quote or "").strip()
    if not q:
        return None
    span = _find_span(md, q)
    if span is None:
        span = _anchored_span(md, q)  # 第③档:表格单元格兜底
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
