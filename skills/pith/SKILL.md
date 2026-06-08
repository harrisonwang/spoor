---
name: pith
description: 当 Agent 需要读取或检查本地文件、简单 URL，并把内容变成适合 LLM 使用的上下文时使用 pith。适用于 PDF、DOCX、PPTX、EPUB、IPYNB、HTML/URL、Markdown、文本、代码、CSV/TSV、XLSX 等文件；在总结文档、提取需求、审阅资料、检查表格、基于本地附件回答问题时，优先用 pith，而不是临时写 pandas/openpyxl、手搓解析器或直接 dump 原始文件。
tags:
  - docx
  - xlsx
  - pptx
  - pdf
  - url
  - epub
  - ipynb
---

# Pith

用 `pith` 先读文档和表格。它是本地 CLI 预处理工具，不是 OCR、在线办公 API、MCP server，也不是通用格式转换器。

## 怎么选命令

- 读支持的本地文件时，优先用 `pith <path>`。
- 读简单的 HTML/Markdown 网页时，可以用 `pith <url>`。
- 如果 `pith` 不在 `PATH` 里，并且当前仓库就是 pith 源码仓库，从仓库根目录运行 `cargo run --quiet -- <path>`。
- 文件类型不好判断时，加 `--format <format>` 明确指定。
- 默认输出模式交给 `pith` 判断：文档型输出 Markdown，CSV/XLSX 输出 JSON。
- CSV/XLSX 只有在用户只是想快速瞄一眼小表时，才用 `-m md`。大表不要转 Markdown。
- PDF、DOCX、PPTX、EPUB、IPYNB、HTML、Markdown、文本这类文档不要用 `-m json`。

## 格式规则

文档型内容读 Markdown：

- 包括 PDF、DOCX、PPTX、EPUB、IPYNB、HTML/URL、Markdown、文本和代码。
- 把输出的 Markdown 当作证据来源；留意标题、列表、表格、链接、脚注、slide、sheet、chapter、speaker notes 等结构。

表格型内容读 `pith-table-json-v2`：

- 包括 CSV/TSV 和 XLSX。
- 先看 `tables[]`、`headers`、`range`、`row_range`、`truncated`、`warnings`，再判断要不要继续缩小范围。
- XLSX 要看 `sheet` 和 `workbook_sheets`；一个 sheet 对应一个 table entry。

## 表格读取流程

1. 先运行 `pith <file.csv>` 或 `pith <file.xlsx>`。
2. 看 JSON 里的 table、sheet、headers、range 和 row_range，判断哪部分和问题相关。
3. 如果 `truncated` 是 true，或者表太宽太长，就再跑一次并收窄：
   - 用 `--sheet <name>` 选 XLSX 里的 sheet。
   - 用 `--columns a,b,c` 只看需要的列。
   - 用 `--rows <first:last>` 按 Excel 行号精确读取。
   - 用 `--limit <n>` 和 `--offset <n>` 做窗口采样。
4. 引用表格依据时，带上 source、sheet、列名和 `row_range.first` / `row_range.last`。

## 输出截断时怎么处理

- `pith` 默认对整次命令 stdout 使用 256 KiB 总量上限，多文件和 glob 共享预算。
- Markdown 末尾出现 `> [!WARNING]` 和 `Content is incomplete` 时，不要假设后续内容不存在；应缩小输入范围，或在确有必要时使用 `--max-output-bytes <n>` 提高上限。
- 表格 JSON 顶层 `truncated: true` 表示整次输出因总量上限不完整；table 内 `truncated: true` 表示该 table preview 不完整。优先用 `--sheet` / `--rows` / `--columns` 收窄后重试。
- 总量截断后的 table `row_range` 描述截断前的选择范围，不代表所有范围内 rows 都已返回。
- 批处理 stderr 只详细列出前 20 条失败；看到 additional failures omitted 时，不要假设其余输入成功。

## 出错时怎么处理

- PDF 没有 text layer 时，`pith` 会返回非零退出码，并在 stderr 输出
  `{"is_error":true,"reason":"image-only PDF",...}`。看到这个错误后停止猜测
  PDF 内容，明确告诉用户需要外部 OCR；`pith` 默认不做 OCR。
- `reason` 为 `parse memory limit exceeded` 时，说明输入、容器解压内容或提取结果超过共享解析预算。不要猜测未读取内容；优先缩小输入，确有必要时再用 `--max-parse-bytes <n>` 提高预算。
- Office/EPUB 报 ZIP、archive 或 safety limit 错误时，先当作文件损坏、不支持或超过安全限制处理。除非用户要求做取证排查，不要绕过 `pith` 手动解压。
- 文档格式用 `-m json` 报错时，去掉 `-m json` 重新跑。
- 如果任务是操作飞书、企微、钉钉等在线办公平台，用对应平台的 CLI/API skill；只有内容已经落成本地文件或简单 URL 时，才交给 `pith` 读取。

## 常用例子

```bash
pith report.pdf
pith proposal.docx
pith slides.pptx
pith data.xlsx
pith data.xlsx --sheet Revenue --columns month,region,revenue
pith data.xlsx --sheet Revenue --rows 200:260
pith data.csv --limit 20
pith "docs/**/*.pdf"
```
