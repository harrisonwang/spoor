# pith

离线、单二进制、CLI-first 的 LLM-friendly 文档预处理工具。

`pith` 把文档转成 LLM 直接能读的形式：**文档型** 内容（PDF / DOCX / PPTX / EPUB / IPYNB / HTML / Markdown / text / code）输出 Markdown；**表格型** 内容（CSV / XLSX）输出 schema + preview 的 JSON。它的定位不是 OCR、云服务、MCP server 或通用格式互转工具，而是一个稳定、可脚本化、可在本地处理敏感文件的 CLI。

## 核心定位

| 内容形态 | 默认输出 | 谁会消费它 |
| --- | --- | --- |
| **文档型**：PDF、DOCX、PPTX、EPUB、IPYNB、HTML/URL、Markdown、text、code | Markdown | 顺序读取的人或 LLM |
| **表格型**：CSV、XLSX | JSON（headers + preview + range） | 程序、脚本、RAG pipeline、Agent tool、LLM 当 context 时 |

人想看 XLSX 原貌请用 Excel / WPS / Numbers——那不是 `pith` 的市场。`pith` 只服务"把内容喂给 LLM / 脚本 / pipeline" 这一类需求。

## 使用

```bash
pith report.pdf                                    # 文档型 → Markdown
pith report.docx slides.pptx                       # 文档型 → Markdown
pith data.xlsx                                     # 表格型 → JSON
pith data.csv | jq '.tables[0].headers'            # 表格型 → JSON
cat data.csv | pith --format csv -                 # 从 stdin 读取（- 表示标准输入）
pith https://example.com/article                   # 文档型 → Markdown
pith "*.pdf"                                       # glob
pith report.pdf | llm "Summarize risks and action items"
```

`pith` 支持多个输入，并会在程序内部展开本地 glob，例如 `*.pdf`、`docs/**/*.md`。URL 不会被当作 glob 展开。

输出模式按文件格式自动分派；`-m` 用来显式覆盖默认值（详见下面"输出模式"）。

## 帮助输出

```text
离线、单二进制、CLI-first 的 LLM-friendly 文档预处理工具

Usage:
  pith [OPTIONS] <input>...

Arguments:
  <input>...  文件路径、URL、本地 glob，或 - 表示标准输入；可传多个，URL 与 - 不参与 glob 展开。

Options:
      --format <format>    覆盖自动 format 检测（默认按 magic-byte / 扩展名推断）。 [possible values: html, markdown, pdf, docx, xlsx, pptx, csv, ipynb, epub, text]
  -m, --mode <mode>        覆盖默认输出模式；表格型（CSV/XLSX）默认 json，其他默认 md。
      --sheet <name>       XLSX 限定 sheet；找不到时报错并列出可用 sheets。CSV 无此概念，自动忽略。
      --rows <first:last>  限定数据行的 Excel 行号区间，例如 `5:104`（含两端）。与 --limit/--offset 互斥。
      --columns <columns>  按列名筛选，逗号分隔；找不到时报错并列出可用列。
      --limit <n>          每个 table 最多返回多少数据行；默认 100。
      --offset <n>         跳过前 N 条数据行再应用 --limit；默认 0。
      --max-output-bytes <n>
                           整次命令 stdout 的最大字节数；默认 262144（256 KiB）。
      --max-parse-bytes <n>
                           解析输入、中间文本和容器解压内容的共享字节预算；默认 67108864（64 MiB）。
  -h, --help               显示帮助。
  -V, --version            显示版本。

For tables (CSV/XLSX), the recommended pattern is:

  pith file.xlsx                              # see structure + preview
  pith file.xlsx --sheet L1 --rows 5:104      # read a slice
  pith file.xlsx --columns 分类,技能          # filter columns

pith bounds JSON previews by default (first 100 data rows per table) and
caps total CLI output at 256 KiB. Use --limit/--rows to narrow tables or
--max-output-bytes to raise the total output cap. Parsing uses a shared
64 MiB data-volume budget by default; raise it with --max-parse-bytes.

Examples:
  pith report.pdf
  pith data.xlsx
  pith data.csv | jq '.tables[]'
  cat data.csv | pith --format csv -
  pith https://example.com/article
  pith "*.pdf"
  pith report.pdf | llm "Summarize risks and action items"
```

