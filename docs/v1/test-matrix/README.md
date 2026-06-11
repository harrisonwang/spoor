# 测试矩阵

这里记录 `pith` 的测试设计，而不是复制完整输出。

精确输出由 `tests/snapshots/*.snap` 负责；设计意图、覆盖范围、已知缺口放在本目录。这样评审 snapshot diff 时可以先判断“这个行为是否属于契约”，再决定是否接受输出变化。

## 文档结构

| 文档 | 范围 |
| --- | --- |
| [docx.md](docx.md) | WordprocessingML、段落、列表、表格、脚注、tracked changes |
| [xlsx.md](xlsx.md) | workbook、sheet、单元格类型、日期、公式 cached value、大表策略 |
| [pptx.md](pptx.md) | slide 顺序、文本框、表格、speaker notes |
| [pdf.md](pdf.md) | text layer、页序、当前 PDF 边界 |
| [epub.md](epub.md) | OPF spine、chapter 顺序 |
| [ipynb.md](ipynb.md) | notebook cells、source 形态、language hint、错误处理 |
| [csv.md](csv.md) | delimiter、编码、quoted fields、ragged rows、截断 |
| [html.md](html.md) | readability、heading/list/link/table、噪声过滤 |
| [plain.md](plain.md) | 纯文本编码、CLI 输出模式契约 |
| [adversarial.md](adversarial.md) | 坏输入、恶意输入、安全边界 |

## 每个用例记录什么

每个格式文档都按同一种表格写：

| 字段 | 含义 |
| --- | --- |
| Fixture | 输入样本路径 |
| Test | Rust 测试函数名 |
| 验证契约 | 这个用例锁定的 LLM/工程行为 |
| 价值 | 为什么这个行为值得测试 |
| 状态 | 当前是否通过 |
| 后续缺口 | 明确还没有覆盖的相关能力 |

## 新增用例判断

新增 fixture 前先问：

- 是否覆盖新的 LLM 输出契约？
- 是否防止一个真实回归？
- 是否覆盖安全、性能或 token 经济边界？
- 是否只是在换一组内容，但行为和已有用例重复？
- 是否需要同步更新 `docs/ENGINEERING_DECISIONS.md`？

如果只是精确输出变化，不新增契约，通常只需要审阅 snapshot diff；如果新增了行为边界，就应更新对应格式的测试矩阵。
