# 浏览器本地语料库（批处理 → 检索 → 导出）

**本示例唯一证明：纯浏览器里把「一整批 / 一整个文件夹」的混合文档一次性解析成可检索语料库——单文件失败不阻断整批、跨文档全文检索、并确定性导出 JSONL chunk 与 manifest，全程零上传。** 定位上：[`../cloudflare-worker`](../cloudflare-worker/) 是无头单文档 API、[`../cloudflare-pages`](../cloudflare-pages/) 是单文档的本地↔边缘演练场，这里独有的是**批量 + 检索 + 导出**（喂给索引 / RAG 流水线的前处理）。

这个纯浏览器工作台把一批混合本地文档转换为可检索语料库，全程不上传文件。
它是浏览器侧的主能力示例，取代了功能重复的单文件“本地文件对话”示例。

它展示：

- DOCX、XLSX、PDF、PPTX、HTML、EPUB、IPYNB 与基础格式的混合批处理；
- 单文件失败不阻断整个批次，并展示稳定错误码；
- 展示并导出带 page/slide 位置的解析完整性 warnings；
- 跨文档本地全文检索；
- 内嵌图片（不分格式）：查看器把 `spoor://` 占位符——DOCX/PPTX 图片、PDF 内嵌图（`spoor://pdf/obj/...`）、
  PDF 整页矢量图（`spoor://pdf/page/...`，SVG）——统一列成缩略图，点击才调用 `extract_media` 按需提取
  单个资源字节、按字节嗅探 MIME 后内联渲染（PDF 整页 SVG 也能显示）；
- 确定性的 JSONL chunk 与 manifest 导出，便于接入后续索引流水线。

从 `v0.8.3` 起，发布的 `@harrisonwang/spoor-wasm` 默认包含全部重点格式。
示例为每个文件设置 16 MiB 解析上限，但没有设置文件数量和语料库总大小上限；
实际容量受浏览器内存限制，生产实现应自行增加批次数量、总字节数和取消机制。
XLSX/CSV 仍遵循 spoor 的表格预览契约，默认最多保留每张表前 100 条数据行。
点「加载中文样例」会一并载入 `public/sample-image-doc.docx`（真实纯图片文档，所有
内嵌图片已量化重压），选中后即可在查看器内点击占位符本地提取图片。spoor 不理解
图片内容，要理解图片仍需把字节交给外部 VLM。

```bash
cd examples/local-corpus-explorer
npm install
npm run dev
```

验证与部署：

```bash
npm run check
npm run deploy
```

Pages 项目地址为 `spoor-corpus-demo.pages.dev`。
