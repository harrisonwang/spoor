# 工程决策

本文档记录 `spoor` 的产品边界和提取契约。核心定位是：

> 离线、单二进制、CLI-first、LLM-friendly。

`spoor` 不是为了复刻 `extract-text` 的逐字输出，也不是 Docling/Marker/LlamaParse 这类重型文档智能系统的替代品。它应该成为稳定、轻量、可脚本化的本地 CLI：**按内容形态自动输出 LLM-friendly 表示**——文档型（PDF / DOCX / PPTX / EPUB / IPYNB / HTML / Markdown / text / code）输出 Markdown，表格型（CSV / XLSX）输出 schema + preview 的 JSON。

这个 LLM-friendly 表示是按内容形态决定的，不按消费者类型决定。同一份 5000 行 XLSX 给 LLM 看 Markdown 表格是灾难（token 爆炸、丢表头）；给它 `{headers, preview_rows, row_count, range}` 则是常数 token + 一眼能看懂的形态。这就是为什么表格型默认走 JSON 而不是 Markdown，也是为什么文档型默认走 Markdown 而不是 JSON——把 prose 切成 block 装进 JSON，既浪费 token 又破坏顺序阅读的语义。

## 产品边界

坚持：

- 本地、离线、单二进制。
- CLI-first，能被 shell、CI、Claude Code、Codex、Cursor、自建 Agent 等直接调用。
- 默认输出按格式分派：文档型 → Markdown，表格型 → JSON；`-m` 仅作显式覆盖。
- JSON 自描述：包含 `usage` 字符串和 `workbook_sheets` 结构信息，让消费者不依赖外部 `--help` 也能决定下一步。
- 优先结构清楚、token 经济、行为可预期、可审计。

不追：

- 云服务、GUI、Agent 平台、MCP server、`spoor inspect` 子命令。
- 像素级排版还原、通用格式互转、通用 block-oriented JSON。
- 默认 OCR、VLM、LLM 增强或重模型依赖。
- Firecrawl/Jina Reader 式爬虫能力。
- 和 Excel / WPS / Numbers 竞争"人看表格原貌"这件事。

## 判断标准

"转成某种格式"不自动等于 LLM 友好。输出是否合格按下面标准判断：

1. **形态对路**：文档型走 Markdown，表格型走 JSON；不应让 LLM 读 5000 行的 Markdown 表格，也不应让 LLM 读 prose 切 block 的 JSON。
2. **结构保真**：标题、段落、列表、表格、链接、脚注、页、sheet、slide、chapter 等结构不能轻易丢。
3. **阅读顺序正确**：DOCX 段落顺序、PPTX slide 顺序、EPUB spine 顺序、PDF page 顺序要可预期。
4. **噪声低**：脚本、样式、导航、广告、装饰图形、空占位符、重复页眉页脚默认不进入正文。
5. **token 经济**：表格型默认走 JSON preview，常数 token；文档型 Markdown 丢弃视觉样式只留语义结构。
6. **JSON 自描述**：表格 JSON 含 `usage` 字符串和 `workbook_sheets`，`truncated` + `warnings` 告诉消费者还有多少没读、怎么继续读。
7. **转换性能可控**：不能为了格式还原引入过重依赖或不受控内存占用。
8. **恶意输入安全**：ZIP 类格式必须有 entry cap、压缩比限制、单 entry 解压大小限制和总输出限制。

## CLI 契约

当前 CLI 是一等入口。`spoor -h` 应保持短、稳定、适合复制到 README：

```text
离线、单二进制、CLI-first 的 LLM-friendly 文档预处理工具

Usage:
  spoor [OPTIONS] <input>...

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

  spoor file.xlsx                              # see structure + preview
  spoor file.xlsx --sheet L1 --rows 5:104      # read a slice
  spoor file.xlsx --columns 分类,技能          # filter columns

spoor bounds JSON previews by default (first 100 data rows per table) and
caps total CLI output at 256 KiB. Use --limit/--rows to narrow tables or
--max-output-bytes to raise the total output cap. Parsing uses a shared
64 MiB data-volume budget by default; raise it with --max-parse-bytes.

Examples:
  spoor report.pdf
  spoor data.xlsx
  spoor data.csv | jq '.tables[]'
  cat data.csv | spoor --format csv -
  spoor https://example.com/article
  spoor "*.pdf"
  spoor report.pdf | llm "Summarize risks and action items"
```

CLI 行为约定：

