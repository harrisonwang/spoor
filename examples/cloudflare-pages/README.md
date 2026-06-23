# Cloudflare Pages：浏览器本地解析 + 放置选择 + 取图 UX

**本示例唯一证明：同一个纯 Rust WASM 包，文件既可「留在浏览器本地解析、零上传」，也可一键切到 Cloudflare 边缘执行——放在哪里由你选；并能点开正文里的内嵌图片占位符、就地把那张图取出来内联渲染。** 全格式解析是各示例共享的底座；这里独有的是无头的 [`../cloudflare-worker`](../cloudflare-worker/) 给不了的：有 UI、文件可不离浏览器、把「本地 vs 边缘」这个放置选择本身做成可切换演示。只想要一个纯后端解析 API，去看 cloudflare-worker。

示例的两种模式：

- **本地 WASM**：文件始终留在浏览器内，直接解析并检索。
- **Pages Function**：把原始 bytes 发送到 `/api/parse`，在 Cloudflare Workers Runtime 中运行同一个 WASM 包。

从 `v0.8.3` 起，发布的 `@harrisonwang/spoor-wasm` 默认包含 DOCX、XLSX、
PDF、PPTX、HTML、EPUB、IPYNB 以及基础文本格式。示例在两种模式下都设置
16 MiB 请求/解析上限；生产环境还应增加身份认证、限流和外层超时。
两种模式都返回完整 `ParseResult`，页面会显式提示解析完整性 warnings，Agent
应按 warning code 与 page/slide 位置决定是否信任或转交外部 OCR/VLM。

浏览器构建使用 `vite-plugin-wasm`。Pages Functions 导入 `.wasm` 时得到
`WebAssembly.Module`，因此 `src/edge-spoor.ts` 会显式实例化模块。

## 内嵌图片提取（extract_media）

解析 DOCX/PPTX/PDF 后，正文里的内嵌图会以 `spoor://` 占位符出现，**不分格式**：DOCX/PPTX 图片
`spoor://{docx,pptx}/part/.../media/*`、PDF 内嵌图 `spoor://pdf/obj/{id}/{gen}`、PDF 整页矢量图
`spoor://pdf/page/{n}`（SVG）。页面把这些占位符统一列成缩略图，点击才调用 `extract_media` 按需提取
**单个**资源字节（懒取、单资源，与 Agent 只取相关图的用法一致），按字节嗅探出 MIME 后用 `Blob`
在浏览器内联渲染（PDF 整页 SVG 也能正确显示）——**图片提取始终在浏览器本地完成，与解析走本地还是
边缘无关，文件不离开浏览器**。点「加载图片示例（DOCX）」可载入随站点附带的样例直接体验。

本示例依赖 `@harrisonwang/spoor-wasm ^0.8.17`（DOCX/PPTX/PDF 占位符提取均已覆盖）。spoor 不解码或
理解图片内容，要理解图片仍需把字节交给外部 VLM。`public/sample-image-doc.docx` 是一份真实的
纯图片文档示例，所有内嵌图片已量化重压（约 5 MiB → 349 KiB），便于随仓库携带。

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