## 安装

macOS / Linux：

```bash
brew install harrisonwang/tap/pith
```

Windows（Scoop）：

```powershell
scoop bucket add harrisonwang https://github.com/harrisonwang/scoop-bucket
scoop install pith
```

跨平台（npm，企业内网友好）：

```bash
npm install -g @harrisonwang/pith
```

`@harrisonwang/pith` 是 JS 薄壳，通过 `optionalDependencies` 按平台拉对应的 `@harrisonwang/pith-<platform>-<arch>` 子包——零 postinstall 脚本，完全走 npm registry。

跨平台源码安装（需要 Rust toolchain）：

```bash
cargo install --git https://github.com/harrisonwang/pith
```

当前不发布到 crates.io，也不把 `cargo binstall pith` 作为安装承诺。推荐普通用户优先用 Homebrew / Scoop / npm，它们安装的都是 GitHub Release 里的预构建单二进制。

从源码构建：

```bash
cargo build --release
./target/release/pith file.docx
```

## 输出模式

### Markdown：文档型的默认

stdout 为 Markdown-like 正文。目标是结构清楚、token 经济、便于直接塞进 LLM 上下文。

当前原则：

- 保留标题、段落、列表、表格、链接、脚注、sheet、slide、chapter 等语义结构。
- 丢弃字体、字号、颜色、边距、主题、动画、装饰 shape 等纯视觉样式。
- 对 sheet、slide、chapter 等内容块保留清楚边界。
- 不做 OCR，不执行 notebook，不计算 Excel 公式。

CSV/XLSX 也可以走 Markdown（`-m md`），但仅推荐用于小表终端 peek；大表不保证可用，请走 JSON。

整次命令的 stdout 默认最多 `262144` bytes（256 KiB），多个输入和 glob
共享这一总预算。可用 `--max-output-bytes <n>` 提高上限，但更推荐先缩小输入范围。
Markdown 被截断时，末尾会包含显式 marker，stderr 也会输出 warning：

```markdown
> [!WARNING]
> pith 输出在 262144 字节的总上限处被截断。内容不完整；请缩小输入范围，或用 --max-output-bytes <n> 调高上限。
```

批处理中的 skipped/error diagnostics 最多详细输出前 20 条，其余合并成 omitted
summary，避免大量坏文件把 stderr / CI 日志撑爆。

解析输入、ZIP 声明的总解压量、提取后的文本/表格和多输入保留结果共享默认
`67108864` bytes（64 MiB）解析预算。超过预算时，`pith` 返回非零退出码和结构化错误：

```json
{
  "is_error": true,
  "code": "parse_budget_exceeded",
  "reason": "超出解析预算",
  "hint": "解析在 ... 阶段超出了 67108864 字节的数据量预算。请缩小输入范围，或用 --max-parse-bytes <n> 调高上限。",
  "recoverable": true
}
```

可用 `--max-parse-bytes <n>` 调整。该限制是对可控输入、中间结果和解压数据体积的保守预算，
用于在分配前或增长过程中尽早失败；它不是操作系统级精确 RSS 硬限制，第三方解析库仍可能有短时额外开销。

### 结构化错误与 code

`pith` 的结构化错误是 stderr 上的单行 JSON 信封：`{is_error, code, reason, hint, recoverable}`。
**消费者应按 `code` 分支**；`reason` / `hint` 是中文展示文本，可能改写，不要对它们做字符串匹配。
当前的稳定 code：

