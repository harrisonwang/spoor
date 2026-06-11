# spoor：定位与工程规划

> 本计划是 pith 的下一阶段：从单一 CLI 演进为**一个 Rust 文档引擎，多种交付形态**，并更名为 `spoor`。
> 现有定位（离线、确定性、LLM-friendly，不做 OCR/云服务/MCP）全部保留，只是把引擎的运行方式从"必须是独立进程"扩展为"可嵌入任意宿主"。

## 一句话定位

把文档转成 LLM 可直接消费的文本。同一套 Rust 核心引擎，根据运行环境提供 CLI、原生库、WASM 三种交付形态。敏感文件始终不离开你的运行环境。

## 三项设计原则

| 原则 | 回答的问题 | 现状 |
| --- | --- | --- |
| **面向 LLM** | 输出形态：文档型 → Markdown，表格型 → schema + preview 的 JSON，token 经济 | ✅ 已有 |
| **面向 Agent** | 调用方式：状态自描述（usage / truncated / warnings），失败时按稳定错误码分支 | ✅ 已有 |
| **面向嵌入** | 运行位置：输入 bytes、输出结构化结果；无隐式 I/O、无全局状态、资源有上限，恶意输入不会危及宿主 | ⬜ 性质已具备，形态待交付 |

面向嵌入是本阶段的核心目标。它不引入新功能，而是将已有的工程特性——限制单次解析的数据量、ZIP 炸弹防御、每次调用无状态、结构化错误——从 CLI 进程的壳里释放出来，让浏览器、Edge Runtime、桌面应用、多租户沙箱都能直接内嵌这套引擎。

## 交付形态与场景

| 交付形态 | 目标场景 | 核心优势 |
| --- | --- | --- |
| **CLI**（单二进制 `spoor`） | Shell 脚本、CI/CD、本地开发机、个人 Agent（Claude Code / Cursor 直接调用） | 开箱即用，pipe 友好 |
| **Rust crate**（`spoor-core`） | Tauri / Rust 桌面客户端、嵌入式服务——本地 AI 客户端的文本提取底座 | 零进程开销，直接函数调用 |
| **Python / Node 原生绑定** | RAG 数据管道（Airflow / Dagster）、后端服务、Electron 桌面应用 | 免去子进程频繁启停，拿到的是结构化结果和异常，而不是字符串 |
| **WASM** | 浏览器插件、纯前端离线应用（"本地文件对话"无后端）、Cloudflare Workers / Lambda 的请求时文档清洗 | 文档 100% 不出端；冷启动 ≈ 0；WASM 本身就是沙箱 |

**防御层不是第五种交付形态，而是四种形态共享的安全基座**：限制单次解析的数据量、ZIP 炸弹防御（入口大小 / 解压比 / 总上限三重限制）、输出封顶、结构化错误。有了这层防御，四种交付形态都可以部署到不可信环境中处理恶意文档——无论是多租户容器、Wasmtime / Wasmer 沙箱，还是受限的 Lambda 函数。

注意：这里说的"数据量预算"是解析阶段处理的数据体积上限，不是操作系统级的 RSS 硬限制。多租户隔离仍然需要容器或 WASM runtime 的外层配合，两者互补。

## 为什么这些场景成立（对应代码现状）

- **防御层已实现**：限制单次解析的数据量、ZIP 三重上限、256 KiB 输出封顶、带稳定错误码的结构化错误，均有测试覆盖。
- **core 的边界已设计好**：[架构设计](../design/architecture.md) 规定 core 只收 bytes，无网络、无文件、无进程退出。这种纯粹性恰好是 PyO3、napi、WASM 三个绑定的共同基础——一次拆分，三处受益。
- **WASM 可行性已验证**：所有解析依赖均可编译到 wasm32 目标（pdf-extract、calamine、zip+miniz、quick-xml、scraper、csv、encoding_rs 均为纯 Rust 实现）。唯一不能进 wasm32 的是网络请求和文件读取，它们本来就只属于 CLI 层。

## 工程约束

- **WASM 体积**：PDF / Office 解析链可能把包体推到数 MB，而部分 Edge 平台有体积上限。按格式做编译特性开关（如浏览器版全功能、Worker 版裁剪），发布前实测体积。
- **WASM 包只发 npm**：`spoor-wasm` 是 wasm-bindgen 入口，没有 Rust 消费者，不占 crates.io。
- **命名占位已核查（2026-06-11）**：crates.io 的 `spoor` / `spoor-core` / `spoor-cli` 可用；PyPI 的 `spoor` 已被占用，改用 `pyspoor`；npm 的 `spoor` 已被占用，改用 `@harrisonwang/spoor`。发布前需再次确认。
- **不为假设场景预先优化**：每个交付形态以可运行的 demo 作为交付标准，不做"理论上能跑"。