- 默认输出模式由 effective format（含 `--format` override 后）决定：CSV/XLSX → `json`，其他 → `md`。
- `-m` 仅作为显式覆盖；表格型可显式选 `-m md`（终端快速查看小表，大表不保证可用）；非表格格式 `-m json` 返回错误并提示使用 `-m md`。
- `--mode json` 输出 `spoor-table-json-v2`，schema 详见下方"输出模式 / json"。
- 多输入在 Markdown 模式下按 `# Source: ...` 分块；JSON 模式下所有表合并到顶层 `tables[]` 数组。
- 本地 glob 由程序内部展开；URL 不参与 glob。
- `--format` 只覆盖文件/URL 内容格式，不定义 URL 抓取策略。
- stdin/pipe 已实现：输入 `-` 读取标准输入；无路径无扩展名，format 走 magic-byte 检测或 `--format` override（表格型从 stdin 需显式 `--format csv`）。`-` 不参与 glob，可与文件混用。
- 表格筛选参数已实现：`--sheet`（仅 XLSX；CSV 无 sheet 概念，自动忽略）、`--rows <first:last>`（Excel 行号，含两端）、`--columns <a,b,c>`、`--limit <n>`、`--offset <n>`。`--rows` 与 `--limit`/`--offset` 互斥（clap conflicts_with）。找不到的 sheet/columns 直接报错并列出可用列表。
- `--extract <uri>` 是格式无关的单资源二进制输出入口；资源 URI scheme 标识格式并由 core 分派。当前仅支持正文中实际输出的安全 `spoor-docx://word/media/*` URI，后续格式不新增专属 CLI flag。

## 库契约

当前采用 Cargo workspace。`spoor-core` 的正式 API（以 `crates/spoor-core/src/lib.rs` 的 re-export 为准）：

- 请求与限制：`ParseRequest`、`ParseLimits`、`TableFilter`（`TableFilter::build` 是
  跨宿主共用的筛选校验与组装入口）
- 检测与解析：`detect_format`、`parse`、`parse_document_result`、`parse_document`、`parse_tables`
- 内嵌媒体：`extract_media`；格式无关入口，当前支持安全 `spoor-docx://` 与 `spoor-pdf://` URI
- 类型化结果：`ParseResult`、`ParseContent`、`DocumentResult`、`TableResult`、`ParseStats`、`SpoorWarning`、`WarningCode`、`WarningLocation`
- 类型化错误：`SpoorError`、`ErrorCode`、`ParseStage`
- format / mode：`Format`、`OutputMode`、`default_mode_for`
- table JSON schema 类型：`JsonOutput`、`HeaderInfo`、`PreambleInfo`、`RowRange`、`TABLE_SCHEMA_VERSION`、`TABLE_USAGE`、`a1_range`、`cells_to_values`

解析器细节保持在 `parse/` 内部，不作为公共 API。文件、URL、stdin、glob
和进程退出仅存在于 `spoor-cli`；Python 的 `parse_path` 也是绑定层的便捷函数。
公共边界不暴露 `anyhow::Result`。

Python、Node 与 WASM 绑定都通过 `TableFilter::build` 暴露表格筛选
（`sheet`/`rows`/`columns`/`limit`/`offset`，`rows` 为含两端的 1-based 行号且与
`limit`/`offset` 互斥），并都提供 `extract_media`。表格筛选与媒体提取的校验、
分页与错误码在 CLI、Python、Node、WASM 四个宿主等价——这是 spoor「同一引擎、
跨宿主等价信号」契约的一部分。

`parse` 和 `parse_document_result` 保留结构化完整性 warnings，供 Agent 按稳定
code 与 `location.kind=page/slide` 分支。`parse_document` 仅用于明确不需要诊断的
Markdown 兼容调用。CLI Markdown 模式把文档 warning 同时写入 stderr 和 stdout
尾部，避免 Agent 只消费 stdout 时错过降级信息。

## 输出模式

### `md`

文档型的默认模式。面向模型上下文，优先保留语义结构，丢弃纯视觉信息。

当前策略：

- 文本文档输出 Markdown-like 文本。
- 小表输出 GFM table。
- notebook 输出 markdown cells 和 code cells，默认不输出 cell outputs。
- slide/sheet/chapter 使用明确标题分块。

表格型（CSV/XLSX）使用 `-m md` 是显式 override，仅用于终端快速查看小表；大表行为不保证。

后续策略：

- Markdown 大表降级（小表 GFM / 中表 fenced TSV / 超大表摘要+range+truncation，仅限特定场景下的 `-m md`，P2）。

### `json`

表格型（CSV/XLSX）的默认模式。schema 为 `spoor-table-json-v2`，自描述（含 `usage` 字符串），扁平 `tables[]`。

