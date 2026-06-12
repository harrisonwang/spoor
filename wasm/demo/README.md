# Browser drag-and-drop demo

Build the browser-targeted WASM package, then serve the repository root:

```bash
cd crates/spoor-wasm
npm run build:web
cd ../../wasm/demo
npm run dev
```

Open `/wasm/demo/` on the printed local URL. Files are parsed entirely in the
browser.
