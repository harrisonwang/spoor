# spoor 路线图（2026-06）

> 本文是 spoor 阶段的产品路线图，基于 2026-06-11 的联网调研，取代原单 CLI 阶段的产品路线图。定位与工程规划见 [定位与工程规划](../planning/overview.md)。

## 本轮交付状态

P0 与 P1 已全部完成：workspace/core/CLI 更名、类型化错误、PyO3、napi-rs、
WASM、`SECURITY.md`、发布工作流和场景 demo 均已实现。WASM release 实测：

| 形态 | Raw | gzip | 目标 |
| --- | ---: | ---: | ---: |
| `core-formats` | 1,429,029 B | 591,942 B | ≤ 3 MiB |
| `full`（默认发布，含 PDF/EPUB/IPYNB） | 2,234,561 B | 861,544 B | ≤ 3 MiB |

P2 仍根据采用信号投入，不在本轮预先扩展范围内；Lambda、Electron、
本地混合语料库与确定性 RAG / 搜索索引摄取示例已随本轮一并提供。

## 调研结论（一句话）

"Rust core + 多语言绑定 + WASM"这条路线，过去半年已经从无人区变成了战场——但目前没有人同时做到**体积小、纯 Rust、MIT 许可证、内建防御契约**这四点，这就是 spoor 的生态位。

## 竞品调研（核查日期 2026-06-11）

| 项目 | 状态 | 关键差异 | 竞争分析 |
| --- | --- | --- | --- |
| **kreuzberg**（约 8.5k★，Elastic-2.0，活跃） | Rust core 文档智能框架：90+ 格式、15 种语言绑定、CLI/REST/MCP/Docker；**有完整防御层**（SECURITY.md 威胁模型：ZipBombValidator 压缩比 100×/归档 500 MiB、内嵌文件 50 MiB、递归深度 3、60 s 提取超时、内容增长预算、XML 实体炸弹验证，均可配置） | WASM ✅（多格式+OCR）；Rust crate ✅；绑定 ✅。**WASM 实测**：主模块 gzip 后 8.2 MB + 捆绑的 PDFium 1.9 MB ≈ **10.1 MB**——超免费档 3 MB 三倍多，连付费档 10 MB 都刚好踩线；且浏览器端 PDF 走 **PDFium（C++ 编译为 WASM）**，"纯 Rust PDF"在 WASM 形态不成立 | **最强竞品，格式/绑定/防御全面覆盖**。弱项集中在三点：体积（wasm 10.1 MB gzip、wheel 27–34 MB、Docker 1–1.3 GB）、WASM 端非纯 Rust（依赖 PDFium）、Elastic-2.0 非 OSI 开源（限制 SaaS 托管，企业合规敏感） |
| **LiteParse**（LlamaIndex，约 9.9k★，Apache-2.0，活跃） | "fast and light"本地解析器，交付形态与 spoor 规划相同（Rust core + napi/PyO3/WASM/CLI） | WASM ✅ 但**仅 PDF**（官方说明 "PDF parsing — WebAssembly build"，4.1 MB）；PDF 依赖 PDFium（C 库），Office 靠 **LibreOffice 转换**，OCR 靠 Tesseract | 验证了四形态打法的可行性，但架构依赖重型原生组件，浏览器端 Office 解析无法实现；输出是 bbox JSON 而非 Markdown-first；产品定位是 LlamaParse 云服务的引流入口 |
| **MarkItDown**（微软，约 150.7k★，MIT，活跃） | Python 文档→Markdown 的事实标准 | WASM ❌；Rust crate ❌；纯 Python | 验证了需求体量；无嵌入形态，不构成四形态竞争 |
| **Docling**（IBM，约 61.4k★，MIT，活跃） | ML 文档理解标杆；**正在用 docling-slim 瘦身**（2026-06-10 仍在推进），但 standard extra 仍拉 torch | WASM ❌；Rust crate ❌ | 依赖臃肿已被社区证实（issue #809：装上后容器从 1 GB 涨到 7 GB），但"安装轻量"这个卖点正在被它自己追赶——spoor 的差异点必须落在嵌入/离线/防御，不能只讲安装体积 |
| **extractous**（约 1.8k★） | **已停更约 18 个月**（最后提交 2024-12-21）；Rust 壳 + Tika 经 GraalVM AOT 的混合架构，wheel 41–49 MB，JS 绑定从未交付 | WASM ❌（架构上不可能）；"纯 Rust"❌ | 此路线的头部先行者已出局，其混合架构恰好是反面教材 |
| **unpdf**（unjs，约 1.2k★，MIT） | PDF.js 的 serverless 魔改版（内联 worker + polyfill 才能跑 Workers） | 仅 PDF、仅 JS 运行时 | 边缘文档解析需求的存在证明；JS 生态要靠各种 hack 才能跑单一格式，无防御层 |
| **Cloudflare toMarkdown** | 平台内置免费"文档→Markdown"（Workers AI），支持 PDF/Office/HTML 等 | 平台托管服务，文档必须交给 Cloudflare 处理 | **平台方用真金白银验证了需求**，同时是边缘场景的直接竞争。spoor 的差异：不绑平台、文档不出运行环境、覆盖 toMarkdown 未列的 PPTX/EPUB/IPYNB |
| **anytomd-rs**（42★，Apache-2.0，活跃） | 纯 Rust 文档→Markdown，明确不做 PDF | WASM ❌；体量极小 | 同路线但缺 PDF；印证"纯 Rust + 含 PDF"组合的稀缺性 |