非表格格式使用 `-m json` 返回错误并提示使用 `-m md`。**不**为 DOCX/PDF/PPTX/IPYNB 设计通用 block-oriented JSON——把 prose 切成 block 装进 JSON，既浪费 token 又破坏顺序阅读语义，是错误方向。

```json
{
  "schema_version": "spoor-table-json-v2",
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

Schema 字段说明：

- 顶层：`schema_version`、`usage`、`tables[]`、`truncated`、`warnings[]`
- table 必有：`source`、`format`、`range`、`column_count`、`headers`、`rows`、`row_range`、`truncated`、`warnings`
- table 表格型条件字段：
  - XLSX：`sheet`、`workbook_sheets`、`title`、`header_row`、`preamble`
  - CSV：`delimiter`

Schema 设计理由：

| 字段 | 设计理由 |
|------|----------|
| 顶层 `usage` | 一行字符串告诉消费者如何缩小范围；让 JSON 自描述，不依赖外部 `--help` 或 wrapper。`usage` 是单一发布常量 `TABLE_USAGE`；测试 `table_usage_lists_every_narrowing_flag` 从 clap 定义派生筛选选项并断言每个都出现在 `TABLE_USAGE` 里——某个选项改名或新增却忘了同步会直接挂 CI，避免文档与实际行为悄悄偏离。 |
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
- 内嵌栅格图片在正文中的安全 `spoor-docx://word/media/*` 占位符

默认丢弃：

- 字体、字号、颜色、边距、对齐
- 图片语义、纯装饰图形
- tracked changes 的删除内容
- Word 内部样式细节

待补：

- comments/endnotes
- image alt/caption
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
- 装饰图形
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

纯图片 PDF 没有 text layer 时返回非零退出码，并在 stderr 输出可解析的结构化错误：`code: "image_only_pdf"`、中文 reason/hint（明确提示需要 OCR）、`recoverable: true`。受密码保护的 PDF 同理返回 `code: "encrypted_pdf"`、`recoverable: false`。结构化错误格式为 `{is_error, code, reason, hint, recoverable}`，消费者按 `code` 分支；`reason`/`hint` 是中文展示文本，不构成契约。全部稳定 code 见 README「结构化错误与 code」一节。

### EPUB

保留：

- OPF spine 顺序
- chapter boundary
- HTML 内部 heading/list/link/inline formatting

正文已经复用 HTML Markdown renderer，可保留基础 heading/list/link/inline
formatting。仍缺固定版式、图片/音视频、复杂导航、CSS 布局以及更完整的 HTML
语义节点支持。

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

CSV 默认走 `-m json`，输出 table-native schema，不走 Markdown 反解析。`-m md` 仅用于终端快速查看小表。

JSON 规则：

- 第 1 行视为 header。
- 保留 `source`、`format`、`delimiter`、A1-style `range`、`column_count`、`headers`、`rows`、`row_range`、`truncated`、`warnings`。
- 没有 sheet 概念，不输出 `sheet` / `workbook_sheets` / `title` / `header_row` / `preamble`。
- preview 最多 100 条数据行，超过则 `truncated: true` 并写入 warning。

Markdown 路径（特定场景，仅 `-m md`）后续可考虑和 XLSX 统一：

- 小表：GFM table。
- 大表：fenced CSV/TSV。
- 超大表：row cap + truncation marker。

CSV 不应默认做强类型推断，因为 CSV 本质是文本格式。

### HTML / URL

目标场景是 `spoor https://…` 的 URL 抓取；本地 `.html` 文件是边角，不为它单独设计。已有策略：

- 优先 `article`，其次 `main`，最后 `body`。
- 跳过 `script/style/nav/header/footer/aside`。
- heading/list/table/blockquote/pre/code 转 Markdown。
- `<img>` 转标准 Markdown 图片 `![alt](src)`，Agent 可直接把 URL 交给外部 VLM；`data:` URI 或无 src 的图片退回 `[图片：alt]` 占位符，避免把 base64 灌进上下文。
- `<a href>` / `<img src>` 的相对地址按页面 URL 做 best-effort 绝对化：`source_name` 是 http(s) URL 时解析相对路径（`/abs`、`../rel`、`?query`），已是绝对地址、协议相对 `//`、`data:`/`mailto:` 等 scheme、`#fragment` 一律不动。绝对化是纯字符串运算（不引入 `url` crate、不联网），CLI / 库 / WASM 行为一致；没有 http(s) base（本地文件、stdin、未带 URL 的字节调用）时链接保持原样。

待补：

