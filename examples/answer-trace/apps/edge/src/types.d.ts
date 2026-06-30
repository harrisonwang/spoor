// 资产与 wasm 模块的环境声明（wrangler 的模块规则把它们注入为对应类型）。

declare module "@harrisonwang/spoor-wasm/spoor_wasm_bg.wasm" {
  const mod: WebAssembly.Module;
  export default mod;
}

declare module "@harrisonwang/spoor-wasm/spoor_wasm_bg.js" {
  // wasm-bindgen 胶水，无类型声明：parse_bytes / extract_media / __wbg_set_wasm。
  export function __wbg_set_wasm(exports: WebAssembly.Exports): void;
  export const parse_bytes: (...args: unknown[]) => unknown;
  export const extract_media: (...args: unknown[]) => Uint8Array;
}

declare module "*.md" {
  const text: string;
  export default text;
}

declare module "*.pdf" {
  const bytes: ArrayBuffer;
  export default bytes;
}
