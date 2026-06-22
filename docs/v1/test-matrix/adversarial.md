# Adversarial 测试矩阵

Adversarial 测试的目标是坏输入必须清楚失败或安全完成：不 panic、不 hang、不输出误导性内容。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `adversarial/01_empty.docx` | `empty_file_treated_as_docx` | 空 DOCX 返回 zip/archive/empty 相关错误 | 失败信息可诊断 | passed | 统一错误码 |
| `adversarial/02_not_zip.docx` | `non_zip_data_treated_as_docx` | 非 zip 数据按 DOCX 解析时清楚失败 | 不误判为正常文档 | passed | detected MIME 提示 |
| `adversarial/03_truncated_zip.docx` | `truncated_zip` | 截断 zip 不 panic，返回非空错误 | 抗损坏文件 | passed | 错误消息稳定性 |
| `adversarial/04_broken.ipynb` | `broken_json_ipynb` | 坏 JSON notebook 返回 JSON 相关错误 | Agent 可判断输入损坏 | passed | 字段路径 |
| `adversarial/05_compression_bomb.docx` | `compression_bomb_rejected_when_capped` | 不 panic；ZIP ratio/entry/total cap 和 CLI 解析内存上限由专项测试覆盖 | 安全前置用例 | passed | 嵌套容器策略 |

## 下一批优先用例

- ZIP 实际解压量与恶意中央目录不一致。
- 嵌套 zip / 巨大 sharedStrings / 巨大 XML text node。

专项覆盖：

- `limits::tests::zip_total_uncompressed_size_respects_parse_budget`：ZIP 总声明解压量受解析内存上限约束。
- CLI parse-budget tests：本地文件、stdin、多输入累计、提取文本膨胀超限时返回结构化错误。
- Python、Node 与 WASM smoke：同一份压缩炸弹 fixture 在 1 MiB 预算下返回
  `parse_budget_exceeded`；坏 DOCX 返回 `invalid_container`；CFB/OLE magic
  返回 `legacy_or_encrypted_office`。