数据来源：GitHub API / npm registry / PyPI（2026-06-11 实时查询）；Cloudflare、AWS、Vercel、Deno 官方文档；docling issue #809、#2393。

## 平台硬约束（决定 WASM 工程目标）

| 平台 | 关键限制 | spoor 的应对 |
| --- | --- | --- |
| Cloudflare Workers | 包体 gzip 后 free 3 MB / paid 10 MB；CPU free 10 ms / paid 默认 30 s 可至 5 min；内存 128 MB（WASM 计入） | **核心格式 WASM 包 gzip ≤ 3 MB 是 P0 工程目标**；free 档 10 ms 只够小文件，主场景落在付费档；限制单次解析的数据量和输出封顶在 128 MB 共享内存下是刚需 |
| AWS Lambda | zip 50 MB / 解压 250 MB / 容器 10 GB；超时 15 min；同步 payload 6 MB | 限制宽松。走 Rust 静态二进制或 Python 绑定即可，大文档配合 S3 事件模式 |
| Vercel | Edge runtime 官方劝退（建议迁回 Node.js）；Hobby 档 1 MB | 在 Vercel 上使用 **Node 原生绑定**而非 WASM |
| Deno | 2.1 起 WASM 作为一等公民 import + 官方 wasmbuild 工具 | 分发路径现成；JSR/Deno 生态可低成本顺带覆盖 |

## 差异化主张自查（诚实版）

| 原主张 | 自查结果 | 修正 |
| --- | --- | --- |
| A. 没有"单引擎多格式 WASM" | **被推翻**：@kreuzberg/wasm 已做到多格式+OCR | 聚焦为："没有**塞得进免费边缘配额、适合浏览器插件**的多格式 WASM"（10.1 MB vs 目标 ≤3 MB gzip） |
| B. 没有"轻量纯 Rust 可嵌入 crate" | 部分成立：extractous 出局（混合架构+已停更）；LiteParse 依赖 PDFium/LibreOffice；kreuzberg 自称纯 Rust PDF 但 crate 体量是框架级 | 聚焦为："没有**单一职责、依赖可控、MIT 许可证**的纯 Rust 文档引擎 crate" |
| C. Python 生态依赖臃肿 | 成立但差距在缩小：docling-slim 在瘦身；kreuzberg wheel 也免 torch 但仍有 27–34 MB | 卖点从"装得小"升级为"**wheel 个位数 MB + 离线确定性 + 防御契约**"（对标 tokenizers 2.5–10 MB 的工程水准） |
| D. 内建防御层（限制数据量/ZIP 炸弹/输出封顶）罕见 | **被推翻**（2026-06-11 源码核实）：kreuzberg 有完整且文档化更好的防御层（SECURITY.md 威胁模型 + ZipBombValidator + 提取超时 + XML 炸弹验证器），覆盖面比 spoor 现状只多不少 | 重写为**攻击面论述**："安全差异不在有没有防御，而在需要防御的面积"——spoor 是一个可以整体审计的小代码面（单一职责、依赖可控、无 C/C++ 组件），kreuzberg 的防御层是包在 90+ 格式解析器、PDFium、tree-sitter、143 个 LLM provider 外面的。另外向其学习：已补上同等水准的 SECURITY.md 威胁模型。 |

**修正后的定位语**：

spoor 要做文档解析领域的 jq 或 ripgrep——体积小、攻击面小、全栈纯 Rust（含 WASM 端，而竞品的浏览器 PDF 解析都依赖 PDFium / C++）、MIT 许可证、单一职责（文档 → LLM 友好文本）、内建 Agent 交互契约（Markdown-first + 表格筛选协议 + 稳定错误码）。

它不是 kreuzberg 那样的文档智能框架，也不是 LiteParse 那样的云服务引流件。**绝不与 kreuzberg 拼格式数量、绑定语言数量和防御清单长度**——要拼的是"小"与"纯"，这是它的架构做不到的。

## 路线图

### P0：core 拆分 + 第一个绑定（证明"嵌入友好"的最小闭环）

