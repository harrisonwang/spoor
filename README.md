# gist

把文件或 URL 转成适合 LLM / Agent 使用的文本。**默认 `--mode md`**：stdout 为 Markdown-like 正文（与 `json` 相对，表示**输出形态**，工具整体仍面向模型与 Agent）。

`json` 模式已经预留，但目前只是一个扁平占位 schema；真正的 block/anchor 结构化 JSON 稍后再实现。

## 使用

```bash
gist report.docx
gist data.xlsx
gist slides.pptx
gist paper.pdf
gist notebook.ipynb
gist book.epub
gist data.csv

gist report.docx --mode md
gist report.docx -m json
gist file.txt --format text

gist *.pdf
gist "*.pdf"
gist report.pdf notes.docx
```

`gist` 支持多个输入，并会在程序内部展开本地 glob（如 `*.pdf`、`docs/**/*.md`），因此在 Windows `cmd.exe`、PowerShell、macOS/Linux shell 下行为更一致。URL 不会被当作 glob 展开。

## 输出模式

### `md`，默认

目标是生成低噪声、结构清楚、token 相对经济、便于直接塞进上下文的 Markdown-like 文本。

基本原则：

- 保留标题、段落、列表、表格、链接、脚注、sheet、slide、chapter 等语义结构。
- 丢弃字体、字号、颜色、边距、主题、装饰 shape 等纯视觉样式。
- 对表格、sheet、slide、page 等内容块保留清楚边界。
- 小表优先 GFM table；大表后续会切到 fenced TSV/CSV 并带截断说明。
- 转换过程中不做 OCR，不执行 notebook，不计算 Excel 公式。

### `json`，占位

当前输出如下：

```json
{
  "mode": "json",
  "schema_version": "gist-json-v0",
  "status": "placeholder",
  "format": "docx",
  "source": "report.docx",
  "content": "...markdown body..."
}
```

后续 JSON 模式会改成 block-oriented 结构，例如 `blocks[]`、`source_anchor`、`page`、`slide`、`sheet`、`row_range` 等字段。现在不要把 `gist-json-v0` 当成最终稳定 schema。

## 支持格式

| 格式 | 当前策略 | 主要取舍 |
| --- | --- | --- |
| DOCX | 转成结构化 Markdown | 保留标题、段落、列表、表格、链接、脚注；默认接受 tracked changes |
| XLSX | sheet + 小表 GFM table | 日期转 ISO，公式用 cached value；大表 compact 策略待补 |
| PPTX | slide blocks | 保留 slide 顺序、表格、speaker notes；视觉顺序和 bullet 层级仍需加强 |
| PDF | text layer passthrough | 目前缺 page boundary 和去页眉页脚；扫描 PDF/OCR 不支持 |
| EPUB | OPF spine 顺序 | 当前正文渲染仍偏弱，后续应复用 HTML Markdown renderer |
| IPYNB | markdown + code cells | 默认丢弃 outputs 和 raw cells，code fence 带 language hint |
| CSV/TSV | 小表 GFM table | 自动编码和 delimiter；大文件截断策略已有但还需统一到 mode contract |
| HTML/URL | 基础正文抽取 | 支持 article/main、heading、list、link、table；暂不作为重点 |
| Markdown/text/code | passthrough | 原文质量决定输出质量；代码 fenced block 策略待补 |

## 构建

```bash
cargo build --release
./target/release/gist file.docx
```

## 设计边界

`gist` 不是 `extract-text` 的逐字复刻。它的目标是 LLM-oriented extraction：

- 更重视语义结构，而不是视觉样式。
- 更重视 token 经济，而不是无损还原所有 XML 细节。
- 更重视 Agent 可定位、可分块、可引用，而不是只输出一大段纯文本。

当前仍缺少几个关键工程能力：

- ZIP 类格式的统一安全限制：entry cap、compression ratio cap、total output cap。
- PDF page boundary、阅读顺序修复、重复页眉页脚识别。
- EPUB 的完整 Markdown renderer。
- 大表自动选择 GFM table / fenced TSV / 截断摘要。
- JSON mode 的 block schema。

这些比继续增加新格式更优先。

## 测试

```bash
cargo check
cargo test
```

快照测试使用 `insta`。没有安装 `cargo-insta` 时，可以用：

```bash
INSTA_UPDATE=always cargo test
```

接受新快照后不要提交 `.snap.new` 文件，只提交正式 `.snap`。

测试用例的设计意图和覆盖缺口记录在 `docs/test-matrix/`。新增 fixture 时，先更新对应格式的测试矩阵，再接受 snapshot。
