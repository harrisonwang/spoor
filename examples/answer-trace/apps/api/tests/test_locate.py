"""定位器在真实 BYD spoor 产物上的行为(不需要 CF)—— phase 2 反幻觉核心。"""

from app.services import locate as loc
from app.services import store

MD = store.document_markdown()


def test_locate_found_on_page_1():
    r = loc.locate(MD, "经营性净现金流 1335 亿元，同降 21%")
    assert r is not None
    assert r["page"] == 1
    assert "1335" in r["hit"]
    # span 能切回原文
    assert MD[r["span"]["start"] : r["span"]["end"]].replace("\n", "").startswith("经营性净现金流")


def test_locate_whitespace_tolerant():
    # 模型 quote 多/少空格也应命中
    r = loc.locate(MD, "经营性净现金流  1335 亿元， 同降 21%")
    assert r is not None
    assert "1335" in r["hit"]


def test_locate_table_number_on_page_2():
    r = loc.locate(MD, "133,454")
    assert r is not None
    assert r["page"] == 2


def test_fabricated_quote_not_found():
    # 杜撰数字定位不到 → None → 上层强制降级为「无法核验」
    assert loc.locate(MD, "营收已达 9,999 亿元，再创历史新高") is None


def test_empty_quote_is_none():
    assert loc.locate(MD, "") is None
    assert loc.locate(MD, "   ") is None
