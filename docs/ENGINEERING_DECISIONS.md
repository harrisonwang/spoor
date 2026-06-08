# 工程决策

本文档记录 `pith` 的产品边界和 extraction contract。最新 roadmap 的核心定位是：

> 离线、单二进制、CLI-first、LLM-friendly、可定位。

`pith` 不是为了复刻 `extract-text` 的逐字输出，也不是 Docling/Marker/LlamaParse 这类重型文档智能系统的替代品。它应该成为稳定、轻量、可脚本化的本地 CLI：**按内容形态自动输出 LLM-friendly 表示**——文档型（PDF / DOCX / PPTX / EPUB / IPYNB / HTML / Markdown / text / code）输出 Markdown，表格型（CSV / XLSX）输出 schema + preview 的 JSON。

LLM-friendly 表示按内容形态决定，不按消费者类型决定。同一份 5000 行 XLSX 给 LLM 看 Markdown 表格是灾难（token 爆炸、丢表头）；给它 `{headers, preview_rows, row_count, range}` 是常数 token + 一眼看懂的形态。这就是为什么表格型默认走 JSON 而不是 Markdown，也是为什么 prose 型默认走 Markdown 而不是 JSON——把 prose 切 block 装进 JSON 信封既贵又破坏顺序读取语义。

## 产品边界

坚持：

- 本地、离线、单二进制。
- CLI-first，能被 shell、CI、Claude Code、Codex、Cursor、自建 Agent 等直接调用。
- 默认输出按 format 分派：文档型 → Markdown，表格型 → JSON；`-m` 仅作显式覆盖。
- JSON 自描述：包含 `usage` 字符串和 `workbook_sheets` 结构信息，让消费者不依赖外部 `--help` 也能决定下一步。
- 优先结构清楚、token 经济、可定位、可审计。

不追：

- 云服务、GUI、Agent 平台、MCP server、`pith inspect` 子命令。
- 像素级排版还原、通用格式互转、通用 block-oriented JSON。
- 默认 OCR、VLM、LLM 增强或重模型依赖。
- Firecrawl/Jina Reader 式爬虫能力。
- 和 Excel / WPS / Numbers 竞争"人看表格原貌"这件事。

## 判断标准

"转成某种格式"不自动等于 LLM 友好。输出是否合格按下面标准判断：

1. **形态对路**：文档型走 Markdown，表格型走 JSON；不应让 LLM 读 5000 行的 Markdown 表格，也不应让 LLM 读 prose 切 block 的 JSON。
2. **结构保真**：标题、段落、列表、表格、链接、脚注、页、sheet、slide、chapter 等结构不能轻易丢。
3. **阅读顺序正确**：DOCX 段落顺序、PPTX slide 顺序、EPUB spine 顺序、PDF page 顺序要可预期。
4. **噪声低**：脚本、样式、导航、广告、装饰 shape、空占位符、重复页眉页脚默认不进入正文。
5. **token 经济**：表格型默认走 JSON preview，常数 token；文档型 Markdown 丢弃视觉样式只留语义结构。
6. **JSON 自描述**：表格 JSON 含 `usage` 字符串和 `workbook_sheets`，`truncated` + `warnings` 告诉消费者还有多少没读、怎么继续读。
7. **转换性能可控**：不能为了格式还原引入过重依赖或不受控内存占用。
8. **恶意输入安全**：ZIP 类格式必须有 entry cap、压缩比限制、单 entry 解压大小限制和总输出限制。

## CLI Contract

当前 CLI 是一等入口。`pith -h` 应保持短、稳定、适合复制到 README：

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

CLI 行为约定：

