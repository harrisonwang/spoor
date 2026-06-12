# PDF 测试矩阵

PDF 当前只覆盖 text layer 抽取。PDF 是版面格式，不是语义格式，所以后续重点应是页边界、阅读顺序和噪声控制。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `pdf/01_basic.pdf` | `basic_text_layer` | 单页 text layer 可抽取并带 `## Page 1` | 基础 PDF 文本抽取 | passed | 标题/段落结构推断 |
| `pdf/02_multipage.pdf` | `multipage_has_page_boundaries` | 多页按顺序输出精确的 `## Page N` 边界 | 支持页码定位并防止只读第一页 | passed | 页眉页脚去重 |
| `pdf/03_ascii_only.pdf` | `ascii_baseline` | ASCII 文本不被编码处理破坏 | 最小稳定基线 | passed | Unicode PDF 字体映射 |
| `pdf/04_image_only.pdf` | `image_only_pdf_returns_structured_error` | 无 text layer 时返回可解析的 JSON 错误，明确需要 OCR | 防止 Agent 把空输出当成功并猜测内容 | passed | 可选 OCR backend |
| `pdf/05_mixed_text_and_image.pdf` | `mixed_pdf_reports_page_level_missing_text` | 混合 PDF 仅对无文本层页返回 `pdf_page_no_text_layer` 与页码 | Agent 可只把缺失页路由到外部 OCR/VLM | passed | 更广泛乱码/Type3 字体 corpus |

## 明确不覆盖

- image-only PDF：当前不做 OCR；返回 `image_only_pdf` 结构化错误。
- 复杂多栏阅读顺序：当前没有可靠修复。
- 页眉页脚去重：待实现。
- 可疑文本层仅做保守诊断，不自动改写或 OCR。

## 下一批优先用例

- 重复页眉页脚识别。
- 断词修复，例如行尾 hyphenation。
- 双栏/三栏阅读顺序与标题层级。
