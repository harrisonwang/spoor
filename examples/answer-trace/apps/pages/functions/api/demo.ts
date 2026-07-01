// GET /api/demo —— 内置三轮对话 + 其依据的原文 markdown。
import { cfg, type Env } from "../_lib/config";
import { json } from "../_lib/http";
import { rewriteSpoorImages } from "../_lib/mediaUrls";
import * as store from "../_lib/store";
import { count as countTokens } from "../_lib/tokens";

export const onRequestGet: PagesFunction<Env> = async ({ request, env }) => {
  const base = request.url;
  const d = await store.loadDemo(env, base);
  const raw = await store.documentMarkdown(env, base);
  const md = rewriteSpoorImages(raw, 0);
  return json({
    source: {
      documentId: d.source.documentId,
      title: d.source.title,
      pages: d.source.pages.length,
      markdown: md,
      tokens: countTokens(raw),
      contextLimit: cfg(env).contextLimit,
    },
    traces: d.traces,
  });
};