- 默认输出模式由 effective format（含 `--format` override 后）决定：CSV/XLSX → `json`，其他 → `md`。
- `-m` 仅作为显式覆盖；表格型可显式选 `-m md`（终端 peek，大表不保证可用）；非表格格式 `-m json` 返回错误并提示使用 `-m md`。
- `--mode json` 输出 `pith-table-json-v2`，schema 详见下方"输出模式 / json"。
- 多输入在 Markdown 模式下按 `# Source: ...` 分块；JSON 模式下所有表合并到顶层 `tables[]` 数组。
- 本地 glob 由程序内部展开；URL 不参与 glob。
- `--format` 只覆盖文件/URL 内容格式，不定义 URL 抓取策略。
- stdin/pipe 已实现：输入 `-` 读取标准输入；无路径无扩展名，format 走 magic-byte 检测或 `--format` override（表格型从 stdin 需显式 `--format csv`）。`-` 不参与 glob，可与文件混用。
- 表格收窄 flag 已实现：`--sheet`（仅 XLSX；CSV 无 sheet 概念，自动忽略）、`--rows <first:last>`（Excel 行号，含两端）、`--columns <a,b,c>`、`--limit <n>`、`--offset <n>`。`--rows` 与 `--limit`/`--offset` 互斥（clap conflicts_with）。找不到的 sheet/columns 硬错并列出可用列表。

## Library Contract

当前 crate 保持单 package、单 binary。`src/lib.rs` 暴露的正式 API（以 `src/lib.rs` 的 re-export 为准，按调用流程分组）：

- 输入解析：`SourceInput`、`Source`、`is_url`、`resolve_input` → `ResolvedInput`
- format / mode：`Format`、`FormatArg`、`OutputMode`、`default_mode_for`、`ExtractOptions`
- 文档型抽取：`extract_md` → `ExtractedDocument`，`render_documents`
- 表格型抽取：`extract_table_entries`（用 `TableFilter` 收窄）→ `TableEntry`，`render_json`
- table JSON schema 类型：`JsonOutput`、`HeaderInfo`、`PreambleInfo`、`RowRange`、`TABLE_SCHEMA_VERSION`、`TABLE_USAGE`、`a1_range`、`cells_to_values`

extractor 细节保持内部模块，不作为公共 API。第二阶段 workspace / `pith-core` / PyO3
边界与迁移门槛见 [Core 与 Python Binding 架构](CORE_PYTHON_ARCHITECTURE.md)。拆分前先把
公共 `anyhow::Result` 收敛为 typed result/error，并用 benchmark 验证 subprocess 开销是否真实构成瓶颈。

## 输出模式

### `md`

文档型的默认模式。面向模型上下文，优先保留语义结构，丢弃纯视觉信息。

当前策略：

- 文本文档输出 Markdown-like 文本。
- 小表输出 GFM table。
- notebook 输出 markdown cells 和 code cells，默认不输出 cell outputs。
- slide/sheet/chapter 使用明确标题分块。

表格型（CSV/XLSX）使用 `-m md` 是显式 override，仅用于终端 peek 小表；大表行为不保证。

待补策略：

- PDF 加 page boundary（P0）。
- EPUB 复用 HTML Markdown renderer（P1）。
- Markdown 大表降级（小表 GFM / 中表 fenced TSV / 超大表摘要+range+truncation，仅 niche，P2）。
- 所有格式统一 total output cap 和 truncation marker。

### `json`

表格型（CSV/XLSX）的默认模式。schema 为 `pith-table-json-v2`，自描述（含 `usage` 字符串），扁平 `tables[]`。

非表格格式使用 `-m json` 返回错误并提示使用 `-m md`。**不**为 DOCX/PDF/PPTX/IPYNB 设计通用 block-oriented JSON——把 prose 切 block 装进 JSON 信封既贵又破坏顺序读取语义，是错误方向。

