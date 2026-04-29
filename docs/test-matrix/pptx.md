# PPTX 测试矩阵

PPTX 测试重点是 slide 级边界、文本内容、表格和 speaker notes。视觉主题、动画、转场默认不属于 LLM mode 契约。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `pptx/01_basic.pptx` | `basic_slides_with_titles_and_bullets` | 每页输出 `## Slide N`，标题和正文文本进入对应 slide | 保留演示文稿阅读边界 | passed | bullet 层级 marker |
| `pptx/02_with_table.pptx` | `tables_in_slides` | slide 内表格输出 GFM table | 防止表格被压平成一列文本 | passed | merged table cells |
| `pptx/03_with_notes.pptx` | `speaker_notes_are_included` | speaker notes 输出到 slide 下方 | notes 常包含演讲者真实上下文 | passed | notes 与 slide text 的顺序/标题规范 |
| `pptx/04_empty.pptx` | `empty_deck_with_blank_slide` | 空白 slide 输出稳定 | 边界输入 | passed | 是否省略完全空 slide |
| `pptx/05_ordering.pptx` | `slide_ordering_handles_double_digits` | `slide11.xml` 排在 `slide2.xml` 之后 | 防止字典序导致 slide 顺序错误 | passed | 按 presentation rels 顺序，而不只是文件名数字 |

## 下一批优先用例

- shape 坐标阅读顺序：多列/复杂版式。
- chart placeholder 或 chart data extraction。
- 图片 alt/caption placeholder。
- nested bullet 层级和编号。
