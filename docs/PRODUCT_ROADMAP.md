# pith 产品规划与市场分析

> 离线、单二进制、CLI-first、LLM-friendly、可定位

## 核心判断

`pith` 不需要做 MCP server，也不需要把自己包装成 Agent framework 的一部分。

它更应该成为一个稳定、轻量、可脚本化的 CLI 工具：把内容转成 LLM 直接能读的形式。

LLM-friendly 表示按 **内容形态** 决定，不按消费者类型：

| 内容形态 | LLM-friendly 形式 | 谁会消费 |
| --- | --- | --- |
| **文档型**：PDF、DOCX、PPTX、EPUB、IPYNB、HTML、Markdown、text、code | Markdown | 顺序读取的人或 LLM |
| **表格型**：CSV、XLSX | JSON（headers + preview + range） | 程序、脚本、RAG pipeline、Agent tool、LLM 当 context 时 |

人想看 XLSX 原貌请用 Excel / WPS / Numbers。`pith` 不和它们竞争。

最重要的产品哲学：

| 坚持 | 不追 |
|------|------|
| 本地、离线、单二进制 | 云服务 |
| CLI-first，能被任何工具调用 | 重 UI / GUI |
| 文档型 → Markdown；表格型 → JSON（按内容形态自动分派） | Agent 平台 / MCP 协议依赖 |
| 结构清楚、token 经济 | 像素级还原 |
| 可定位、可追溯 | 全自动理解一切文档 |
| "Agent 友好"作为副产品 | 把"Agent 友好"做成独立功能 |

---

## 1. 事实核查表

