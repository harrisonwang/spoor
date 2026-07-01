// locate 分级核验的确定性档位（金档，无需 LLM）。
// 用受控合成表格隔离验证第④档（数值/单位归一），再在真实 byd.md 上做安全性 sanity。

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { locate } from "../locate";

const here = dirname(fileURLToPath(import.meta.url));
const MD = readFileSync(join(here, "../../../static/_demo/byd.md"), "utf-8");

// 合成表格：单位在列头（百万元），四舍五入后 quote 的数字不是表内数字的子串，
// 故只有第④档（数值归一）能命中，能干净地隔离验证它。
const TABLE_MD = [
  "## Page 1",
  "",
  "| 营业总收入（百万元） | 777102 | 969284 |",
  "| 归母净利润（百万元） | 53128 | 65084 |",
  "",
].join("\n");

describe("金档第④档：数值/单位归一", () => {
  it("『营业总收入 9693 亿』≈ 969284 百万，命中该行（亿↔百万，且 9693 非 969284 子串）", () => {
    const r = locate(TABLE_MD, "营业总收入 9693 亿");
    expect(r).not.toBeNull();
    expect(r!.hit).toContain("969284");
  });

  it("同义词『营收约 9693 亿元』也能命中（数值全表唯一，免标签）", () => {
    const r = locate(TABLE_MD, "营收约 9693 亿元");
    expect(r).not.toBeNull();
    expect(r!.hit).toContain("969284");
  });

  it("杜撰量级『9999 亿』定位不到 → null（不误配）", () => {
    expect(locate(TABLE_MD, "海外收入 9999 亿")).toBeNull();
  });
});

describe("既有档位在真实 byd.md 上", () => {
  it("表格坐标重组（第③档）命中 53128", () => {
    const r = locate(MD, "2024A 归母净利润（百万元） 53128");
    expect(r).not.toBeNull();
    expect(r!.hit).toContain("53128");
  });

  it("空 quote → null", () => {
    expect(locate(MD, "")).toBeNull();
    expect(locate(MD, "   ")).toBeNull();
  });

  it("杜撰量级在真实文档也不误配 → null", () => {
    expect(locate(MD, "海外收入 9999 亿")).toBeNull();
  });
});
