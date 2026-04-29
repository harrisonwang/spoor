# CSV 测试矩阵

CSV 测试重点是编码、delimiter、quoted fields 和 token 经济。CSV 本质是文本表格，不默认做强类型推断。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `csv/01_basic.csv` | `basic_comma_separated` | 逗号分隔，小表输出 GFM table，首行 header | 最常见路径 | passed | 无 header 的 fallback |
| `csv/02_tab_separated.csv` | `tab_delimiter_auto_detected` | 自动识别 tab delimiter | 用户无需手动指定 TSV | passed | delimiter 采样冲突 |
| `csv/03_semicolon.csv` | `semicolon_delimiter_european_excel` | 自动识别 semicolon delimiter | 欧洲 Excel 常见导出格式 | passed | 小数逗号不应误判 |
| `csv/04_gbk.csv` | `gbk_encoding_decoded` | GBK CSV 正确解码 | 中文 Windows Excel 关键路径 | passed | Shift-JIS/Big5 |
| `csv/05_utf8_bom.csv` | `utf8_bom_stripped` | UTF-8 BOM 不进入首个 header cell | Excel UTF-8 导出常见 | passed | UTF-16 CSV |
| `csv/06_quoted.csv` | `rfc4180_quoted_fields` | quoted commas、quotes、cell 内 newline 正确解析 | 避免错误拆列 | passed | cell 内多行保留策略 |
| `csv/07_empty.csv` | `empty_file` | 空文件输出稳定 | 边界输入 | passed | 空文件 warning |
| `csv/08_pipe.csv` | `pipe_delimiter` | 自动识别 pipe delimiter | 常见日志/导出格式 | passed | pipe cell escaping |
| `csv/09_ragged.csv` | `ragged_rows_padded` | ragged rows 按最大列数右侧补齐 | GFM table 需要统一列数 | passed | 行级错误提示 |
| `csv/10_large.csv` | `large_file_truncated` | 大文件按行数截断并输出 truncation footer | token 经济和性能保护 | passed | fenced CSV/TSV compact 模式 |

## 下一批优先用例

- 宽表：超过列数阈值时切换 fenced TSV/CSV。
- 无 header CSV：是否生成 `col1` 还是当数据行处理。
- UTF-16LE/BE CSV。
- delimiter 采样在 quoted 内容中的抗干扰能力。
