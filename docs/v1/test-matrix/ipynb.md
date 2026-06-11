# IPYNB 测试矩阵

IPYNB 测试重点是按 cell 顺序保留 notebook 的输入侧内容。默认不执行代码，也不输出 cell outputs。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `ipynb/01_basic.ipynb` | `markdown_and_code_cells` | markdown passthrough，code cell 使用 fenced code block，outputs 跳过 | 控制 token，避免二进制/HTML output 噪声 | passed | 可选短 text output |
| `ipynb/02_source_formats.ipynb` | `source_can_be_string_or_array` | `source` 可以是 string 或 string array | nbformat 两种合法表示都要支持 | passed | 非字符串 source 的错误信息 |
| `ipynb/03_language_hint.ipynb` | `language_hint_from_kernelspec` | 从 kernelspec 推断 code fence language | 提高代码可读性 | passed | cell-level language override |
| `ipynb/04_raw_cells.ipynb` | `raw_cells_skipped` | raw cell 默认跳过 | 降低不确定内容噪声 | passed | 是否提供保留 raw cell 的模式 |
| `ipynb/05_malformed.ipynb` | `malformed_ipynb_returns_clear_error` | malformed notebook 返回包含 `cells` 的清楚错误 | 便于 Agent 判断失败原因 | passed | 更细字段路径 |

## 下一批优先用例

- `--outputs text`：只保留短 stdout/text/plain。
- 大 notebook 截断策略。
- markdown cell 中嵌 HTML 的处理。
- execution_count 是否保留。
