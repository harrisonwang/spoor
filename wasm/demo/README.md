# WASM 底层开发回归页

这个目录不是主产品演示，而是仓库内的低层 WASM 开发页与 smoke test。它用于
快速验证浏览器目标是否能解析 DOCX、XLSX、PDF、PPTX、HTML、EPUB、IPYNB，
以及共享预算、坏 ZIP、压缩炸弹和旧版 Office 拦截是否仍然有效。

面向用户的浏览器演示请使用 `examples/cloudflare-pages` 和
`examples/local-corpus-explorer`。

```bash
cd crates/spoor-wasm
npm run build:web
cd ../../wasm/demo
npm run dev
```

打开输出地址下的 `/wasm/demo/`。文件只在浏览器内解析，使用 core 默认的
64 MiB 单次解析预算。
