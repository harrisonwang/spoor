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
| `plain/01_ascii.txt` | `json_mode_rejects_plain_text` | `--mode json` 对非 CSV/XLSX 返回清楚错误 | JSON 选择性契约 | passed | 无 |
| `(stdin)` | `stdin_dash_reads_text_as_markdown` | `-` 从 stdin 读文本，默认 md | shell 管道一等公民 | passed | 无 |
| `(stdin)` | `stdin_csv_with_format_flag_emits_json` | `--format csv -` 从 stdin 出表格 JSON，`source` 为 `-` | 管道喂表格 | passed | 无 |
| `xlsx/01_basic.xlsx` | `stdin_xlsx_detected_by_magic_bytes` | stdin 无扩展名靠 magic-byte 识别 xlsx | 管道二进制自动识别 | passed | 无 |

## 下一批优先用例

- URL 输入。
- `--format` override 与 detect 失败。
- JSON mode 当前只支持 CSV/XLSX table schema；plain/text 使用 Markdown。
