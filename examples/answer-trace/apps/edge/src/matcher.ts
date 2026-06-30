// matcher：question + spoor 产物 → AnswerTrace。忠实移植自 apps/api/app/services/matcher.py。
//
// 1) 生成(gen)：仅依据文档回答。
// 2) 判定(judge,JSON)：把回答拆成 text/claim，对每个 claim 给 verdict + 原文 quote。
// 3) 确定性装配：每条 claim 的 quote 必须在文档里逐字定位到，才算数据；
//    定位不到 → 强制降级为「无法核验」(反幻觉闸门)。产出符合 spoor.answer-trace.v1。

import type {
  AnswerPart,
  AnswerTrace,
  Evidence,
  NoEvidence,
  QuoteEvidence,
  SourceRef,
  Verdict,
} from "@answer-trace/protocol";
import { SCHEMA_VERSION } from "@answer-trace/protocol";

import { chat } from "./cf";
import { cfg, type Env } from "./config";
import { locate } from "./locate";

const GEN_SYS =
  "你是严谨的中文金融文档助手。**只依据下面的【文档】回答**,不要编造文档中没有的数字或事实," +
  "数字必须与文档一致。回答简洁、直接给结论。";

const JUDGE_SYS =
  "你是严格的事实核查器。给你【文档】【问题】【回答】。把【回答】**按出现顺序、逐字**拆成片段:" +
  "普通叙述为 text;可核查的事实陈述为 claim。所有片段的 text 顺序拼接必须等于原【回答】(不增删字)。" +
  "**只输出 JSON,不要解释**。/no_think\n" +
  "claim 规则:必须是**完整、含具体数值/指标**的事实句(例:『Q4 归母净利润 150 亿元,同环比+73%/+29%』)。" +
  "**严禁**把光秃秃的字段名/标题(如『归母净利润:』『毛利率:』)单独当一个 claim——" +
  "claim 的 text 必须覆盖到数值本身,把『字段名 + 数值(+变动)』整段作为一个 claim。\n" +
  "每个 claim 的 quote 必须是【文档】里一字不差的原文子串;没有就留空字符串。\n" +
  "依据若来自表格,quote 取**数据所在的整行原文**(形如 `| 行名 | 值1 | 值2 |`)或**该单元格数值本身**," +
  "切勿把列名/行名/单位拆开重拼成一句——那不是原文子串,会定位失败。\n" +
  '格式:{"parts":[' +
  '{"type":"text","text":"..."},' +
  '{"type":"claim","text":"含数值的完整事实句","verdict":"supported|partial|unsupported",' +
  '"quote":"文档中支持它的原文(逐字),没有就\\"\\"","note":"为何 partial/unsupported,可空",' +
  '"truth":"正确值,仅 unsupported 时可填,可空"}' +
  "]}\n" +
  "判据:supported=核心数值/事实被文档明确支持(允许等价换算,如 1335亿 = 133,454百万);" +
  "partial=措辞接近但需复核(概括/约数等);" +
  "unsupported=文档没有或与文档矛盾(quote 留空)。";

interface JudgeSeg {
  type?: string;
  text?: string;
  verdict?: Verdict;
  quote?: string;
  note?: string;
  truth?: string;
}

async function generate(env: Env, question: string, md: string): Promise<string> {
  return chat(
    env,
    cfg(env).genModel,
    [
      { role: "system", content: GEN_SYS },
      { role: "user", content: `【文档】\n${md}\n\n【问题】${question}` },
    ],
    { temperature: 0.3 },
  );
}

async function judge(env: Env, question: string, answer: string, md: string): Promise<{ parts?: JudgeSeg[] }> {
  const raw = await chat(
    env,
    cfg(env).judgeModel,
    [
      { role: "system", content: JUDGE_SYS },
      { role: "user", content: `【文档】\n${md}\n\n【问题】${question}\n\n【回答】${answer}` },
    ],
    { temperature: 0, jsonMode: true },
  );
  return parseJson(raw);
}

function parseJson(raw: string): { parts?: JudgeSeg[] } {
  let s = raw.trim().replace(/^```(?:json)?|```$/gm, "").trim();
  const m = s.match(/\{[\s\S]*\}/);
  if (!m) throw new Error(`判定模型未返回 JSON:${raw.slice(0, 200)}`);
  return JSON.parse(m[0]) as { parts?: JudgeSeg[] };
}

export async function buildTrace(
  env: Env,
  question: string,
  md: string,
  source: SourceRef,
): Promise<AnswerTrace> {
  const answer = await generate(env, question, md);
  const parsed = await judge(env, question, answer, md);

  const parts: AnswerPart[] = [];
  const evidence: Evidence[] = [];
  let n = 0;

  for (const seg of parsed.parts ?? []) {
    if (seg.type !== "claim") {
      parts.push({ type: "text", text: seg.text ?? "" });
      continue;
    }

    n += 1;
    const eid = `e${n}`;
    let verdict: Verdict = seg.verdict ?? "unsupported";
    const quote = (seg.quote ?? "").trim();
    const note = seg.note || null;
    const truth = seg.truth || null;
    const located = quote ? locate(md, quote) : null;

    let ev: Evidence;
    if (located && (verdict === "supported" || verdict === "partial")) {
      const q: QuoteEvidence = {
        id: eid,
        kind: "quote",
        verdict,
        page: located.page,
        before: located.before,
        hit: located.hit,
        after: located.after,
        span: located.span,
      };
      if (note) q.note = note;
      ev = q;
    } else {
      // 反幻觉:模型说支持、但原文里定位不到(或本就没给依据)→ 一律无法核验。
      verdict = "unsupported";
      const ne: NoEvidence = {
        id: eid,
        kind: "none",
        verdict: "unsupported",
        page: null,
        reason:
          note ?? (quote ? "模型给出的依据在原文中未找到,疑似杜撰。" : "原文中未找到支撑该结论的内容。"),
      };
      if (truth) ne.expectedTruth = truth;
      ev = ne;
    }

    evidence.push(ev);
    parts.push({ type: "claim", text: seg.text ?? "", verdict, evidenceIds: [eid] });
  }

  return {
    schema: SCHEMA_VERSION,
    question,
    answer: parts,
    evidence,
    source,
    audit: {
      parser: "spoor-wasm@0.8",
      generator: cfg(env).genModel,
      judge: cfg(env).judgeModel,
      judgedAt: new Date().toISOString().replace(/\.\d{3}Z$/, "Z"),
    },
  };
}
