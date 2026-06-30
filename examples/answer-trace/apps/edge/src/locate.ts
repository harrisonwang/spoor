// 确定性证据定位 —— phase 2 的反幻觉核心。忠实移植自 apps/api/app/services/locate.py。
//
// 判定模型给出"支持某条结论的原文 quote"；这里在真实 spoor 产物里逐字找它：
// 找到 → 给出 before/hit/after + span + 页码（可渲染、可下钻）；
// 找不到 → 返回 null，上层据此把该条降级为「无法核验」（模型说有、原文却没有 = 杜撰）。
//
// 定位分三档，逐档放宽：① 精确子串；② 忽略全部空白再找；③ 表格单元格兜底——
// 判定模型对表格数据给的 quote 常是『列名 行名 数值』的坐标重组，前两档必落空；
// 第三档以最具辨识度的数字为锚点定位，再用标签词校验命中行，把整张表格行作证据。
//
// 区间以 UTF-16 下标计（与前端 markdown.slice(start,end) 一致）。byd.md 全为 BMP 字符，
// 与 Python 端按码点计的结果一致。

const PAGE = /##\s*Page\s+(\d+)/g;
// 数字型 token：带千分位逗号/小数/百分号的数，或纯数字串。作表格证据的"锚点"。
const NUM = /\d[\d,]*(?:\.\d+)?%?/g;
// 标签 token：连续 CJK 或 ≥2 字母的词，用来校验锚点命中的是不是 quote 指的那一行。
const LABEL = /[一-鿿]+|[A-Za-z][A-Za-z]+/g;
const CTX = 30;

export interface Located {
  before: string;
  hit: string;
  after: string;
  span: { start: number; end: number };
  page: number | null;
}

export function pageOf(md: string, pos: number): number | null {
  let page: number | null = null;
  for (const m of md.matchAll(PAGE)) {
    if ((m.index ?? 0) <= pos) page = Number.parseInt(m[1], 10);
    else break;
  }
  return page;
}

// 去掉所有空白，记录每个保留字符回到原文的下标。中文 quote 的空格常和原文不一致，
// 忽略全部空白来匹配最稳，再用下标映射切回精确区间。缓存最近一次（≈lru_cache maxsize=1）。
let _cacheMd = "";
let _cache: { s: string; map: number[] } | null = null;

function stripped(md: string): { s: string; map: number[] } {
  if (md === _cacheMd && _cache) return _cache;
  const chars: string[] = [];
  const map: number[] = [];
  for (let i = 0; i < md.length; i++) {
    const ch = md[i];
    if (/\s/u.test(ch)) continue;
    chars.push(ch);
    map.push(i);
  }
  _cacheMd = md;
  _cache = { s: chars.join(""), map };
  return _cache;
}

function findSpan(md: string, quote: string): [number, number] | null {
  const idx = md.indexOf(quote);
  if (idx !== -1) return [idx, idx + quote.length];
  // 兜底：忽略所有空白再找
  const { s, map } = stripped(md);
  const qn = quote.replace(/\s+/gu, "");
  if (!qn) return null;
  const j = s.indexOf(qn);
  if (j === -1) return null;
  return [map[j], map[j + qn.length - 1] + 1];
}

function allOccurrences(md: string, needle: string): number[] {
  const nn = needle.replace(/\s+/gu, "");
  if (!nn) return [];
  const { s, map } = stripped(md);
  const out: number[] = [];
  let i = s.indexOf(nn);
  while (i !== -1) {
    out.push(map[i]);
    i = s.indexOf(nn, i + 1);
  }
  return out;
}

function lineBounds(md: string, pos: number): [number, number] {
  const ls = md.lastIndexOf("\n", pos - 1) + 1;
  const le = md.indexOf("\n", pos);
  return [ls, le === -1 ? md.length : le];
}

function maxByLen(arr: string[]): string {
  let best = arr[0];
  for (const x of arr) if (x.length > best.length) best = x; // 严格 >：同长取首个（同 Python max）
  return best;
}

// 表格单元格兜底（第③档）。① 取锚点：优先带分隔符（逗号/小数/百分号）的数，再取最长——
// 金融数值常带分隔符，可避开年份（如 2024A）误当锚点。② 找锚点全部出现。
// ③ 用 quote 的标签词（行名/列名）校验命中行，防数字撞车；命中则把整张表格行作证据。
function anchoredSpan(md: string, quote: string): [number, number] | null {
  const nums = [...quote.matchAll(NUM)].map((m) => m[0]);
  if (nums.length === 0) return null;
  const sepNums = nums.filter((n) => /[,.%]/.test(n));
  const anchor = maxByLen(sepNums.length ? sepNums : nums);
  const hasSep = /[,.%]/.test(anchor);
  if (anchor.length < 3 && !hasSep) return null; // 太短（如个位数）不足以辨识，放弃

  const occ = allOccurrences(md, anchor);
  if (occ.length === 0) return null;

  const labels = [...quote.matchAll(LABEL)].map((m) => m[0]).filter((w) => w.length >= 2);
  const scored = occ.map((start) => {
    const [ls, le] = lineBounds(md, start);
    const line = md.slice(ls, le);
    const score = labels.reduce((acc, w) => acc + (line.includes(w) ? 1 : 0), 0);
    return { score, start, ls, le, line };
  });
  scored.sort((a, b) => b.score - a.score); // 稳定排序：同分保持原序
  const best = scored[0];

  // 接受：有标签词时必须至少命中一个（防杜撰标签蹭到巧合数字）；
  // 无标签词时仅当锚点全文唯一且够辨识。
  if (labels.length > 0) {
    if (best.score < 1) return null;
  } else if (!(occ.length === 1 && (anchor.length >= 4 || hasSep))) {
    return null;
  }

  // 命中在 markdown 表格行里 → 整行作证据；否则只圈锚点本身。
  if (best.line.trimStart().startsWith("|")) {
    const ts = best.ls + (best.line.length - best.line.trimStart().length);
    const te = best.le - (best.line.length - best.line.trimEnd().length);
    return [ts, te];
  }
  // 可选收紧：散文里数字+词组的巧合共现可能误命中。若要更强的反幻觉保证，
  // 把下一行改为 `return null;`，让非表格行不走兜底。当前与 Python 端保持一致。
  return [best.start, best.start + anchor.length];
}

function clean(s: string): string {
  return s.replace(/\s+/gu, " ").trim();
}

/** 在 md 里定位 quote；返回可直接进 QuoteEvidence 的片段，找不到返回 null。 */
export function locate(md: string, quote: string): Located | null {
  const q = (quote ?? "").trim();
  if (!q) return null;
  let span = findSpan(md, q);
  if (span === null) span = anchoredSpan(md, q); // 第③档：表格单元格兜底
  if (span === null) return null;
  const [start, end] = span;
  return {
    before: clean(md.slice(Math.max(0, start - CTX), start)),
    hit: clean(md.slice(start, end)),
    after: clean(md.slice(end, end + CTX)),
    span: { start, end },
    page: pageOf(md, start),
  };
}