| 项目 | 复核状态 | 对 pith 的意义 | 来源 |
|------|----------|----------------|------|
| MarkItDown | 约 123k stars；README 明确面向 LLM Markdown，支持 PDF/Office/HTML；PDF 结构、Markdown、编码类 issue 仍存在 | 市场验证“文档转 LLM 输入”需求很大，但 PDF 质量仍是机会点 | [repo](https://github.com/microsoft/markitdown), [#41](https://github.com/microsoft/markitdown/issues/41), [#206](https://github.com/microsoft/markitdown/issues/206), [#296](https://github.com/microsoft/markitdown/issues/296), [#1290](https://github.com/microsoft/markitdown/issues/1290) |
| Docling | 约 59.7k stars；支持高级 PDF、OCR、JSON、本地执行、复杂文档理解 | Docling 是“大而全/ML 文档智能”标杆，pith 不该正面复制，应做轻量确定性入口 | [repo](https://github.com/docling-project/docling), [docs](https://docling-project.github.io/docling/) |
| Docling 轻量安装痛点 | #2481 CPU-only 已关闭；#2393 lightweight installation 仍开放 | 说明“轻依赖、单二进制”是真需求 | [#2481](https://github.com/docling-project/docling/issues/2481), [#2393](https://github.com/docling-project/docling/issues/2393) |
| Marker | 约 35.1k stars；GPL-3.0；模型商业限制；每 worker 峰值约 5GB VRAM | Marker 强在高精度 PDF/OCR，pith 应避免进入重模型战场 | [repo](https://github.com/datalab-to/marker) |
| anytomd-rs | 36 stars；Apache-2.0；README 明确 PDF out of scope | pith 的机会是“anytomd-rs 的轻量 Rust 路线 + 实际覆盖 PDF” | [repo](https://github.com/developer0hye/anytomd-rs) |
| LlamaParse | 商业平台；Free 10K credits，Starter $50，Pro $500，1,000 credits = $1.25；支持 JSON、bounding boxes、citations | 证明“高质量解析/可引用”有人付费，但 pith 的机会是离线低成本 | [pricing](https://www.llamaindex.ai/pricing) |
| Jina Reader | 约 10.8k stars；核心是 URL/read/search，支持 token budget、markdown chunking | Web/URL 方向已有强工具，pith 不应主攻爬虫 | [repo](https://github.com/jina-ai/reader) |
| Firecrawl | 约 120k stars；AGPL-3.0；主战场是 search/scrape/crawl | pith 不该做 Firecrawl 式爬虫 SaaS | [repo](https://github.com/firecrawl/firecrawl) |

---

## 2. 市场分层表

| 层级 | 本质问题 | 代表工具 | 用户真实想要 | pith 应站的位置 | 来源 |
|------|----------|----------|--------------|-----------------|------|
| L1 文本化 | 把字捞出来 | pdftotext、pdfminer、textract | 能跑、不崩、输出不空 | pith 已超过这个层级，不应退化成纯 text dump | [Poppler pdftotext](https://poppler.freedesktop.org/), [pdfminer.six](https://github.com/pdfminer/pdfminer.six), [textract](https://github.com/deanmalmgren/textract) |
| L2 结构化 | 保标题、列表、表格、sheet、slide | MarkItDown、Pandoc、Docling、Marker、anytomd-rs | LLM 能读懂结构 | pith 当前主战场 | [MarkItDown](https://github.com/microsoft/markitdown), [Pandoc](https://pandoc.org/), [Docling](https://github.com/docling-project/docling), [Marker](https://github.com/datalab-to/marker), [anytomd-rs](https://github.com/developer0hye/anytomd-rs) |
| L3 可定位/可追溯 | page、slide、sheet、row range、block anchor | LlamaParse、Docling、OpenAI File Search、Claude citations | LLM 输出能回到原文位置 | pith 下一阶段的差异化 | [LlamaParse pricing](https://www.llamaindex.ai/pricing), [Docling](https://docling-project.github.io/docling/), [OpenAI File Search](https://developers.openai.com/api/docs/guides/tools-file-search), [Claude citations](https://platform.claude.com/docs/en/build-with-claude/citations) |
| L4 智能理解 | OCR、图表理解、复杂表格、VLM 修复 | Marker、Docling、Mistral OCR、Azure Document Intelligence | 扫描件/图表也能读 | pith 可做可选插件，不应默认主线 | [Marker](https://github.com/datalab-to/marker), [Docling](https://github.com/docling-project/docling), [Mistral OCR](https://docs.mistral.ai/capabilities/OCR/basic_ocr/), [Azure Document Intelligence](https://learn.microsoft.com/en-us/azure/ai-services/document-intelligence/) |

---

## 3. 痛点表

| 痛点 | 真实表现 | 事实支撑 | pith 应怎么解 | 来源 |
|------|----------|----------|---------------|------|
| PDF 转 Markdown 经常只剩 raw text | 标题、表格、页边界、阅读顺序丢失 | MarkItDown #41 要求保 PDF tables/titles；#206 反馈 extraction is not in markdown | P0 做 `## Page N`、页眉页脚去重、断词修复、基础 page anchor | [MarkItDown #41](https://github.com/microsoft/markitdown/issues/41), [MarkItDown #206](https://github.com/microsoft/markitdown/issues/206) |
| Python/PyTorch 依赖重 | CI、内网、macOS、无 GPU 环境部署麻烦 | anytomd-rs README 把 Python runtime/dependency hell 当核心卖点；Docling 有 lightweight/CPU-only issue | 保持 Rust 单二进制，不默认引入模型/OCR | [anytomd-rs](https://github.com/developer0hye/anytomd-rs), [Docling #2393](https://github.com/docling-project/docling/issues/2393), [Docling #2481](https://github.com/docling-project/docling/issues/2481) |
| LLM context 被污染 | 一份长 PDF/Excel 直接塞上下文，token 浪费且模型找不到中间证据 | Lost in the Middle 论文说明长上下文中间信息利用不稳；OpenAI File Search 支持结果返回和 metadata filtering | 做 `pith chunk`、token budget、warnings、source anchors | [Lost in the Middle](https://arxiv.org/abs/2307.03172), [OpenAI File Search](https://developers.openai.com/api/docs/guides/tools-file-search) |
| 大表打爆 token | XLSX/CSV 转 GFM table，几千行直接不可用 | pith README 已列“大表策略待补”；Jina Reader 已提供 token budget/chunking 类控制 | 小表 GFM，中表 fenced TSV，超大表摘要 + row/col range | [pith README](../README.md), [Jina Reader](https://github.com/jina-ai/reader) |
| SaaS 不能处理敏感文件 | 合规、投研、法务、银行内网不能上传 | LlamaParse 是商业平台；Docling 强调 local/air-gapped | pith 明确主打本地、离线、可审计 | [LlamaParse pricing](https://www.llamaindex.ai/pricing), [Docling](https://github.com/docling-project/docling) |
| ZIP/Office 安全风险 | DOCX/XLSX/PPTX/EPUB 都是 ZIP 容器，可能有 zip bomb | OWASP 文件上传指南明确提 ZIP/XML bombs、parser exploit、解压后大小限制 | 统一 ZIP 防御层：entry cap、ratio cap、total cap | [OWASP File Upload](https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html) |

---

## 4. 爽点表

| 爽点 | 用户感受 | 已验证信号 | 产品化方向 | 来源 |
|------|----------|------------|-----------|------|
| 一行命令 | `pith report.pdf \| llm "总结风险"`；`pith data.xlsx \| jq ...` | MarkItDown、anytomd-rs、Firecrawl 都把 CLI/API 简洁作为入口 | 保持 CLI 极简，按内容形态自动分派 md/json | [MarkItDown](https://github.com/microsoft/markitdown), [anytomd-rs](https://github.com/developer0hye/anytomd-rs), [Firecrawl](https://github.com/firecrawl/firecrawl) |
| 单二进制 | 不装 Python、不拉模型、不配环境 | anytomd-rs 明确用 pure Rust/zero runtime 做卖点 | release 做好 Homebrew/cargo-binstall/winget | [anytomd-rs](https://github.com/developer0hye/anytomd-rs), [cargo-binstall](https://github.com/cargo-bins/cargo-binstall), [Homebrew](https://brew.sh/) |
| 离线隐私 | 敏感文件不出机器 | Docling、LlamaParse 都把敏感/企业/合规作为卖点 | 默认 no network；URL fetch 也要可控 | [Docling](https://github.com/docling-project/docling), [LlamaParse pricing](https://www.llamaindex.ai/pricing) |
| 可定位 | 回答能说“第 47 页，第 3 个表” | LlamaParse 列出 bounding boxes、citations、JSON；OpenAI File Search 支持返回搜索结果和 metadata filtering | 先做 table JSON + sheet/range/row anchors，再补 PDF page boundary | [LlamaParse pricing](https://www.llamaindex.ai/pricing), [OpenAI File Search](https://developers.openai.com/api/docs/guides/tools-file-search) |
| 低 token 噪声 | LLM 看到正文，不看字体/边距/装饰 | pith 工程决策已明确丢弃视觉样式、保语义结构 | JSON 默认 metadata + preview，不全量 dump；`truncated` + `warnings` 自描述 | [pith engineering decisions](ENGINEERING_DECISIONS.md) |
| CLI 易集成 | Claude Code、Codex、Cursor、shell、CI 都能直接调 | MarkItDown、anytomd-rs、Firecrawl 都提供 CLI/API 使用方式 | 继续把 CLI 作为一等入口，不做 MCP 依赖 | [MarkItDown CLI](https://github.com/microsoft/markitdown), [anytomd-rs CLI](https://github.com/developer0hye/anytomd-rs), [Firecrawl CLI](https://github.com/firecrawl/firecrawl) |

---

## 5. 人群与场景表

| 人群 | 场景故事 | 他们的痛 | pith 的承诺 | 来源 |
|------|----------|----------|------------|------|
| Claude Code / Codex / Cursor 用户 | 工程师收到 38 页 RFP.docx，要让 coding assistant 读取需求并辅助改接口 | 不想装 Docling，不想上传 SaaS，不想手动转格式 | `pith RFP.docx` 输出干净 Markdown；`pith data.xlsx -m json` 输出表格结构 | [Claude Code docs](https://docs.anthropic.com/en/docs/claude-code/overview), [Codex docs](https://developers.openai.com/codex/), [Cursor](https://cursor.com/) |
| RAG/知识库工程师 | 要把 2,000 份 PDF/DOCX/PPTX 进向量库 | chunk 粗糙、引用不准、噪声多 | 先做好 heading/page-aware Markdown 和 `pith chunk`，不先承诺全格式 JSON | [OpenAI File Search](https://developers.openai.com/api/docs/guides/tools-file-search), [LlamaIndex](https://www.llamaindex.ai/) |
| 投研/财务分析 | 处理 10-K、财报 PDF、Excel sheet | 大表爆 token，合规禁止上传 | 本地解析，表格降级，page/sheet/row anchors | [SEC EDGAR](https://www.sec.gov/edgar), [LlamaParse pricing](https://www.llamaindex.ai/pricing) |
| 法务/合规 | 审合同、批注、脚注、修订记录 | 需要证据链，不需要漂亮排版 | DOCX comments/endnotes/tracked changes mode | [Open XML SDK](https://learn.microsoft.com/en-us/office/open-xml/open-xml-sdk), [pith DOCX matrix](test-matrix/docx.md) |
| SRE/文档 CI | release notes、docs pipeline 自动转换 | Python 依赖慢、CI cache 麻烦 | 静态 CLI，可 pin 版本，输出稳定 | [GitHub Actions](https://docs.github.com/en/actions), [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) |
| 研究生/个人知识工作流 | `pith paper.pdf \| llm "总结 methods"` | 不想记一堆工具 | 一个命令覆盖常见格式 | [llm CLI](https://github.com/simonw/llm), [arXiv](https://arxiv.org/) |

---

## 6. 路线图表

### P0 - 必须做

| 功能 | 解决什么真实问题 | 验收标准 | 来源 |
|------|------------------|----------|------|
| 默认输出模式按 format 分派 | 用户不该为"XLSX 该用 md 还是 json"做决策；定位就是表格型 → JSON、文档型 → Markdown | `pith data.xlsx` 默认 JSON；`pith file.pdf` 默认 Markdown；`-m` 仅作显式覆盖；非表格格式 `-m json` 报错并提示 | [pith README](../README.md) |
| CSV/XLSX table JSON v2 | 大表和 spreadsheet 需要程序化字段，且 JSON 自身要自描述 | 顶层 `schema_version` + `usage` + `tables[]` + `truncated` + `warnings[]`；table 内含 `source/format/sheet/workbook_sheets/title/range/column_count/header_row/headers/preamble/rows/row_range/truncated/warnings` | [pith README](../README.md), [OpenAI Structured Outputs](https://developers.openai.com/api/docs/guides/structured-outputs) |
| 表格收窄 flag | LLM/脚本看到 `truncated: true` 时需要能精确切片；HATEOAS-style：从 JSON 的 row_range + workbook_sheets 直接复制 flag 参数 | `--sheet <name>`、`--rows <first:last>`（Excel 行号）、`--columns <a,b,c>`、`--limit <n>`、`--offset <n>`；`--rows` 与 `--limit`/`--offset` 互斥；找不到的 sheet/columns 硬错并列出可用列表；JSON 顶层 `usage` 字符串描述真实 flag | [pith XLSX matrix](test-matrix/xlsx.md), [pith README](../README.md) |
| 总输出上限 ✅ | 防止单个大文档或 glob 撑爆 Agent、shell、CI 和模型上下文 | 整次 stdout 默认 256 KiB；Markdown 有 marker + stderr warning；JSON 保持合法并设置顶层 `truncated` | [pith README](../README.md) |
| 解析预算 ✅ | 防止巨大输入、容器解压内容和提取结果在 stdout 截断前消耗无界内存 | 默认共享 64 MiB；支持 `--max-parse-bytes`；超限返回可机器识别的结构化错误；明确不承诺操作系统级精确 RSS 硬限制 | [pith README](../README.md), [pith adversarial matrix](test-matrix/adversarial.md) |
| PDF page boundary | LLM 回答无法回到页码 | 多页 PDF 输出 `## Page N` 或 JSON page anchor | [pith PDF matrix](test-matrix/pdf.md), [MarkItDown #41](https://github.com/microsoft/markitdown/issues/41) |
| ZIP 安全层 | Office/EPUB 自动处理有 zip bomb 风险 | DOCX/XLSX/PPTX/EPUB 共用 entry cap、ratio cap、total cap | [OWASP File Upload](https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html), [pith adversarial matrix](test-matrix/adversarial.md) |

### P1 - 重要

| 功能 | 解决什么真实问题 | 验收标准 | 来源 |
|------|------------------|----------|------|
| stdin/pipe ✅ | shell 一等公民 | `cat file.csv \| pith --format csv -`（`-` 读 stdin，已完成） | [MarkItDown CLI](https://github.com/microsoft/markitdown), [anytomd-rs CLI](https://github.com/developer0hye/anytomd-rs) |
| `pith chunk`（仅文档型） | RAG/LLM 不能只按固定长度切 | 按 heading/page/slide/chapter 分块；表格型不在此处理 | [OpenAI File Search](https://developers.openai.com/api/docs/guides/tools-file-search), [Lost in the Middle](https://arxiv.org/abs/2307.03172) |
| EPUB/HTML renderer 统一 | EPUB 正文结构弱 | EPUB chapter 内 heading/list/link/table 正常保留 | [pith EPUB matrix](test-matrix/epub.md), [pith HTML matrix](test-matrix/html.md) |

### P2 - 优化

| 功能 | 解决什么真实问题 | 验收标准 | 来源 |
|------|------------------|----------|------|
| Markdown 大表降级（niche） | `-m md` 处理大表的终端 peek 场景；主路径走 JSON 后此条优先级下降 | 小表 GFM；中表 fenced TSV；超大表摘要 + range + truncation | [pith XLSX matrix](test-matrix/xlsx.md), [Jina Reader token budget](https://github.com/jina-ai/reader) |
| pith-core + PyO3 binding | Rust/Python 编排器内直连，减少高频子进程开销并扩大 Agent 开发者覆盖面 | typed core result/error；CLI parity 不变；`parse_bytes` / `parse_path` 释放 GIL；用 benchmark 证明小文件高频调用收益 | [Core/Python architecture](CORE_PYTHON_ARCHITECTURE.md), [crates.io](https://crates.io/) |
| 分发完善 | 降低安装摩擦 | Homebrew、cargo-binstall、winget、apt | [Homebrew](https://brew.sh/), [cargo-binstall](https://github.com/cargo-bins/cargo-binstall), [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/) |

### P3 - 长期

| 功能 | 解决什么真实问题 | 验收标准 | 来源 |
|------|------------------|----------|------|
| 可选 OCR/VLM backend | 扫描件和图表 | 默认关闭，按页 fallback，保留成本/warning | [Marker](https://github.com/datalab-to/marker), [Mistral OCR](https://docs.mistral.ai/capabilities/OCR/basic_ocr/), [Azure Document Intelligence](https://learn.microsoft.com/en-us/azure/ai-services/document-intelligence/) |

---

## 7. 做与不做

| 方向 | 判断 | 原因 | 来源 |
|------|------|------|------|
| 默认输出按内容形态分派 | 必做 | 文档型 → Markdown、表格型 → JSON 是 pith 的核心定位；让用户/Agent 选 `-m` 是把内部决策外泄 | [pith README](../README.md) |
| 选择性 JSON | 必做，且仅对表格型 | CSV/XLSX 的 LLM-friendly 表示就是 JSON；PDF/DOCX/PPTX/IPYNB 是文档型，硬塞 JSON 会破坏顺序读取的语义，也是 Codex block-oriented JSON 那次尝试失败的根本原因 | [pith README](../README.md), [OpenAI File Search](https://developers.openai.com/api/docs/guides/tools-file-search) |
| JSON 自描述（usage + workbook_sheets） | 必做 | 开源 Coding Agent / 自建 Agent 直接 shell 调 pith 时，JSON 必须告诉消费者下一步怎么收窄；不能强依赖 `--help` 或外部 wrapper | [pith README](../README.md) |
| PDF 基础结构 | 必做 | PDF 是最痛格式，anytomd-rs 放弃了，pith 可形成差异 | [anytomd-rs](https://github.com/developer0hye/anytomd-rs), [MarkItDown #41](https://github.com/microsoft/markitdown/issues/41) |
| 表格收窄 flag | 必做 | 默认 preview 之后必须能精确切片，否则 `truncated: true` 是死路 | [pith XLSX matrix](test-matrix/xlsx.md) |
| ZIP 防御 | 必做 | 自动调用和批处理前置条件 | [OWASP File Upload](https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html) |
| Markdown 大表降级 | 降级到 P2 | 表格型主路径改走 JSON 后，`-m md` 处理大表只是 niche 终端 peek，不再是 P0 | [pith XLSX matrix](test-matrix/xlsx.md), [Jina Reader](https://github.com/jina-ai/reader) |
| OCR | 暂不做默认 | 会引入模型、依赖、速度和维护复杂度；可做插件 | [Marker](https://github.com/datalab-to/marker), [Mistral OCR](https://docs.mistral.ai/capabilities/OCR/basic_ocr/) |
| LLM 增强 | 暂不做默认 | 破坏离线、确定性、低成本定位 | [MarkItDown OCR plugin](https://github.com/microsoft/markitdown), [Marker LLM mode](https://github.com/datalab-to/marker) |
| 云服务 | 不优先 | Firecrawl/LlamaParse/Jina 已很强，个人项目不该正面卷 | [Firecrawl](https://github.com/firecrawl/firecrawl), [LlamaParse](https://www.llamaindex.ai/pricing), [Jina Reader](https://github.com/jina-ai/reader) |
| GUI | 不优先 | 目标人群在 CLI、CI、LLM 工具链里 | [MarkItDown CLI](https://github.com/microsoft/markitdown), [anytomd-rs CLI](https://github.com/developer0hye/anytomd-rs) |
| 格式互转 | 不做 | Pandoc/LibreOffice 地盘，和 pith 核心目标偏离 | [Pandoc](https://pandoc.org/), [LibreOffice](https://www.libreoffice.org/) |
| MCP server | 不做 | shell + tool wrapper 已经够用；MCP 会增加协议兼容维护面；社区可以做 `pith-mcp` thin wrapper | [Claude Code docs](https://docs.anthropic.com/en/docs/claude-code/overview), [Codex docs](https://developers.openai.com/codex/) |
| `pith inspect` 子命令 | 不做 | JSON 默认输出（metadata + preview + usage + workbook_sheets）已经覆盖 inspect 的全部价值，加子命令是冗余 | [pith README](../README.md) |
| 通用 block-oriented JSON | 不做 | 文档型用 Markdown 已经是 LLM-friendly 表示；JSON 化 prose 既贵又破坏顺序读取语义 | [pith README](../README.md) |

---

## 8. 最终定位表

| 维度 | pith 应该坚持 | 不该追求 | 来源 |
|------|---------------|----------|------|
| 产品一句话 | LLM 时代的离线文档预处理 CLI | Agent 平台 / 文档智能 SaaS | [MarkItDown](https://github.com/microsoft/markitdown), [Docling](https://github.com/docling-project/docling) |
| 默认形态 | Rust CLI + library | Python ML 框架 / SaaS / MCP server | [anytomd-rs](https://github.com/developer0hye/anytomd-rs), [LlamaParse](https://www.llamaindex.ai/pricing) |
| 输出核心 | 文档型 → Markdown；表格型 → JSON（自动按 format 分派） | 像素级还原 / 单一万能 schema | [pith README](../README.md), [Pandoc](https://pandoc.org/) |
| 决策外泄 | `-m` 仅作显式覆盖；默认值由 pith 替用户决定 | 让用户 / Agent 在每次调用时选 md 还是 json | [pith README](../README.md) |
| Agent 集成 | tool wrapper snippet + AGENTS.md snippet（零代码改动） | MCP server / inspect 子命令 / 内嵌 Agent 框架 | [pith README](../README.md) |
| 质量标准 | 结构清楚、token 经济、可定位、可审计、JSON 自描述 | 人眼排版漂亮 | [pith engineering decisions](ENGINEERING_DECISIONS.md), [Lost in the Middle](https://arxiv.org/abs/2307.03172) |
| 目标格式 | PDF/DOCX/XLSX/PPTX/EPUB/IPYNB/CSV/HTML/text | 图片 OCR/手写/复杂扫描件优先 | [pith README](../README.md), [Marker](https://github.com/datalab-to/marker) |
| 护城河 | 单二进制 + 离线 + 按内容形态自动分派的 LLM-friendly 表示 | 模型精度、OCR 榜单、爬虫规模 | [anytomd-rs](https://github.com/developer0hye/anytomd-rs), [Firecrawl](https://github.com/firecrawl/firecrawl), [Docling](https://github.com/docling-project/docling) |

---

## 执行优先级

建议顺序：

1. 默认输出模式按 format 分派（XLSX/CSV → json，其他 → md）✅ 已完成
2. CSV/XLSX table JSON v2（顶层 `usage` + `tables[].workbook_sheets` 等自描述字段）✅ 已完成
3. 表格收窄 flag（`--sheet` / `--rows` / `--columns` / `--limit` / `--offset`），JSON `usage` 字符串同步描述 ✅ 已完成
4. PDF page boundary ✅ 已完成
5. ZIP 安全层补完
6. stdin/pipe ✅ 已完成
7. `pith chunk`（仅文档型）
8. EPUB/HTML renderer 统一
9. Markdown 大表降级（niche，仅 `-m md` 路径）

这套顺序更符合现在的产品定位：

不是“Agent 能信任的文件入口”，而是 **“按内容形态自动输出 LLM-friendly 表示、开发者能稳定脚本化调用的离线文档预处理器”**。