## 工程目录规划

```
spoor/
├── Cargo.toml                    # workspace root
├── README.md
├── LICENSE
│
├── crates/                       # Rust 原生包，Cargo 管理和发布
│   ├── spoor-core/               # ─── 核心引擎
│   │   ├── Cargo.toml            #     纯逻辑：输入 bytes，输出结构化结果；零 I/O、零 CLI 依赖
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── detect/           # 格式检测（magic bytes + 扩展名 + CFB 拦截）
│   │       ├── parse/            # 各格式解析器（PDF/DOCX/XLSX/CSV/EPUB/HTML/IPYNB/PPTX）
│   │       ├── limits.rs         # 数据量限制、ZIP 炸弹防御
│   │       ├── result.rs         # 结构化 ParseResult
│   │       └── error.rs          # 结构化错误（ErrorCode 契约原样继承）
│   │
│   ├── spoor-cli/                # ─── CLI 入口（clap、glob、stdin、URL 抓取、退出码）
│   │   ├── Cargo.toml            #     binary 名：spoor
│   │   └── src/
│   │       └── main.rs
│   │
│   └── spoor-wasm/               # ─── WASM 入口（wasm-bindgen）
│       ├── Cargo.toml            #     wasm-pack 直接产出 npm 包 @harrisonwang/spoor-wasm
│       └── src/
│           └── lib.rs
│
├── bindings/                     # 跨语言绑定，各自发布到 PyPI / npm
│   ├── python/                   # ─── PyO3 → PyPI: pyspoor
│   │   ├── Cargo.toml
│   │   ├── pyproject.toml        #     maturin
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── spoor/                #     轻量 Python 封装层（dataclass + 异常）
│   │       ├── __init__.py
│   │       ├── exceptions.py
│   │       └── types.py
│   │
│   └── node/                     # ─── napi-rs → npm: @harrisonwang/spoor
│       ├── Cargo.toml
│       ├── package.json
│       ├── src/
│       │   └── lib.rs
│       ├── __test__/
│       └── index.js              #     JS 入口 + 类型声明
│
├── wasm/                         # ─── WASM demo 与边缘示例（npm 包产物来自 crates/spoor-wasm）
│   ├── demo/                     #     浏览器拖拽解析 demo
│   └── cloudflare-worker/        #     CF Worker 请求时清洗示例
│       ├── wrangler.toml
│       └── src/
│           └── index.ts
│
└── examples/
    ├── serverless-lambda/        # AWS Lambda（spoor-cli 二进制或 WASM）
    └── chat-with-local-file/     # 纯前端离线"本地文件对话"
```

## 平台命名汇总

| 入口 | 包名 / 产物 | 发布到哪里 |
| --- | --- | --- |
| Rust core | `spoor-core` | crates.io |
| Rust CLI | `spoor-cli`（binary：`spoor`） | crates.io + Homebrew + Scoop |
| WASM | `@harrisonwang/spoor-wasm` | npm（不发 crates.io） |
| Python | `pyspoor` | PyPI |
| Node.js 原生 | `@harrisonwang/spoor` | npm |
| Homebrew | `harrisonwang/homebrew-tap/spoor` | GitHub tap |
| Scoop | `harrisonwang/scoop-bucket/spoor` | GitHub bucket |

## 推进顺序

1. **稳定类型化接口**（已起步）：`ErrorCode` / `StructuredError` 已落地；剩余按 [架构设计](../design/architecture.md) 收敛 `ParseRequest` / `ParseResult`。
2. **保持行为不变的代码拆分 + 更名**：解析模块移入 `spoor-core`，CLI 仅调用 core；现有 snapshot 和测试全量通过后再执行更名，更名作为一次独立提交。
3. **PyO3 MVP**：提供 `parse_bytes` / `parse_path`，复用同一套测试用例，错误字段与 CLI 保持一致。
4. **napi-rs 与 WASM 入口**：wasm32 编译验证 → 体积实测 → 按需裁剪功能。
5. **场景 demo 验收**：浏览器拖拽解析、CF Worker 清洗、Tauri 最小示例——每个交付形态以可运行的 demo 收尾。

每一步的验收标准（与 CLI 输出一致、跨入口结果等价、先跑基准测试）沿用 [架构设计](../design/architecture.md)，此处不赘述。
