# Plain Text 与 CLI 测试矩阵

Plain text 测试覆盖编码和 passthrough；CLI 测试覆盖输出模式契约。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `plain/01_ascii.txt` | `ascii_passthrough` | ASCII 原样输出 | 最小基线 | passed | 无 |
| `plain/02_utf8.txt` | `utf8_passthrough` | UTF-8 多语言文本原样保留 | 多语言输入 | passed | invalid UTF-8 策略 |
| `plain/03_gbk.txt` | `gbk_decoded` | GBK 正确解码 | 中文文本关键路径 | passed | Big5/Shift-JIS |
| `plain/04_utf16le_bom.txt` | `utf16_le_with_bom_decoded` | UTF-16LE BOM 文件正确解码 | Office/Windows 文本兼容 | passed | UTF-16BE |
| `plain/05_code.py` | `code_file_passthrough` | 代码文本不被 Markdown 改写 | 代码文件可直接给 LLM | passed | 代码是否应 fenced |
| `html/06_links.html` | `default_mode_is_markdown_like_text` | CLI 默认 `--mode md`：stdout 为正文，不是 JSON | 默认模式契约 | passed | stdout newline 统一 |
| `plain/01_ascii.txt` | `json_mode_is_flat_placeholder_schema` | `--mode json` 输出当前占位 schema | JSON 占位契约 | passed | block JSON schema |

## 下一批优先用例

- stdin 输入。
- URL 输入。
- `--format` override 与 detect 失败。
- JSON mode 最终 block schema。