```json
{
  "schema_version": "pith-table-json-v2",
  "usage": "Narrow output with: --sheet <name>, --rows <first:last> (Excel row numbers, inclusive), --columns <a,b,c>, --limit <n>, --offset <n>. Default preview = first 100 data rows per table. --rows conflicts with --limit/--offset.",
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

Schema fields:

- top-level：`schema_version`、`usage`、`tables[]`、`truncated`、`warnings[]`
- table 必有：`source`、`format`、`range`、`column_count`、`headers`、`rows`、`row_range`、`truncated`、`warnings`
- table 表格型条件字段：
  - XLSX：`sheet`、`workbook_sheets`、`title`、`header_row`、`preamble`
  - CSV：`delimiter`

Schema 设计理由：

| 字段 | 设计理由 |
|------|----------|
| 顶层 `usage` | 一行字符串告诉消费者怎么收窄；让 JSON 自描述，不依赖外部 `--help` 或 wrapper。`usage` 是单一发布常量 `TABLE_USAGE`；测试 `table_usage_lists_every_narrowing_flag` 从 clap 定义派生收窄 flag 并断言每个都出现在 `TABLE_USAGE` 里，flag 改名或新增却忘了同步会直接挂 CI，避免悄悄漂移成幻觉源。 |
| 顶层 `tables[]` 扁平化 | 单文件、多 sheet、多文件一律落进同一个数组；消费者迭代逻辑一致；多文件混用时按 `source` 字段分组。 |
| `workbook_sheets` 重复在每个 table 上 | 自描述权重高于去重；让单个 table entry 自包含、可独立处理；redundancy 在 XLSX 多 sheet 时是 N×K bytes，可接受。 |
| `headers` 为 object（key→column_index） | LLM 直接从 key 读字段名；`column_index` 留给程序需要按列号定位的场景。 |
| `rows[]` 直接是 field→value 映射 | `row.分类` 比 `row.values["分类"]` 自然；保留 `row` 字段标记 Excel 行号，配合 `row_range` 让消费者能精确引用。 |
| `preamble` 单独对象 | 与 `rows` 结构对称，`preamble.content` 也是 field→value（如果 preamble 行能映射到 headers）；可能为空。 |
| `row_range` 用 `first`/`last` | 语义比 `start`/`end` 更直观，符合 Excel 行号习惯。 |
| 顶层 `truncated` + `warnings[]` | 告诉消费者整次命令输出是否受总量上限影响。 |
| table `truncated` + `warnings[]` | 告诉消费者单个 table preview 是否不完整、可以决定要不要再调一次。JSON preview 当前最多输出前 100 条数据行。 |

CSV JSON 保留 delimiter、headers、preview rows、总数据行数、列数和 A1-style range。XLSX JSON 额外保留 sheet 名、workbook 内其他 sheet 列表、worksheet range、标题行、真实 header row 和 header 前的 preamble。

JSON 不从 Markdown 反解析，也不承诺 DOCX/PDF/PPTX/IPYNB 的通用 block schema。

## 格式决策

### DOCX

保留：

- 标题层级：`#` 到 `######`
- 段落和空行
- 有序/无序列表以及嵌套层级
- 小表 GFM table
- 链接：`[text](url)`
- 脚注：`[^N]` 和文末定义
- Unicode、smart quotes、RTL 文本
- tracked changes 的插入内容

默认丢弃：

- 字体、字号、颜色、边距、对齐
- 纯装饰图片和 shape
- tracked changes 的删除内容
- Word 内部样式细节

待补：

- comments/endnotes
- image alt/caption placeholder
- 复杂 numbering restart
- chart / embedded object placeholder

### XLSX

保留：

- sheet 名
- 单元格文本、数字、布尔值
- 日期/时间，转成 ISO-like 字符串
- formula cached value
- error cell，例如 `#DIV/0!`
- merged cell 的 top-left 值

默认丢弃：

- 样式、颜色、边框、列宽、冻结窗格
- 公式表达式本身，除非后续提供 `--formulas`
- 空白区域

后续规则：

- 小表：GFM table。
- 宽表/长表：fenced TSV/CSV。
- 超大表：摘要 + row/col range + truncation marker。

JSON 规则：

- XLSX 默认走 `-m json`，输出顶层扁平 `tables[]`，每个 sheet 一个 table entry。
- 保留 `source`、`format`、`sheet`、`workbook_sheets`、A1-style `range`、`title`、`column_count`、`header_row`、`headers`、`preamble`、`rows`、`row_range`、`truncated`、`warnings`。
- `workbook_sheets` 在每个 table 上重复出现，方便消费者无需 group 也能知道还有哪些 sheet 可读。
- preview 最多 100 条数据行，超过则 `truncated: true` 并写入 warning。