| code | 含义 | 建议动作 |
| --- | --- | --- |
| `image_only_pdf` | PDF 没有文本层 | 需要外部 OCR；pith 不做 OCR |
| `parse_budget_exceeded` | 超过共享解析预算 | 缩小输入，或 `--max-parse-bytes <n>` |
| `unsupported_format` | 无法识别输入格式 | 用 `--format` 显式指定 |
| `encrypted_pdf` | PDF 受密码保护 | 先解除密码；不可恢复，勿重试 |
| `legacy_or_encrypted_office` | OLE/CFB 容器（加密 Office 或旧版 .doc/.xls/.ppt） | 解除密码或另存为 docx/xlsx/pptx |
| `invalid_container` | 文件为空、损坏或扩展名与内容不符 | 确认文件完整，或用 `--format` 指定真实格式 |

### JSON：表格型的默认

stdout 为 `pith-table-json-v2`。CSV/XLSX 默认走这条路。其他格式使用 `-m json` 会返回错误并提示使用 `-m md`。

JSON 是表格型的 LLM-friendly 表示——给 LLM headers + preview + row_count + range，**不**给它全量 dump：

```json
{
  "schema_version": "pith-table-json-v2",
  "usage": "收窄输出：--sheet <name>、--rows <first:last>（Excel 行号，含两端）、--columns <a,b,c>、--limit <n>、--offset <n>。默认预览 = 每个 table 前 100 条数据行。--rows 与 --limit/--offset 互斥。",
  "tables": [
    {
      "source": "data.xlsx",
      "format": "xlsx",
      "sheet": "Revenue",
      "workbook_sheets": ["Revenue", "Cost", "Headcount"],
      "title": "Revenue Plan",
      "range": "A1:C1201",
      "column_count": 3,
      "header_row": 3,
      "headers": {
        "month":   { "column_index": 0 },
        "region":  { "column_index": 1 },
        "revenue": { "column_index": 2 }
      },
      "preamble": {
        "row": 2,
        "content": { "source": "finance export" }
      },
      "rows": [
        { "row": 4, "month": "2026-01", "region": "APAC", "revenue": "1000" }
      ],
      "row_range": { "first": 4, "last": 103 },
      "truncated": true,
      "warnings": ["preview limited to first 100 data rows out of 1198"]
    }
  ],
  "truncated": false,
  "warnings": []
}
```

**Schema 设计原则：**

| 字段 | 作用 |
| --- | --- |
| 顶层 `usage` | 一行字符串告诉消费者怎么收窄；让 JSON 自描述，不强依赖 `--help` 或外部 wrapper |
| `tables[]` 扁平化 | 单文件、多 sheet、多文件一律落进同一个数组，消费者迭代逻辑一致 |
| `workbook_sheets` | XLSX 才有；告诉消费者同 workbook 里还有哪些 sheet，避免它自己 group |
| `headers` 为 object | LLM 直接从 key 读字段名；`column_index` 留给程序 |
| `rows[]` 直接是 field→value 映射 | `row.分类` 比 `row.values["分类"]` 更直观；保留 `row` 字段标记 Excel 行号 |
| `preamble` 单独对象 | 与 `rows` 结构对称，`preamble.content` 也是 field→value |
| `row_range` 用 `first`/`last` | 语义比 `start`/`end` 更直观 |
| 顶层 `truncated` + `warnings[]` | 告诉消费者整次命令输出是否因总量上限而不完整 |
| table `truncated` + `warnings[]` | 告诉消费者单个 table preview 是否不完整，决定要不要再调一次 |

`sheet` / `workbook_sheets` 只出现在 XLSX，`delimiter` 只出现在 CSV，`title` / `preamble` 可能为空。JSON 不从 Markdown 反解析，也不承诺 DOCX/PDF/PPTX/IPYNB 的通用 block schema。

收窄 flag（`--sheet`、`--rows`、`--columns`、`--limit`、`--offset`）已实现，适合在 Agent 看到 `truncated: true` 后二次读取。

## Agent / 工具集成

`pith` 不做 MCP server。集成靠两个 zero-overhead snippet：

### 自建 Agent：包成 tool