| 事项 | 验收标准 |
| --- | --- |
| 类型化接口收敛（`ParseRequest`/`ParseResult`，ErrorCode 已落地） | 公共边界不再依赖 `anyhow` 的字符串契约；错误信息跨入口等价 |
| 保持行为不变的代码拆分 `spoor-core`/`spoor-cli` + 更名 | 现有全量 snapshot 和测试不变；更名独立提交；crates.io 占名（已确认可用） |
| PyO3 绑定（`pyspoor`） | maturin + abi3，macOS arm64 wheel 实测 1,379,367 bytes；平台矩阵对齐 tokenizers；预算/坏容器/压缩炸弹/CFB 拦截跨入口通过 |
| 防御契约文档化 | 已补上 SECURITY.md 威胁模型（每条威胁 × 对应防御 × 默认值 × 可配置项）；限制单次解析的数据量 / ZIP 三重上限 / 输出封顶各自有测试清单；提取超时仍待评估（kreuzberg 有，spoor 目前由调用方自行设置时限） |

### P1：WASM 与 Node（兑现差异化）

| 事项 | 验收标准 |
| --- | --- |
| `@harrisonwang/spoor-wasm` | **默认发布全格式 gzip ≤ 3 MB**，并保留 `core-formats` 裁剪构建；浏览器与 CF Worker/Pages demo 跑通 |
| `@harrisonwang/spoor`（napi-rs） | optionalDependencies 平台子包模式（沿用 spoor npm 包已验证的方案）；AnythingLLM/LobeChat 这类用 pdf-parse/pdfjs-dist 组合实现文档解析的项目可以一包替换 |
| 桌面集成样例 | Tauri 最小示例（直接 `cargo add spoor-core`） |

### P2：场景纵深（根据采用信号投入，不提前做）

- ~~Electron / 本地 AI 客户端集成样例。~~ 已提供完整 Electron 桌面应用；对外项目集成 PR 仍按采用信号投入。
- ~~Lambda / S3 批量文档导入示例。~~ 已提供 Lambda 二进制 Layer 示例。
- RAG 数据管道脚本：**不做**。RAG 不作为 spoor 的目标场景；原 `examples/rag-ingestion` 示例已移除。spoor 只保证确定性提取与跨宿主契约，下游检索/向量化由调用方自理。
- ~~EPUB / HTML 基础渲染器统一。~~ 已完成；更完整 HTML 语义节点与 PDF/PPTX
  阅读顺序增强仍按采用信号投入。

## 目标与验收

- 每个交付形态以**可运行的 demo** 收尾（浏览器拖拽、CF Worker、Tauri、RAG 管道脚本），不接受"理论上能跑"。
- 体积是公开承诺：README 直接标注各形态实测体积，像标注 benchmark 一样标注 size。
- 防御层是公开契约：恶意样本测试集（zip bomb、加密文档、伪造扩展名）跨四种形态全部通过。

## 做与不做

| 方向 | 判断 | 原因 |
| --- | --- | --- |
| 拼格式数量 / 拼绑定语言数量 | **不做** | 那是 kreuzberg 的战场，也是它变重的原因；spoor 赢在小与纯 |
| REST API server / MCP server / Docker 化服务 | **不做** | 继承原 CLI 阶段的决策；kreuzberg 已占满该生态位，做就变成第二个它 |
| OCR / VLM 默认内置 | 不做（保持可选后端的长期可能性） | 会破坏纯 Rust、体积、离线确定性三个根基 |
| 跟平台内置能力（CF toMarkdown）正面竞争托管便利性 | 不做 | spoor 的差异点是"不绑平台 + 文档不出端"，不是更便宜的托管 |
| WASM 发 crates.io | 不做 | 没有 Rust 消费者 |
| 按格式做编译裁剪 + 体积预算 | **必做** | 差异化主张 A（修正版）的全部依据 |
| MIT 许可证 | **坚持** | 相比 kreuzberg 的 Elastic-2.0 / marker 的 GPL-3，MIT 是结构性优势，零额外成本 |

## 待跟进问题（下次调研补）

- ~~kreuzberg 是否有等价的防御层~~ **已核实（2026-06-11，源码+SECURITY.md）：有，且文档化更好**。主张 D 已据此改写为攻击面论述，并补上 SECURITY.md 威胁模型。
- ~~@kreuzberg/wasm gzip 后实际大小~~ **已实测（2026-06-11）：主模块 8.2 MB + PDFium 1.9 MB ≈ 10.1 MB gzip**；其浏览器端 PDF 依赖 PDFium 而非纯 Rust。spoor 的 ≤3 MB 目标与"全栈纯 Rust"差异均成立。
- LiteParse 是否会补 Office 的纯 Rust 路线（当前依赖 LibreOffice 转换，如其重写则威胁升级）。
- docling-slim 裸装的真实能力边界。
- Deno Deploy（托管平台）对 WASM import 的支持目前只从 runtime 能力推断，未在 Deploy 文档直接确认。
