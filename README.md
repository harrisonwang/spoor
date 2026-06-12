# spoor

把文档转成 LLM 可直接消费的文本。同一套引擎，根据运行环境提供 CLI、原生库、WASM 三种交付形态。敏感文件始终不离开你的运行环境。

> **当前状态**：`spoor-core`、CLI、Python、Node 与 WASM 入口均已落地。完整规划见 [docs/v1/](docs/v1/)。

## 核心特性

- **按形态自动分派输出**：文档型（PDF/DOCX/PPTX/EPUB/IPYNB/HTML）→ Markdown，表格型（CSV/XLSX）→ JSON（headers + preview + range）
- **离线、单二进制**：无云依赖，不需要 Python 环境，敏感文件本地处理
- **Agent 友好**：结构化错误（稳定 error code）、输出自描述（usage/truncated/warnings）、JSON 扁平 `tables[]`
- **内建防御**：限制单次解析的数据量、ZIP 炸弹三重防御（entry/ratio/total cap）、256 KiB 输出封顶
- **支持格式**：PDF、DOCX、XLSX、PPTX、CSV、EPUB、IPYNB、HTML/URL、Markdown、纯文本

包体大小（2026-06-11 实测）：

| 形态 | 大小 |
|------|------|
| `spoor-core` crate | < 140 KiB |
| CLI（macOS arm64 单二进制） | ~4.7 MiB |
| `pyspoor` abi3 wheel | ~1.3 MiB |
| Node addon | ~2.8 MiB |
| `core-formats` WASM | ~1.4 MiB raw / ~575 KiB gzip |
| `full` WASM（含 PDF） | ~2.1 MiB raw / ~838 KiB gzip |

## 安装

```bash
# macOS / Linux
brew install harrisonwang/tap/spoor

# Windows
scoop bucket add harrisonwang https://github.com/harrisonwang/scoop-bucket
scoop install spoor

# 跨平台 CLI（npm）
npm install -g @harrisonwang/spoor-cli

# 源码安装（需 Rust toolchain）
cargo install spoor-cli
```

## 使用

```bash
# 文档型 → Markdown
spoor report.pdf
spoor report.docx slides.pptx
spoor https://example.com/article

# 表格型 → JSON（schema + preview）
spoor data.xlsx
spoor data.xlsx --sheet Sheet1 --rows 5:104 --columns 分类,金额
spoor data.csv | jq '.tables[0].headers'

# stdin / pipe
cat data.csv | spoor --format csv -

# glob
spoor "docs/*.pdf"

# 直接喂给 LLM
spoor report.pdf | llm "总结风险和行动项"
```

输出模式按格式自动分派，`-m` 可显式覆盖。表格型 JSON 默认返回前 100 行预览，通过 `--rows` / `--columns` / `--limit` / `--offset` 收窄。详见 `spoor --help`。

## 嵌入

Rust core 只接收 bytes 与 metadata，不执行文件、网络或进程 I/O：

```rust
let mut request = spoor_core::ParseRequest::new(bytes);
request.source_name = Some("report.docx");
let result = spoor_core::parse(&request)?;
```

Python 使用 `pyspoor` 的 `parse_bytes` / `parse_path`；Node.js 使用
`@harrisonwang/spoor`；浏览器与 Edge Runtime 使用
`@harrisonwang/spoor-wasm`。可运行示例位于 `wasm/` 与 `examples/`。

## 错误契约

所有入口共享 `SpoorError`，消费者只按稳定 `code` 分支：

| code | 含义 |
| --- | --- |
| `image_only_pdf` | PDF 无文本层，需要外部 OCR |
| `parse_budget_exceeded` | 输入、解压或结果超过解析预算 |
| `unsupported_format` | 无法识别或不支持格式 |
| `encrypted_pdf` | PDF 受密码保护 |
| `legacy_or_encrypted_office` | 旧版或加密 Office 容器 |
| `invalid_container` | ZIP 类容器为空、损坏或类型不符 |
| `parse_failed` | 已规范化的其他解析失败；查看 `stage` |

## 开发

```bash
# 构建
cargo build --release

# 测试
cargo test --locked --workspace --all-targets

# 代码检查
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings

# 快照测试（insta）
cargo insta review
```

## 文档

| 文档 | 内容 |
| --- | --- |
| [定位与工程规划](docs/v1/planning/overview.md) | 一句话定位、设计原则、交付形态、推进顺序 |
| [路线图与竞品分析](docs/v1/planning/roadmap.md) | 竞品调研、平台约束、差异化自查 |
| [架构设计](docs/v1/design/architecture.md) | Core 边界、错误契约、PyO3 接口、迁移顺序 |
| [工程决策](docs/v1/design/decisions.md) | 产品边界、输出模式、格式取舍、安全策略 |
| [测试矩阵](docs/v1/test-matrix/) | 按格式维护的测试覆盖 |
| [安全模型](SECURITY.md) | 威胁、默认防御、边界与结构化错误 |
