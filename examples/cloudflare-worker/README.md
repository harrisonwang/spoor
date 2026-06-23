# Cloudflare Worker 文档解析 API

Worker 接收 `POST` 原始文档 bytes。请提供 `x-filename` 和 `content-type`，
让 spoor 能可靠检测输入。它使用发布的 `@harrisonwang/spoor-wasm`，可以不依赖
仓库 Rust 构建独立部署。

从 `v0.8.3` 起，默认 WASM 包包含 DOCX、XLSX、PDF、PPTX、HTML、EPUB、
IPYNB 及基础格式。示例设置 16 MiB 请求/解析上限。它是公开演示 API，没有
身份认证、租户隔离、持久化或限流；生产部署必须补上这些边界并评估 Workers
CPU/内存限制。

`src/spoor.ts` 会显式实例化 WASM 模块，因为 Cloudflare 导入 `.wasm` 时得到
`WebAssembly.Module`。

```bash
cd examples/cloudflare-worker
npm ci
npm run dev
```

本地请求：

```bash
curl http://localhost:8787
curl -X POST http://localhost:8787 \
  -H 'x-filename: 报告.docx' \
  -H 'content-type: application/vnd.openxmlformats-officedocument.wordprocessingml.document' \
  --data-binary @报告.docx
```

部署到 `workers.dev`：

```bash
npx wrangler login
npm run deploy
```

线上地址为 `spoor-document-cleaner.harrisonwang.workers.dev`。示例没有声明
自定义 `cpu_ms`，可部署到 Workers Free；复杂文档是否能在免费 CPU 配额内完成
取决于内容结构，不能只按文件字节数判断。