注意：当前日期转换还没有完整处理 Excel 1904 date system。

### PPTX

保留：

- slide 顺序和 `## Slide N`
- slide title/body 文本
- 小表 GFM table
- speaker notes

默认丢弃：

- 主题、颜色、动画、转场
- 装饰 shape
- 图片二进制内容

待补：

- 按 shape 坐标恢复更接近视觉阅读顺序。
- bullet 层级和 marker。
- chart 数据提取或 chart placeholder。
- image alt/caption placeholder。

### PDF

PDF 是版面格式，不是语义文档。当前只使用 text layer，输出还不够理想。

应该保留：

- page boundary：`## Page N`
- 尽可能正确的阅读顺序
- 重要标题和段落

默认丢弃或压缩：

- 重复页眉页脚
- 页码装饰
- 水印

待补：

- 断词修复
- 多栏阅读顺序

Image-only PDF 没有 text layer 时返回非零退出码，并在 stderr 输出可解析的
结构化错误：`reason: "image-only PDF"`、明确 OCR hint、`recoverable: true`。

### EPUB

保留：

- OPF spine 顺序
- chapter boundary
- HTML 内部 heading/list/link/inline formatting

当前只完成了 spine 顺序；正文仍然主要是 text extraction，Markdown 结构不够好。P1 应复用或抽象 HTML renderer。

### IPYNB

保留：

- markdown cell
- code cell
- kernelspec language hint
- cell 顺序

默认丢弃：

- raw cell
- outputs
- base64 图片
- widget/html output

原因：outputs 经常体积大、噪声高、包含二进制或 HTML widget。后续可以提供 `--outputs text` 只保留短 stdout / text/plain。

### CSV / TSV

保留：

- header
- delimiter 识别
- 编码识别
- RFC 4180 quoted fields

CSV 默认走 `-m json`，输出 table-native schema，不走 Markdown 反解析。`-m md` 仅用于终端 peek 小表。

JSON 规则：

- 第 1 行视为 header。
- 保留 `source`、`format`、`delimiter`、A1-style `range`、`column_count`、`headers`、`rows`、`row_range`、`truncated`、`warnings`。
- 没有 sheet 概念，不输出 `sheet` / `workbook_sheets` / `title` / `header_row` / `preamble`。
- preview 最多 100 条数据行，超过则 `truncated: true` 并写入 warning。

Markdown 路径（niche，仅 `-m md`）后续可考虑和 XLSX 统一：

- 小表：GFM table。
- 大表：fenced CSV/TSV。
- 超大表：row cap + truncation marker。

CSV 不应默认做强类型推断，因为 CSV 本质是文本格式。

### HTML / URL

当前不是重点，但已有基础策略：

- 优先 `article`，其次 `main`，最后 `body`。
- 跳过 `script/style/nav/header/footer/aside`。
- heading/list/link/table 转 Markdown。

待补：

- `<pre>` / `<code>` fenced block
- blockquote
- image alt/caption
- nested list 缩进
- 更稳定的 readability 算法

## 安全与性能

ZIP/Office 安全是自动调用和批处理的前置条件。

已完成：

- DOCX/PPTX/EPUB 共用统一 ZIP 读取 helper。
- entry count cap。
- per-entry decompressed-size cap。
- compression-ratio cap。
- CLI stdout 默认总量上限 256 KiB；多个输入共享预算。
- Markdown 截断时追加显式 warning marker，并向 stderr 输出 warning。
- JSON 保持合法 envelope，顶层 `truncated: true` + `warnings[]`；优先移除末尾 rows/tables。
- skipped/error diagnostics 最多详细输出前 20 条，其余汇总，避免 stderr / CI log warning flood。
- 输入读取、ZIP archive 总解压量、提取结果和多输入保留结果共享默认 64 MiB 解析预算；支持 `--max-parse-bytes`，超限返回结构化错误。

待补：

