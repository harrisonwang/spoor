// spoor-wasm 初始化 + 类型化封装。初始化写法照搬 examples/cloudflare-worker。
import wasmModule from "@harrisonwang/spoor-wasm/spoor_wasm_bg.wasm";
import * as bindings from "@harrisonwang/spoor-wasm/spoor_wasm_bg.js";

const instance = new WebAssembly.Instance(wasmModule, {
  "./spoor_wasm_bg.js": bindings,
});
bindings.__wbg_set_wasm(instance.exports as WebAssembly.Exports);

export interface TableEntry {
  source: string;
  format: string;
  sheet?: string;
  title?: string;
  headers: Record<string, { column_index: number }>;
  rows: Record<string, string>[];
  [k: string]: unknown;
}

export interface ParseResult {
  content:
    | { kind: "document"; value: { source: string; format: string; markdown: string } }
    | { kind: "tables"; value: { tables: TableEntry[]; serialized_bytes: number } };
  warnings: { code: string; message: string; location?: { kind: string; number: number } }[];
  stats: { input_bytes: number; output_bytes: number; format: string; page_count?: number };
}

/** 解析文档/表格字节为结构化结果。与 pyspoor 的 parse_bytes 行为等价。 */
export function parseBytes(bytes: Uint8Array, sourceName?: string, maxParseBytes?: number): ParseResult {
  return bindings.parse_bytes(bytes, sourceName, undefined, undefined, maxParseBytes) as ParseResult;
}

/** 按 spoor:// 安全 URI 取回单个内嵌媒体原始字节。 */
export function extractMedia(
  bytes: Uint8Array,
  uri: string,
  sourceName?: string,
  maxParseBytes?: number,
): Uint8Array {
  return bindings.extract_media(bytes, uri, sourceName, undefined, undefined, maxParseBytes);
}
