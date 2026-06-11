# spoor

把文档转成 LLM 可直接消费的文本。同一套引擎，根据运行环境提供 CLI、原生库、WASM 三种交付形态。敏感文件始终不离开你的运行环境。

> **当前状态**：CLI 已可用（binary 名暂为 `pith`，更名为 `spoor` 进行中）。完整规划见 [docs/v1/](docs/v1/)。

## 核心特性

- **按形态自动分派输出**：文档型（PDF/DOCX/PPTX/EPUB/IPYNB/HTML）→ Markdown，表格型（CSV/XLSX）→ JSON（headers + preview + range）
- **离线、单二进制**：无云依赖，不需要 Python 环境，敏感文件本地处理
- **Agent 友好**：结构化错误（稳定 error code）、输出自描述（usage/truncated/warnings）、JSON 扁平 `tables[]`
- **内建防御**：限制单次解析的数据量、ZIP 炸弹三重防御（entry/ratio/total cap）、256 KiB 输出封顶
- **支持格式**：PDF、DOCX、XLSX、PPTX、CSV、EPUB、IPYNB、HTML/URL、Markdown、纯文本

## 安装

```bash
# macOS / Linux
brew install harrisonwang/tap/pith

# Windows
scoop bucket add harrisonwang https://github.com/harrisonwang/scoop-bucket
scoop install pith

# 跨平台（npm）
npm install -g @harrisonwang/pith

# 源码安装（需 Rust toolchain）
cargo install --git https://github.com/harrisonwang/pith
```

## 使用

```bash
# 文档型 → Markdown
pith report.pdf
pith report.docx slides.pptx
pith https://example.com/article

# 表格型 → JSON（schema + preview）
pith data.xlsx
pith data.xlsx --sheet Sheet1 --rows 5:104 --columns 分类,金额
pith data.csv | jq '.tables[0].headers'

# stdin / pipe
cat data.csv | pith --format csv -

# glob
pith "docs/*.pdf"

# 直接喂给 LLM
pith report.pdf | llm "总结风险和行动项"
```

输出模式按格式自动分派，`-m` 可显式覆盖。表格型 JSON 默认返回前 100 行预览，通过 `--rows` / `--columns` / `--limit` / `--offset` 收窄。详见 `pith --help`。

## 开发

```bash
# 构建
cargo build --release

# 测试
cargo test --locked

# 代码检查
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings

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
