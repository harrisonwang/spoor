// answer-trace 边缘后端：单个 Cloudflare Worker，复刻原 FastAPI 的 /api/* 契约。
// /api/demo（内置三轮）· /api/ask（Workers AI 真问真答）· /api/upload（spoor-wasm 解析）
// · /api/media（取内嵌图）。语料存 KV，corpusId 在前端后续请求里带回。

import { cfg, llmEnabled, type Env } from "./config";
import * as corpus from "./corpus";
import { buildTrace } from "./matcher";
import { rewriteSpoorImages } from "./mediaUrls";
import { extractMedia, parseBytes, type TableEntry } from "./spoor";
import * as store from "./store";
import { count as countTokens } from "./tokens";

const MAX_REQUEST_BYTES = 16 * 1024 * 1024;

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === "OPTIONS") return new Response(null, { status: 204, headers: cors() });

    const url = new URL(request.url);
    const path = url.pathname;

    try {
      if (path === "/api/health") return json({ status: "ok" });
      if (path === "/api/demo" && request.method === "GET") return await demo(request, env);
      if (path === "/api/ask" && request.method === "POST") return await ask(request, env);
      if (path === "/api/upload" && request.method === "POST") return await upload(request, env);
      if (path === "/api/media" && request.method === "GET") return await media(url, env);
      return json({ detail: "not found" }, 404);
    } catch (err) {
      return json({ detail: `内部错误:${String(err)}` }, 500);
    }
  },
};

// ── /api/demo ───────────────────────────────────────────────────────────────
async function demo(request: Request, env: Env): Promise<Response> {
  const origin = new URL(request.url).origin;
  const d = store.loadDemo();
  const meta = d.source as { documentId: string; title: string; pages: unknown[] };
  const raw = store.documentMarkdown();
  const md = rewriteSpoorImages(raw, origin, 0);
  return json({
    source: {
      documentId: meta.documentId,
      title: meta.title,
      pages: meta.pages.length,
      markdown: md,
      tokens: countTokens(raw),
      contextLimit: cfg(env).contextLimit,
    },
    traces: d.traces,
  });
}

// ── /api/ask ─────────────────────────────────────────────────────────────────
async function ask(request: Request, env: Env): Promise<Response> {
  const body = (await request.json().catch(() => ({}))) as { question?: string; corpusId?: string };
  const question = (body.question ?? "").trim();
  if (!question) return json({ detail: "问题不能为空" }, 400);
  if (!llmEnabled(env)) {
    return json(
      {
        detail:
          "未配置模型后端:设 AT_BASE_URL + AT_API_KEY(OpenRouter/DeepSeek/z.ai 等),或 CF_ACCOUNT_ID + CF_API_TOKEN。",
      },
      503,
    );
  }
  const corpusId = body.corpusId ?? null;
  try {
    const md = await corpus.markdown(env, corpusId);
    const src = await corpus.sourceRef(env, corpusId);
    return json(await buildTrace(env, question, md, src));
  } catch (exc) {
    return json({ detail: `模型调用或解析失败:${String(exc)}` }, 502);
  }
}

// ── /api/upload ──────────────────────────────────────────────────────────────
async function upload(request: Request, env: Env): Promise<Response> {
  const declared = Number(request.headers.get("content-length") ?? 0);
  if (declared > MAX_REQUEST_BYTES) return json({ detail: "请求超过此演示的 16 MiB 上限。" }, 413);

  const form = await request.formData();
  // workers-types 把 getAll 标成 string[]，运行时其实是 File；按 arrayBuffer 鸭子判定取文件。
  const files = (form.getAll("files") as unknown[]).filter(
    (f): f is File => typeof f === "object" && f !== null && "arrayBuffer" in f,
  );

  const docs: corpus.DocBytes[] = [];
  const results: { name: string; ok: boolean; chars?: number; error?: string }[] = [];
  for (const f of files) {
    const name = f.name || "file";
    const data = new Uint8Array(await f.arrayBuffer());
    try {
      const md = parseToMarkdown(name, data); // 单个文件失败不连累其它
      docs.push({ name, markdown: md, bytes: data });
      results.push({ name, ok: true, chars: md.length });
    } catch (exc) {
      results.push({ name, ok: false, error: String(exc) });
    }
  }

  let corpusId: string | undefined;
  if (docs.length) corpusId = await corpus.setDocs(env, docs);

  // 用本地 docs 直接拼响应，避免 read-after-write 读到刚写的 KV 键（最终一致）。
  const joined = docs.length ? corpus.joinMarkdown(docs) : store.documentMarkdown();
  const src = docs.length ? corpus.uploadedSourceRef(docs, joined) : store.sourceRef();
  const origin = new URL(request.url).origin;
  const md = rewriteSpoorImages(joined, origin, 0, corpusId);

  return json({
    files: results,
    source: src,
    markdown: md,
    tokens: countTokens(joined),
    contextLimit: cfg(env).contextLimit,
    corpusId,
  });
}

