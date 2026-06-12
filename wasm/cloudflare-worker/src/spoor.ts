import wasmModule from '@harrisonwang/spoor-wasm/spoor_wasm_bg.wasm';
import * as bindings from '@harrisonwang/spoor-wasm/spoor_wasm_bg.js';

const instance = new WebAssembly.Instance(wasmModule, {
  './spoor_wasm_bg.js': bindings,
});

bindings.__wbg_set_wasm(instance.exports);

export const parseBytes = bindings.parse_bytes;
