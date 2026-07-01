---
name: spoor
description: 读取 PDF/DOCX/XLSX/CSV/PPTX/EPUB/HTML 等文档，转成 LLM 可消费的文本（文档→Markdown，表格→JSON），支持按页/表/行列收窄与内嵌图提取。
---

# spoor 文档解析技能

当用户要"读 / 总结 / 提取"一个**非纯文本**文档（PDF、Word、Excel、PPT、EPUB、网页…）时，
用 `run_shell` 调用 `spoor` CLI。纯文本 / 代码文件仍用 `read_file`。

## 基本用法

- 读整篇：`spoor data/byd.pdf`
- 读表格（看结构 + 前几行）：`spoor data/sales.csv`
- 大 PDF 只取某几页：`spoor data/byd.pdf --pages 1:3`
- XLSX 指定表 + 列 + 行数：`spoor data/book.xlsx --sheet Sheet1 --columns 分类,金额 --limit 20`
- 行区间（与 --limit/--offset 互斥）：`spoor data/sales.csv --rows 2:4`

## 输出怎么读

- 文档型 → Markdown；表格型 → JSON（headers + 前 N 行 preview + range）。
- 结尾可能有 **warnings**，例如 `pdf_page_no_text_layer` 表示某页是扫描件、没有可提取文本层——
  要**如实转达用户**，不要假装读到了那页内容。

## 提取内嵌图（交给 VLM）

- 正文里出现 `![...](spoor://...)` 占位符时，用 `spoor <文件> --extract <spoor://...>` 提取；
  run_shell 会把图片存到 `.spoor-media/`。spoor 本身不解读图片。

## 出错怎么办

- 按稳定错误 `code` 处理并把 hint 转达用户：`unsupported_format`、`encrypted_pdf`、
  `parse_budget_exceeded`（改用收窄参数）等。
