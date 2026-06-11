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
- pith 没有 `--pages`、`--page`、`--ocr`、`--json`、`--output` 这类 flag；输出模式只有 `-m md|json`。PDF 页码已经以 `## Page N` 标题内嵌在 Markdown 输出里，要定位某一页，直接在输出里找对应标题。

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

判断截断只认两个稳定信号，不要匹配具体警告文案（文案是中文人话，可能改写）：

- Markdown：末尾出现 `> [!WARNING]` 引用块。
- 表格 JSON：顶层 `truncated: true` 表示整次输出因总量上限不完整；table 内 `truncated: true` 表示该 table preview 不完整。

处理方式：

- `pith` 默认对整次命令 stdout 使用 256 KiB 总量上限，多文件和 glob 共享预算。看到截断信号时，不要假设后续内容不存在。
- 表格优先用 `--sheet` / `--rows` / `--columns` 收窄后重试；文档型缩小输入范围，或在确有必要时用 `--max-output-bytes <n>` 提高上限。
- 总量截断后的 table `row_range` 描述截断前的选择范围，不代表所有范围内 rows 都已返回。
- 批处理时，被跳过的输入会同时出现在 stderr（`warning:` 行）和 stdout 末尾的 `> [!WARNING]` 块里（Markdown 模式）或 JSON 顶层 `warnings[]` 里；最多详列 20 条，看到「另有 N 个失败」时不要假设其余输入成功。
- 输出只有一个 `> [!NOTE]` 块、说明「未抽取到文本内容」时，表示该输入本身没有可提取的文本（或格式判断有误，可用 `--format` 覆盖重试），不要把空内容当成读取成功后凭空编造正文。

## 出错时怎么处理

`pith` 的结构化错误是 stderr 上的单行 JSON：`{"is_error":true,"code":"...","reason":"...","hint":"...","recoverable":...}`。**只按 `code` 分支**；`reason` 和 `hint` 是中文展示文本，可能改写，不要对它们做字符串匹配。

| code | 含义 | 正确的下一步 |
| --- | --- | --- |
| `image_only_pdf` | PDF 没有文本层 | 停止猜测内容，明确告诉用户需要外部 OCR；pith 不做 OCR |
| `parse_budget_exceeded` | 输入、容器解压内容或提取结果超过共享解析预算 | 不要猜测未读取内容；优先缩小输入，确有必要时用 `--max-parse-bytes <n>` |
| `unsupported_format` | 无法识别输入格式 | 已知文件类型时用 `--format` 显式指定；真正不支持的格式（图片等）如实告知用户 |
| `encrypted_pdf` | PDF 受密码保护 | 不可恢复，停止重试；请用户先解除密码。pith 没有 `--password` 这类 flag |
| `legacy_or_encrypted_office` | OLE/CFB 容器：加密 Office 文档或旧版 .doc/.xls/.ppt | 不可恢复，停止重试；请用户解除密码或另存为 docx/xlsx/pptx |
| `invalid_container` | 文件为空、损坏或扩展名与内容不符 | 确认文件完整；扩展名不可靠时用 `--format` 指定真实格式重试一次 |

其他注意：

- 没有 JSON 信封的报错（stderr 纯文本）按 `reason` 原样如实上报，不要反复重试探测。
- 文档格式用 `-m json` 报错时，去掉 `-m json` 重新跑。
- 除非用户要求做取证排查，不要绕过 `pith` 手动解压 Office/EPUB 文件。
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
