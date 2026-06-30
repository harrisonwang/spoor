// 数据出口:demo(内置三轮)+ ask(真问真答)+ upload(多文件→pyspoor 解析)。
// api 不在时 demo 回退到内置 fixture(含 byd.md,所以离线也能下钻看原文)。

import type {
    AnswerTrace,
    DemoPayload,
    UploadResult,
} from "@answer-trace/protocol";
import demo from "@answer-trace/protocol/fixtures/demo.json";
import bydMarkdown from "@answer-trace/protocol/fixtures/byd.md?raw";

const API =
    (import.meta.env.VITE_API_URL as string | undefined) ??
    "http://localhost:8000";

// 用内置 demo.json + byd.md 拼出离线 fixture(符合新的 DemoPayload 形状)。
// 离线时 spoor:// 图片无法提取,换成纯文本说明。
const demoSrc = (
    demo as { source: { documentId: string; title: string; pages: unknown[] } }
).source;
const offlineMarkdown = bydMarkdown.replace(
    /!\[([^\]]*)\]\(spoor:\/\/[^)]+\)/g,
    (_, alt) => `*${alt}（离线预览,图片未提取）*`,
);
export const fixture: DemoPayload = {
    source: {
        documentId: demoSrc.documentId,
        title: demoSrc.title,
        pages: demoSrc.pages.length,
        markdown: offlineMarkdown,
        tokens: Math.round(offlineMarkdown.length * 0.6), // 离线粗估
        contextLimit: 8192,
    },
    traces: (demo as { traces: AnswerTrace[] }).traces,
};

export interface LoadResult {
    data: DemoPayload;
    origin: "api" | "fixture";
}

export async function loadDemo(): Promise<LoadResult> {
    try {
        const res = await fetch(`${API}/api/demo`, {
            signal: AbortSignal.timeout(2500),
        });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return { data: (await res.json()) as DemoPayload, origin: "api" };
    } catch {
        return { data: fixture, origin: "fixture" };
    }
}

async function detail(res: Response): Promise<string> {
    try {
        return (
            ((await res.json()) as { detail?: string }).detail ??
            `HTTP ${res.status}`
        );
    } catch {
        return `HTTP ${res.status}`;
    }
}

// 真问真答:api → Cloudflare Workers AI 生成 + 判定。
export async function askQuestion(question: string): Promise<AnswerTrace> {
    const res = await fetch(`${API}/api/ask`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ question }),
    });
    if (!res.ok) throw new Error(await detail(res));
    return (await res.json()) as AnswerTrace;
}

// 多文件上传 → pyspoor 解析 → 设为当前语料,返回新语料 markdown。
export async function uploadFiles(files: File[]): Promise<UploadResult> {
    const form = new FormData();
    for (const f of files) form.append("files", f);
    const res = await fetch(`${API}/api/upload`, {
        method: "POST",
        body: form,
    });
    if (!res.ok) throw new Error(await detail(res));
    return (await res.json()) as UploadResult;
}
