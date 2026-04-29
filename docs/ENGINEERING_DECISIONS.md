# 工程决策

本文档记录 `gist` 的 LLM-oriented extraction contract。它不是为了复刻 `extract-text` 的逐字输出，而是为了把不同文件格式转换成更适合 LLM 和 Agent 使用的文本表示。

当前只定义两种输出模式（均面向 LLM / Agent；名称表示 **stdout 形态**）：

- `md`：默认，stdout 为 Markdown-like 正文。
- `json`：stdout 为 JSON 占位 envelope，完整 block JSON 稍后实现。

## 判断标准

“转成 Markdown”不自动等于 LLM 友好。我们按下面几条判断输出是否合格：

1. **结构保真**：标题、段落、列表、表格、链接、脚注、页、sheet、slide、chapter 等结构不能轻易丢。
2. **阅读顺序正确**：DOCX 段落顺序、PPTX slide 顺序、EPUB spine 顺序、PDF page 顺序要可预期。
3. **噪声低**：脚本、样式、导航、广告、装饰 shape、空占位符、重复页眉页脚默认不进入正文。
4. **token 经济**：小表可以 GFM table；大表、宽表更适合 fenced TSV/CSV 或摘要。
5. **便于 Agent 调用**：内容边界清楚，后续 JSON mode 能给出 block、anchor、页码、sheet、row range 等信息。
6. **转换性能可控**：不能为了格式还原引入过重依赖或不受控内存占用。
7. **恶意输入安全**：ZIP 类格式必须逐步补齐 entry cap、压缩比限制和总输出限制。

## 输出模式

### `md`

默认模式。面向模型上下文，优先保留语义结构，丢弃纯视觉信息。

当前策略：

- 文本文档输出 Markdown-like 文本。
- 小表输出 GFM table。
- notebook 输出 markdown cells 和 code cells，默认不输出 cell outputs。
- slide/sheet/chapter 使用明确标题分块。

待补策略：

- 大表自动切换 fenced TSV/CSV。
- PDF 加 page boundary。
- EPUB 复用 HTML Markdown renderer。
- 所有格式统一 total output cap 和 truncation marker。

### `json`

当前只是占位：

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

最终目标不是把 Markdown 包一层 JSON，而是输出 block-oriented schema：

- `blocks[]`
- `kind`
- `text`
- `source_anchor`
- `page`
- `slide`
- `sheet`
- `row_range`
- `truncated`

这部分暂不实现，避免过早锁定错误 schema。

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

和 `extract-text` 的主要差异：

- `pStyle="ListBullet"` / `ListNumber` 会识别成列表；`extract-text` 常漏掉。
- decimal numbering 输出 `1.`、`2.`；`extract-text` 通常 flatten 成 `-`。
- 表格里的 `|` 用 `\|` 转义，而不是替换成外观相似字符。
- 相邻 bold/italic run 会尽量输出合法 Markdown。

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

当前小表输出 GFM table。这个对短表很友好，但对大表不经济。后续规则应当是：

- 小表：GFM table。
- 宽表/长表：fenced TSV/CSV。
- 超大表：摘要 + row/col range + truncation marker。

注意：当前日期转换还没有完整处理 Excel 1904 date system，这是实现层面的待补项。

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

当前优于 `extract-text` 的地方：

- 表格不再扁平化成一列文本。
- speaker notes 会进入输出。

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

- page boundary
- 断词修复
- 多栏阅读顺序
- image-only PDF 明确提示需要 OCR

### EPUB

保留：

- OPF spine 顺序
- chapter boundary
- HTML 内部 heading/list/link/inline formatting

当前只完成了 spine 顺序；正文仍然主要是 text extraction，Markdown 结构不够好。这是明确待补项，应复用或抽象 HTML renderer。

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

当前小表输出 GFM table。后续应和 XLSX 统一：

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

`extract-text` 的一个重要工程选择是 zip 防御能力：

- per-entry decompressed-size cap
- compression-ratio bomb threshold
- total extracted text cap
- service-limits 模式

`gist` 目前还没有完整实现这些能力。下一阶段应优先补一个统一的 ZIP 读取层，让 DOCX/XLSX/PPTX/EPUB 共用限制逻辑，而不是每个 extractor 自己 `read_to_string`。

这是服务化和 Agent 自动调用场景的前置条件。

## 测试策略

测试不应只验证“输出不为空”。应覆盖：

- 结构是否保留：heading/list/table/link/footnote。
- 噪声是否丢弃：script/style/nav/empty placeholders。
- 顺序是否正确：spine、slide number、sheet order。
- token 经济策略：大表和长 CSV 的截断。
- 错误是否清楚：坏 zip、空文件、坏 JSON。
- JSON mode：当前只验证占位 schema，暂不验证 block schema。
