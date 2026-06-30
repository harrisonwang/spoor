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


# ── 表格单元格兜底(第③档):坐标重组 quote 仍能定位 ──────────────────────────


def test_table_coordinate_quote_locates_row():
    # 判定模型对表格给的 quote 常是『列名 行名 数值』重组——前两档子串匹配必落空,
    # 但数值在原文表格行里;兜底应命中,且把整行作证据。
    r = loc.locate(MD, "2024A\n营业总收入（百万元）\n777102")
    assert r is not None
    assert r["page"] == 1
    assert "777102" in r["hit"]
    assert "营业总收入" in r["hit"]  # 整行作证据,行名可见


def test_table_anchor_picks_correct_column():
    # 锚定数值本身(而非列名),即便不匹配列头也能落到正确单元格所在行。
    r = loc.locate(MD, "2025E 归母净利润（百万元） 53128")
    assert r is not None
    assert "53128" in r["hit"]


def test_table_percent_value_locates():
    r = loc.locate(MD, "毛利率(%) 2024A 19.44")
    assert r is not None
    assert "19.44" in r["hit"]


def test_fabricated_label_on_real_number_rejected():
    # 数字真实存在(777102),但杜撰的标签词不在该行 → 不可蹭定位,保持「无法核验」。
    assert loc.locate(MD, "海外业务收入 777102") is None


def test_lone_digit_anchor_rejected():
    # 坐标重组使整句无法精确匹配,只剩个位数可作锚点——太弱,不得据此定位。
    # (行名真实存在且该行含数字 7,但兜底因锚点过弱而拒绝,不会误命中。)
    assert loc.locate(MD, "营业总收入（百万元） 7") is None