// ── /api/media ───────────────────────────────────────────────────────────────
async function media(url: URL, env: Env): Promise<Response> {
  const uri = url.searchParams.get("uri");
  if (!uri) return json({ detail: "缺少 uri 查询参数。" }, 400);
  const docRaw = Number(url.searchParams.get("doc") ?? "0");
  const doc = Number.isFinite(docRaw) ? docRaw : 0;
  const corpusId = url.searchParams.get("corpus");

  const docInfo = await corpus.getDoc(env, corpusId, doc);
  if (!docInfo) return json({ detail: "document not found" }, 404);

  try {
    const raw = extractMedia(docInfo.bytes, uri, docInfo.name, MAX_REQUEST_BYTES);
    return new Response(raw, {
      headers: { ...cors(), "content-type": contentType(raw), "cache-control": "public, max-age=3600" },
    });
  } catch (exc) {
    return json({ detail: `extract media failed: ${String(exc)}` }, 422);
  }
}

// ── helpers ──────────────────────────────────────────────────────────────────
function parseToMarkdown(name: string, data: Uint8Array): string {
  const res = parseBytes(data, name, MAX_REQUEST_BYTES);
  if (res.content.kind === "document") return res.content.value.markdown;
  return tablesToMarkdown(res.content.value.tables);
}

// 表格类（CSV/XLSX）渲染成 markdown 表格，让定位器一视同仁地在文本里找证据。
function tablesToMarkdown(tables: TableEntry[]): string {
  const out: string[] = [];
  for (const t of tables) {
    const name = (t.sheet as string) || (t.title as string) || (t.source as string) || "表";
    const headersMeta = t.headers ?? {};
    const headers = Object.keys(headersMeta).sort(
      (a, b) => headersMeta[a].column_index - headersMeta[b].column_index,
    );
    out.push(`## ${name}\n`);
    if (headers.length) {
      out.push("| " + headers.join(" | ") + " |");
      out.push("| " + headers.map(() => "---").join(" | ") + " |");
      for (const row of t.rows ?? []) {
        out.push("| " + headers.map((h) => String(row[h] ?? "")).join(" | ") + " |");
      }
    }
    out.push("");
  }
  return out.join("\n");
}

// 按魔数判定图片类型，无需 URI 后缀（PDF 整页图是 SVG、内嵌图可能 PNG/JPEG）。
function contentType(raw: Uint8Array): string {
  if (raw[0] === 0x89 && raw[1] === 0x50 && raw[2] === 0x4e && raw[3] === 0x47) return "image/png";
  if (raw[0] === 0xff && raw[1] === 0xd8 && raw[2] === 0xff) return "image/jpeg";
  if (raw[0] === 0x47 && raw[1] === 0x49 && raw[2] === 0x46) return "image/gif";
  if (
    raw[0] === 0x52 && raw[1] === 0x49 && raw[2] === 0x46 && raw[3] === 0x46 &&
    raw[8] === 0x57 && raw[9] === 0x45 && raw[10] === 0x42 && raw[11] === 0x50
  ) {
    return "image/webp";
  }
  const head = new TextDecoder().decode(raw.subarray(0, 64)).trimStart().toLowerCase();
  if (head.startsWith("<?xml") || head.startsWith("<svg")) return "image/svg+xml";
  return "application/octet-stream";
}

function cors(): Record<string, string> {
  return {
    "access-control-allow-origin": "*",
    "access-control-allow-methods": "GET, POST, OPTIONS",
    "access-control-allow-headers": "content-type",
    "x-content-type-options": "nosniff",
  };
}

function json(data: unknown, status = 200): Response {
  return Response.json(data, { status, headers: { ...cors(), "cache-control": "no-store" } });
}
