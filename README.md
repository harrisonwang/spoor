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
  <input>...  文件路径、URL 或本地 glob，可传多个；URL 不参与 glob 展开。

Options:
      --format <format>  覆盖自动 format 检测；可选：pdf、docx、xlsx、pptx、epub、csv、ipynb、html、markdown、text。
  -m, --mode <mode>      覆盖默认输出模式；表格型（CSV/XLSX）默认 json，其他默认 md。
  -h, --help             显示帮助。
  -V, --version          显示版本。

Examples:
  pith report.pdf
  pith data.xlsx
  pith data.xlsx -m md           # 终端 peek 小表
  pith data.csv | jq '.tables[]'
  pith https://example.com/article
  pith "*.pdf"
  pith report.pdf | llm "Summarize risks and action items"
```

## 安装

```bash
brew install harrisonwang/tap/pith
cargo install --git https://github.com/harrisonwang/pith
```

当前不发布到 crates.io，也不把 `cargo binstall pith` 作为安装承诺。推荐普通用户优先用 Homebrew，它安装的是 GitHub Release 里的预构建单二进制。

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

### JSON：表格型的默认

stdout 为 `pith-table-json-v2`。CSV/XLSX 默认走这条路。其他格式使用 `-m json` 会返回错误并提示使用 `-m md`。

JSON 是表格型的 LLM-friendly 表示——给 LLM headers + preview + row_count + range，**不**给它全量 dump：

```json
{
  "schema_version": "pith-table-json-v2",
  "usage": "Narrow output with: --sheet <name>, --rows <first:last>, --columns <a,b,c>, --limit <n>, --offset <n>. See --help.",
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
| `truncated` + `warnings[]` | 告诉消费者还有多少没读，决定要不要再调一次 |

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
npx skills add harrisonwang/pith
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
| PDF | Markdown | text layer passthrough | page boundary、断词修复、多栏阅读顺序、页眉页脚去重 |
| EPUB | Markdown | OPF spine 顺序 | 复用 HTML renderer，保留 chapter 内 heading/list/link/table |
| IPYNB | Markdown | markdown + code cells；丢弃 outputs/raw cells | 可选短 text output |
| CSV/TSV | JSON | 编码识别、delimiter 识别、preview rows、range、收窄 flag（--rows/--columns/--limit/--offset）；大文件 row cap | — |
| HTML/URL | Markdown | article/main/body 抽取，heading/list/link/table 转 Markdown | 更稳定 readability、pre/code、blockquote、image alt/caption |
| Markdown/text/code | Markdown | passthrough | 代码文件 fenced block 策略 |

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
- PDF page boundary：多页 PDF 输出 `## Page N`。
- ZIP 安全层补完：已有第一版 entry cap、per-entry size cap、compression ratio cap；仍需 total output cap 和用户可配置 limits。

P1：

- stdin/pipe：`cat file.csv | pith --format csv -`。
- `pith chunk`：按 heading/page/table/slide 分块（仅文档型）。
- EPUB/HTML renderer 统一。

P2/P3：

- Markdown 大表降级：`-m md` 处理大表时按小表/中表/超大表分档。文档型主路径不依赖此条。
- 稳定 Rust library API。
- Homebrew/GitHub Release 分发完善，后续再评估 winget/apt。
- 可选 OCR/VLM backend，默认关闭。

不做：

- MCP server——shell + tool wrapper 已经够用，社区可以做 `pith-mcp` thin wrapper。
- `pith inspect` 子命令——JSON 默认输出（metadata + preview）已经覆盖 inspect 的全部价值。
- 通用 block-oriented JSON——文档型用 Markdown 已经够好，硬塞 JSON 会破坏顺序读取的语义。

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