- 操作系统级精确 RSS / address-space 硬限制；当前解析预算约束可控数据体积，不承诺覆盖第三方库所有短时分配。
- 更细粒度的用户可配置 ZIP limits。
- XLSX 依赖层的额外安全包裹评估。

URL 读取当前有 30 秒 timeout，并受共享解析预算约束。后续如果 URL 成为重点，需要补更明确的 redirect、content-type、host allow/deny、offline mode 策略；当前 roadmap 不主攻爬虫。

## 分发决策

当前不发布到 crates.io，因此：

- package name 保持 `pith`。
- binary name 保持 `pith`。
- Homebrew/GitHub Release 是预编译二进制主路径。
- `cargo install --git https://github.com/harrisonwang/pith` 是源码安装路径。
- 不把 `cargo binstall pith` 作为当前安装承诺。

如果未来决定发布 crates.io，再重新评估 package 名称所有权和 cargo-binstall metadata。

## Roadmap 状态

P0：

- 默认输出模式按 format 分派（XLSX/CSV → json，其他 → md）：已完成。
- CSV/XLSX table JSON v2（顶层 `usage` + 扁平 `tables[]` + `workbook_sheets`）：已完成。
- 表格收窄 flag（`--sheet` / `--rows` / `--columns` / `--limit` / `--offset`）：已完成。`--rows` 使用 Excel 行号（含两端），与 `--limit`/`--offset` 互斥；找不到的 sheet/columns 硬错并列出可用列表。
- PDF page boundary：已完成，每页输出 `## Page N`。
- 总输出上限：已完成，默认 256 KiB，支持 `--max-output-bytes`，Markdown/JSON 均有可检测 truncation 信号。
- 解析预算：已完成，默认 64 MiB，支持 `--max-parse-bytes`，覆盖输入、ZIP 总解压量、提取结果和多输入累计结果。
- ZIP 安全层：entry count、per-entry size、compression ratio 和 archive total decompressed cap 已完成；仍缺细粒度用户配置和 XLSX 依赖层评估。

P1：

- stdin/pipe：已完成（`-` 读 stdin，format 靠 magic-byte 或 `--format`）。
- `pith chunk`（仅文档型）：未实现。
- EPUB/HTML renderer 统一：未实现。

P2/P3：

- Markdown 大表降级（niche，仅 `-m md`）：未实现。
- Rust library API：已有第一版 facade 和 table JSON schema 类型；第二阶段先稳定 typed core contract，再无行为拆包并发布 PyO3 binding。
- 分发完善：Homebrew/GitHub Release 已作为主线，winget/apt 未做。
- 可选 OCR/VLM backend：长期方向，默认关闭。

明确不做：

- `pith inspect` 子命令——JSON 默认输出（metadata + preview + usage + workbook_sheets）已经覆盖 inspect 的全部价值。
- MCP server——shell + tool wrapper 已经够用；社区可以做 `pith-mcp` thin wrapper。
- 通用 block-oriented JSON——文档型用 Markdown 已经是 LLM-friendly 表示。

## 测试策略

测试不应只验证“输出不为空”。应覆盖：

- 默认模式分派：`pith data.xlsx` 默认 JSON、`pith file.pdf` 默认 Markdown、`-m` override 正确生效。
- 结构是否保留：heading/list/table/link/footnote。
- 噪声是否丢弃：script/style/nav/empty placeholders。
- 顺序是否正确：spine、slide number、sheet order。
- token 经济策略：preview cap、truncation warning。
- 错误是否清楚：坏 zip、空文件、坏 JSON、非表格格式 `-m json` 报错。
- CLI contract：help、version、多输入、glob、format override、mode override、错误退出码。
- JSON mode：验证 CSV/XLSX `pith-table-json-v2` schema——顶层 `usage`、`tables[]` 扁平、顶层 `truncated`、`workbook_sheets` 在 XLSX 出现、`delimiter` 在 CSV 出现、`headers` 是 object、`rows[]` 是 field→value 映射、`row_range.first/last`、truncation warnings、非表格格式的错误提示。