- image caption
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
- JSON 保持合法结构，顶层 `truncated: true` + `warnings[]`；优先移除末尾 rows/tables。
- skipped/error diagnostics 最多详细输出前 20 条，其余汇总，避免 stderr / CI log warning flood。
- 输入读取、ZIP archive 总解压量、提取结果和多输入保留结果共享默认 64 MiB 数据量预算；支持 `--max-parse-bytes`，超限返回结构化错误。

待补：

- 操作系统级精确 RSS / address-space 严格限制；当前数据量预算约束可控数据体积，不承诺覆盖第三方库所有短时分配。
- 更细粒度的用户可配置 ZIP limits。
- XLSX 依赖层的额外安全包裹评估。

URL 读取当前有 30 秒 timeout，并受共享数据量预算约束。后续如果 URL 成为重点，需要补更明确的 redirect、content-type、host allow/deny、offline mode 策略；当前路线图不主攻爬虫。

## 分发决策

当前分发约定：

- crates.io package 名为 `spoor-core` 与 `spoor-cli`，由 release workflow 按依赖顺序发布。
- binary name 保持 `spoor`。
- Homebrew/GitHub Release 是预编译二进制主路径。
- `cargo install spoor-cli` 是源码安装路径。
- 不把 `cargo binstall spoor` 作为当前安装承诺。

crates.io 发布由 release workflow 管理。

## 路线图状态

P0：

- 默认输出模式按 format 分派（XLSX/CSV → json，其他 → md）：已完成。
- CSV/XLSX table JSON v2（顶层 `usage` + 扁平 `tables[]` + `workbook_sheets`）：已完成。
- 表格筛选参数（`--sheet` / `--rows` / `--columns` / `--limit` / `--offset`）：已完成。`--rows` 使用 Excel 行号（含两端），与 `--limit`/`--offset` 互斥；找不到的 sheet/columns 直接报错并列出可用列表。
- PDF page boundary：已完成，每页输出 `## Page N`。
- 总输出上限：已完成，默认 256 KiB，支持 `--max-output-bytes`，Markdown/JSON 均有可检测 truncation 信号。
- 数据量预算：已完成，默认 64 MiB，支持 `--max-parse-bytes`，覆盖输入、ZIP 总解压量、提取结果和多输入累计结果。
- ZIP 安全层：entry count、per-entry size、compression ratio 和 archive total decompressed cap 已完成；仍缺细粒度用户配置和 XLSX 依赖层评估。

P1：

- stdin/pipe：已完成（`-` 读 stdin，format 靠 magic-byte 或 `--format`）。
- EPUB/HTML renderer 统一：已完成基础复用；更完整 HTML 语义节点仍按采用信号投入。

P2/P3：

- Markdown 大表降级（特定场景，仅 `-m md`）：未实现。
- Rust library API：类型化 core 契约、CLI、PyO3、napi-rs 与 WASM 已完成。
- 分发完善：crates.io、PyPI、npm、Homebrew、Scoop 与 GitHub Release 已有发布工作流；winget/apt 未做。
- 可选 OCR/VLM backend：长期方向，默认关闭。

明确不做：

- `spoor inspect` 子命令——JSON 默认输出（metadata + preview + usage + workbook_sheets）已经覆盖 inspect 的全部价值。
MCP server——通过 shell 配合工具封装已足够；社区可以做 `spoor-mcp` 轻量封装。
- 通用 block-oriented JSON——文档型用 Markdown 已经是 LLM-friendly 表示。
- `spoor chunk` / 文档分块——文档是顺序读介质，截断是尾部小概率问题：缩小输入或 `--max-output-bytes` 重跑即可。为它引入"清单 + 切片 + 汇总"整套机制，是把表格的随机访问思维套用到顺序读的文档上，已明确否决（2026-06）。

## 测试策略

测试不应只验证"输出不为空"。应覆盖：

- 默认模式分派：`spoor data.xlsx` 默认 JSON、`spoor file.pdf` 默认 Markdown、`-m` override 正确生效。
- 结构是否保留：heading/list/table/link/footnote。
- 噪声是否丢弃：script/style/nav/empty placeholders。
- 顺序是否正确：spine、slide number、sheet order。
- token 经济策略：preview cap、truncation warning。
- 错误是否清楚：坏 zip、空文件、坏 JSON、非表格格式 `-m json` 报错。
- CLI 契约：help、version、多输入、glob、format override、mode override、错误退出码。
- JSON mode：验证 CSV/XLSX `spoor-table-json-v2` schema——顶层 `usage`、`tables[]` 扁平、顶层 `truncated`、`workbook_sheets` 在 XLSX 出现、`delimiter` 在 CSV 出现、`headers` 是 object、`rows[]` 是 field→value 映射、`row_range.first/last`、truncation warnings、非表格格式的错误提示。
