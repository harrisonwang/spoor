// 数据出口：内置演示资产作为静态文件放在 static/_demo/，Functions 用 env.ASSETS 取。
// 避免把 .md/.pdf 打进 bundle 的规则不确定性；base 传 request.url 用于拼资源 URL。

import type { Env } from "./config";

export interface DemoFile {
  source: { documentId: string; title: string; pages: unknown[] };
  traces: unknown[];
}

async function asset(env: Env, base: string, path: string): Promise<Response> {
  const res = await env.ASSETS.fetch(new URL(path, base));
  if (!res.ok) throw new Error(`内置演示资源 ${path} 不可用（${res.status}）`);
  return res;
}

export async function loadDemo(env: Env, base: string): Promise<DemoFile> {
  return (await asset(env, base, "/_demo/demo.json")).json();
}

export async function documentMarkdown(env: Env, base: string): Promise<string> {
  return (await asset(env, base, "/_demo/byd.md")).text();
}

export async function sourceRef(
  env: Env,
  base: string,
): Promise<{ documentId: string; title: string; pages: number }> {
  const d = await loadDemo(env, base);
  return { documentId: d.source.documentId, title: d.source.title, pages: d.source.pages.length };
}

export async function builtinRawBytes(env: Env, base: string): Promise<Uint8Array> {
  return new Uint8Array(await (await asset(env, base, "/_demo/byd.pdf")).arrayBuffer());
}
