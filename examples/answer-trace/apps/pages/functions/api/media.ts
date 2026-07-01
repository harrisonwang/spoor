// GET /api/media?uri=&doc=&corpus= —— 按 spoor:// 安全 URI 取回内嵌图。
import { type Env } from "../_lib/config";
import * as corpus from "../_lib/corpus";
import { json } from "../_lib/http";
import { MAX_REQUEST_BYTES, contentType } from "../_lib/parse";
import { extractMedia } from "../_lib/spoor";

export const onRequestGet: PagesFunction<Env> = async ({ request, env }) => {
  const url = new URL(request.url);
  const uri = url.searchParams.get("uri");
  if (!uri) return json({ detail: "缺少 uri 查询参数。" }, 400);
  const docRaw = Number(url.searchParams.get("doc") ?? "0");
  const doc = Number.isFinite(docRaw) ? docRaw : 0;
  const corpusId = url.searchParams.get("corpus");

  const docInfo = await corpus.getDoc(env, request.url, corpusId, doc);
  if (!docInfo) return json({ detail: "document not found" }, 404);

  try {
    const raw = extractMedia(docInfo.bytes, uri, docInfo.name, MAX_REQUEST_BYTES);
    return new Response(raw, {
      headers: { "content-type": contentType(raw), "cache-control": "public, max-age=3600" },
    });
  } catch (exc) {
    return json({ detail: `extract media failed: ${String(exc)}` }, 422);
  }
};
