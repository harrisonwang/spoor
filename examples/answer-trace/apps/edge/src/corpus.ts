// 当前语料（KV 态）。等价于 apps/api/app/services/corpus.py 的全局单例，但落到 KV：
// 每次上传生成 corpusId，前端在后续 /ask、/media 带回它，实现多用户隔离。
// 上传后即为对话依据；未上传（无 corpusId）则回退到内置 byd.md。
//
// 注意：KV 是最终一致的，上传响应不要 read-after-write 自己刚写的键——
// upload handler 用本地 docs 直接拼响应，KV 只供后续请求读。

import type { Env } from "./config";
import * as store from "./store";

const TTL = 60 * 60 * 24; // 24h：演示数据自清理

interface StoredDoc {
  name: string;
  markdown: string;
}
interface StoredCorpus {
  docs: StoredDoc[];
}

export interface DocBytes {
  name: string;
  markdown: string;
  bytes: Uint8Array;
}

/** 把多份解析结果拼成单篇 markdown（与 corpus.py markdown() 一致）。 */
export function joinMarkdown(docs: StoredDoc[]): string {
  return docs.map((d) => `# 文件:${d.name}\n\n${d.markdown}`).join("\n\n");
}

/** 上传语料的 source 引用（与 corpus.py source_ref() 一致）。 */
export function uploadedSourceRef(
  docs: StoredDoc[],
  joined: string,
): { documentId: string; title: string; pages: number } {
  const first = docs[0].name;
  const title = docs.length === 1 ? first : `${first} 等 ${docs.length} 个文件`;
  return { documentId: "uploaded", title, pages: joined.split("## Page ").length - 1 };
}

/** 写入一份语料，返回 corpusId。原始字节按 doc 序号单独存（供 extract_media）。 */
export async function setDocs(env: Env, docs: DocBytes[]): Promise<string> {
  const corpusId = crypto.randomUUID();
  const stored: StoredCorpus = { docs: docs.map((d) => ({ name: d.name, markdown: d.markdown })) };
  await env.CORPUS.put(`corpus:${corpusId}`, JSON.stringify(stored), { expirationTtl: TTL });
  await Promise.all(
    docs.map((d, i) => env.CORPUS.put(`raw:${corpusId}:${i}`, d.bytes, { expirationTtl: TTL })),
  );
  return corpusId;
}

async function getCorpus(env: Env, corpusId: string): Promise<StoredCorpus | null> {
  const raw = await env.CORPUS.get(`corpus:${corpusId}`);
  return raw ? (JSON.parse(raw) as StoredCorpus) : null;
}

/** 对话依据的整篇 markdown：有 corpusId 用 KV 语料，否则回退内置。 */
export async function markdown(env: Env, corpusId: string | null): Promise<string> {
  if (corpusId) {
    const c = await getCorpus(env, corpusId);
    if (c && c.docs.length) return joinMarkdown(c.docs);
  }
  return store.documentMarkdown();
}

export async function sourceRef(
  env: Env,
  corpusId: string | null,
): Promise<{ documentId: string; title: string; pages: number }> {
  if (corpusId) {
    const c = await getCorpus(env, corpusId);
    if (c && c.docs.length) return uploadedSourceRef(c.docs, joinMarkdown(c.docs));
  }
  return store.sourceRef();
}

/** 按索引取文档含原始字节（供 extract_media）。无 corpusId 时 index=0 回退内置演示。 */
export async function getDoc(
  env: Env,
  corpusId: string | null,
  index: number,
): Promise<DocBytes | null> {
  if (corpusId) {
    const c = await getCorpus(env, corpusId);
    if (!c || index < 0 || index >= c.docs.length) return null;
    const buf = await env.CORPUS.get(`raw:${corpusId}:${index}`, "arrayBuffer");
    if (!buf) return null;
    const doc = c.docs[index];
    return { name: doc.name, markdown: doc.markdown, bytes: new Uint8Array(buf) };
  }
  if (index === 0) {
    return { name: "byd_report.pdf", markdown: store.documentMarkdown(), bytes: store.builtinRawBytes() };
  }
  return null;
}