```python
import json, subprocess

def read_table(path: str, sheet: str | None = None, rows: str | None = None,
               columns: list[str] | None = None) -> dict:
    """Extract structured table data from CSV/XLSX. Returns headers + preview + row_count."""
    args = ["pith", "-m", "json", path]
    if sheet:   args += ["--sheet", sheet]
    if rows:    args += ["--rows", rows]
    if columns: args += ["--columns", ",".join(columns)]
    return json.loads(subprocess.check_output(args))
```

对应 OpenAI / Anthropic function-calling 的 tool schema：

```json
{
  "name": "read_table",
  "description": "Extract structured table data from CSV/XLSX. Returns headers, preview rows (first 100), row_count, range. Use sheet/rows/columns to narrow.",
  "input_schema": {
    "type": "object",
    "properties": {
      "path":    { "type": "string" },
      "sheet":   { "type": "string", "description": "XLSX sheet name" },
      "rows":    { "type": "string", "description": "Excel row range, e.g. '5:104'" },
      "columns": { "type": "array", "items": { "type": "string" } }
    },
    "required": ["path"]
  }
}
```

### 开源 Coding Agent / .cursorrules / AGENTS.md

```markdown
For CSV/XLSX files, prefer `pith <path>` over reading the raw file or writing pandas/openpyxl.
`pith` outputs JSON with headers + preview rows + row_count + range.
Narrow large tables with `--sheet`, `--rows <first:last>`, `--columns <a,b,c>`.

For PDF/DOCX/PPTX, use `pith <path>` to get Markdown.
```

把这一段贴进 `.cursorrules`、`AGENTS.md`、`.clinerules` 或对应工具的 system instruction 即可。

### Agent Skill

仓库内提供一个最小 Skill：`skills/pith/SKILL.md`。它不增加新协议，只把 `pith` 的调用策略分发给 Agent：什么时候优先用 `pith`、表格为什么默认 JSON、看到 `truncated` 后如何用 `--sheet` / `--rows` / `--columns` 继续收窄。

推荐安装方式：

```bash
brew install harrisonwang/tap/pith
npx skills add harrisonwang/pith -y -g
```

Skill 安装交给 multi-agent installer 处理；以 `~/.agents/skills` 作为 universal skill 目录，再按需同步或链接到 Claude Code、Codex、Cursor、GitHub Copilot 等具体 Agent。`pith` 不要求用户手工维护 `~/.codex/skills`。

然后在新的 Agent 会话里尝试：

```text
$pith 读取 tests/fixtures/xlsx/02_multi_sheets.xlsx，告诉我有哪些 sheet 和字段
$pith 读取 tests/fixtures/csv/10_large.csv，只看 id,value 的前 10 行
$pith 读取 tests/fixtures/docx/01_basic.docx，总结标题和列表结构
$pith 读取 tests/fixtures/pptx/03_with_notes.pptx，提取 slide 和 speaker notes
```

## 支持格式

| 格式 | 默认输出 | 当前策略 | 主要缺口 |
| --- | --- | --- | --- |
| DOCX | Markdown | 标题、段落、列表、表格、链接、脚注；默认接受 tracked changes | comments/endnotes、图片 alt/caption、复杂 numbering restart |
| XLSX | JSON | sheet + workbook_sheets + headers + preview rows + 收窄 flag（--sheet/--rows/--columns/--limit/--offset）；日期转 ISO-like；公式用 cached value | 1904 date system |
| PPTX | Markdown | slide 顺序、标题/正文、表格、speaker notes | 坐标阅读顺序、bullet 层级、chart/image placeholder |
| PDF | Markdown | text layer passthrough + `## Page N` page boundary | 断词修复、多栏阅读顺序、页眉页脚去重 |
| EPUB | Markdown | OPF spine 顺序 | 复用 HTML renderer，保留 chapter 内 heading/list/link/table |
| IPYNB | Markdown | markdown + code cells；丢弃 outputs/raw cells | 可选短 text output |
| CSV/TSV | JSON | 编码识别、delimiter 识别、preview rows、range、收窄 flag（--rows/--columns/--limit/--offset）；大文件 row cap | — |
| HTML/URL | Markdown | article/main/body 抽取，heading/list/link/table 转 Markdown | 更稳定 readability、pre/code、blockquote、image alt/caption |
| Markdown/text/code | Markdown | passthrough | 代码文件 fenced block 策略 |

