// spoor-wasm 模块的环境声明（wrangler 的 Pages Functions bundler 注入为对应类型）。

declare module "@harrisonwang/spoor-wasm/spoor_wasm_bg.wasm" {
  const mod: WebAssembly.Module;
  export default mod;
}

declare module "@harrisonwang/spoor-wasm/spoor_wasm_bg.js" {
  export function __wbg_set_wasm(exports: WebAssembly.Exports): void;
  export const parse_bytes: (...args: unknown[]) => unknown;
  export const extract_media: (...args: unknown[]) => Uint8Array;
}
