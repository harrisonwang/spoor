// 数据出口：把演示资产打进 Worker。等价于 apps/api/app/services/store.py。
// - loadDemo()：内置三轮对话（demo.json）。
// - documentMarkdown()：内置演示对应的真实 spoor 产物（byd.md）。
// - builtinRawBytes()：内置 PDF 原始字节，供 extract_media 取内嵌图。

import demoJson from "./data/demo.json";
import bydMarkdown from "./data/byd.md";
import bydPdf from "./data/byd.pdf";

interface DemoSource {
  documentId: string;
  title: string;
  pages: unknown[];
}
interface DemoFile {
  source: DemoSource;
  traces: unknown[];
}

const demo = demoJson as DemoFile;

export function loadDemo(): DemoFile {
  return demo;
}

export function documentMarkdown(): string {
  return bydMarkdown;
}

export function sourceRef(): { documentId: string; title: string; pages: number } {
  const s = demo.source;
  return { documentId: s.documentId, title: s.title, pages: s.pages.length };
}

export function builtinRawBytes(): Uint8Array {
  return new Uint8Array(bydPdf);
}
