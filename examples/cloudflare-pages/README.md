# Cloudflare Pages 浏览器与边缘解析示例

这个可部署示例同时展示两种形态：

- **本地 WASM**：文件始终留在浏览器内，直接解析并检索。
- **Pages Function**：把原始 bytes 发送到 `/api/parse`，在 Cloudflare Workers Runtime 中运行同一个 WASM 包。

从 `v0.8.3` 起，发布的 `@harrisonwang/spoor-wasm` 默认包含 DOCX、XLSX、
PDF、PPTX、HTML、EPUB、IPYNB 以及基础文本格式。示例在两种模式下都设置
16 MiB 请求/解析上限；生产环境还应增加身份认证、限流和外层超时。
两种模式都返回完整 `ParseResult`，页面会显式提示解析完整性 warnings，Agent
应按 warning code 与 page/slide 位置决定是否信任或转交外部 OCR/VLM。

浏览器构建使用 `vite-plugin-wasm`。Pages Functions 导入 `.wasm` 时得到
`WebAssembly.Module`，因此 `src/edge-spoor.ts` 会显式实例化模块。

## 本地运行

```bash
cd examples/cloudflare-pages
npm install

# 仅静态前端，本地 WASM 模式可用，边缘模式不可用
npm run dev

# 完整 Pages + Functions Runtime
npm run dev:pages
```

## 验证与部署

```bash
npm run check
npx wrangler login
npm run deploy
```

Pages 项目地址为 `spoor-pages-demo.pages.dev`。CI 部署需要配置
`CLOUDFLARE_API_TOKEN` 和 `CLOUDFLARE_ACCOUNT_ID`。
