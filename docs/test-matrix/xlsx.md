# XLSX 测试矩阵

XLSX 测试重点是 sheet 和 cell 的语义值，而不是 Excel 的视觉格式。默认不计算公式，只读取文件中已有的 cached value。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `xlsx/01_basic.xlsx` | `basic_sheet_as_gfm_table` | 单 sheet、小表转 GFM table，首行作为 header | LLM 能直接理解列结构 | passed | header 推断失败时的 fallback |
| `xlsx/02_multi_sheets.xlsx` | `multiple_sheets_with_empty_one` | 每个 sheet 有 `## Sheet: name` 边界，空 sheet 也保留 | Agent 需要知道内容所属 sheet | passed | hidden sheet、very hidden sheet |
| `xlsx/03_types.xlsx` | `cell_types_numbers_dates_bools_formulas` | 字符串、数字、布尔、日期、时间、公式 cached value 稳定输出 | 避免 Excel serial date 对 LLM 不友好 | passed | Excel 1904 date system、locale format |
| `xlsx/04_sparse_merged.xlsx` | `sparse_rows_and_merged_cells` | 稀疏行补齐，merged cell 使用 top-left 值 | 防止行列错位 | passed | merged range 注释 |
| `xlsx/05_formulas.xlsx` | `formulas_use_cached_value` | 有 `<v>` 时输出 cached value，无 cached value 不求值 | 性能、安全、确定性取舍 | passed | 可选输出公式表达式 |
| `xlsx/06_empty.xlsx` | `empty_workbook` | 空 workbook 输出稳定 | 边界输入 | passed | 空 workbook warning |
| `xlsx/07_special_chars.xlsx` | `special_characters_safe_for_markdown` | pipe 转义，cell 内 newline/tab 变空格，Unicode 保留 | Markdown table 不被破坏 | passed | fenced TSV 模式下的 escaping |
| `xlsx/08_shared_strings.xlsx` | `shared_strings_resolve_correctly` | `t=s` cell 正确解析 sharedStrings | XLSX 常见字符串存储方式 | passed | rich text shared string runs |

## 下一批优先用例

- 大表/宽表：自动从 GFM table 切到 fenced TSV，并带 row/col range。
- 公式表达式：未来如果支持 `--formulas`，需要覆盖表达式和值的并存策略。
- 数字格式：百分比、货币、日期格式是否尊重 cell style。
- sheet 可见性：hidden sheet 是否默认跳过或标注。