Image-only PDF 没有可提取的 text layer。此时 `pith` 返回非零退出码，并在 stderr
输出机器可读错误，明确提示需要 OCR，避免 Agent 把空输出当作成功：

```json
{"is_error":true,"code":"image_only_pdf","reason":"纯图片 PDF（无文本层）","hint":"该 PDF 没有文本层，需要 OCR，但 pith 不执行 OCR。","recoverable":true}
```

## 设计边界

`pith` 不是 `extract-text` 的逐字复刻，也不是 Docling/Marker/LlamaParse 这类重型文档智能系统的替代品。它坚持：

- 本地、离线、单二进制。
- CLI-first，能被 shell、CI、Claude Code、Codex、Cursor、自建 Agent 等直接调用。
- LLM-friendly：文档型 → Markdown，表格型 → JSON。
- 结构清楚、token 经济、可审计，而不是像素级还原。
- 不默认引入 OCR、VLM、LLM 增强、云服务、GUI 或 MCP server。
- 不和 Excel / WPS / Numbers 竞争"人看表格原貌"这件事。

## Roadmap

P0：

- 默认输出模式按 format 分派：CSV/XLSX → json，其他 → md。
- CSV/XLSX table JSON v2：扁平 `tables[]` + 顶层 `usage` + `workbook_sheets`。
- 表格收窄 flag：`--sheet`、`--rows`、`--columns`、`--limit`、`--offset`。
- 默认总输出上限：整次命令共享 256 KiB，Markdown 使用 truncation marker，JSON 保持合法并设置顶层 `truncated`。
- 默认解析预算：输入、ZIP 总解压量、提取结果和多输入保留结果共享 64 MiB，超限返回结构化错误。（已完成）
- PDF page boundary：PDF 每页输出 `## Page N`。（已完成）
- ZIP 安全层：已有 entry cap、per-entry size cap、compression ratio cap 和 archive total decompressed cap；解析总预算可由 `--max-parse-bytes` 配置。

P1：

- EPUB/HTML renderer 统一。

已完成：

- stdin/pipe：`cat file.csv | pith --format csv -`（`-` 表示标准输入；无扩展名时 format 靠 magic-byte 或 `--format`）。

P2/P3：

- Markdown 大表降级：`-m md` 处理大表时按小表/中表/超大表分档。文档型主路径不依赖此条。
- 稳定 typed Rust core API，并在 benchmark 验证收益后拆分 `pith-core` / CLI、优先发布 PyO3 binding。设计见 [`docs/CORE_PYTHON_ARCHITECTURE.md`](docs/CORE_PYTHON_ARCHITECTURE.md)。
- Homebrew/GitHub Release 分发完善，后续再评估 winget/apt。
- 可选 OCR/VLM backend，默认关闭。

不做：

- MCP server——shell + tool wrapper 已经够用，社区可以做 `pith-mcp` thin wrapper。
- `pith inspect` 子命令——JSON 默认输出（metadata + preview）已经覆盖 inspect 的全部价值。
- 通用 block-oriented JSON——文档型用 Markdown 已经够好，硬塞 JSON 会破坏顺序读取的语义。
- `pith chunk` / 文档分块——文档是顺序读介质，截断是尾部小概率问题：缩小输入或 `--max-output-bytes` 重跑即可。为它造"清单 + 切片 + 收敛"整套机制是为尾部问题造整套机制，已明确否决。

## 测试

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

快照测试使用 `insta`。没有安装 `cargo-insta` 时，可以用：

```bash
INSTA_UPDATE=always cargo test
```

测试用例的设计意图和覆盖缺口记录在 `docs/test-matrix/`。新增 fixture 时，先更新对应格式的测试矩阵，再接受 snapshot。
