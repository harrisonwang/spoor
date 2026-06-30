// locate.ts 在真实 BYD spoor 产物上的行为。移植自 apps/api/tests/test_locate.py，
// 逐条对应，验证 TS 端与 Python 端三档定位的行为一致。

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { locate } from "../src/locate";

const here = dirname(fileURLToPath(import.meta.url));
const MD = readFileSync(join(here, "../src/data/byd.md"), "utf-8");

describe("locate", () => {
  it("命中第 1 页正文", () => {
    const r = locate(MD, "经营性净现金流 1335 亿元，同降 21%");
    expect(r).not.toBeNull();
    expect(r!.page).toBe(1);
    expect(r!.hit).toContain("1335");
    expect(MD.slice(r!.span.start, r!.span.end).replace(/\n/g, "")).toMatch(/^经营性净现金流/);
  });

  it("容忍空白差异", () => {
    const r = locate(MD, "经营性净现金流  1335 亿元， 同降 21%");
    expect(r).not.toBeNull();
    expect(r!.hit).toContain("1335");
  });

  it("命中第 2 页表格数字", () => {
    const r = locate(MD, "133,454");
    expect(r).not.toBeNull();
    expect(r!.page).toBe(2);
  });

  it("杜撰数字定位不到 → null", () => {
    expect(locate(MD, "营收已达 9,999 亿元，再创历史新高")).toBeNull();
  });

  it("空 quote → null", () => {
    expect(locate(MD, "")).toBeNull();
    expect(locate(MD, "   ")).toBeNull();
  });

  // ── 表格单元格兜底（第③档） ──────────────────────────────────────────────
  it("坐标重组 quote 命中整行", () => {
    const r = locate(MD, "2024A\n营业总收入（百万元）\n777102");
    expect(r).not.toBeNull();
    expect(r!.page).toBe(1);
    expect(r!.hit).toContain("777102");
    expect(r!.hit).toContain("营业总收入");
  });

  it("锚定数值落到正确单元格所在行", () => {
    const r = locate(MD, "2025E 归母净利润（百万元） 53128");
    expect(r).not.toBeNull();
    expect(r!.hit).toContain("53128");
  });

  it("百分比值可定位", () => {
    const r = locate(MD, "毛利率(%) 2024A 19.44");
    expect(r).not.toBeNull();
    expect(r!.hit).toContain("19.44");
  });

  it("真数字 + 杜撰标签 → 不可蹭定位", () => {
    expect(locate(MD, "海外业务收入 777102")).toBeNull();
  });

  it("锚点过弱（个位数）→ 拒绝", () => {
    expect(locate(MD, "营业总收入（百万元） 7")).toBeNull();
  });
});
