# Adversarial 测试矩阵

Adversarial 测试的目标是坏输入必须清楚失败或安全完成：不 panic、不 hang、不输出误导性内容。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `adversarial/01_empty.docx` | `empty_file_treated_as_docx` | 空 DOCX 返回 zip/archive/empty 相关错误 | 失败信息可诊断 | passed | 统一错误码 |
| `adversarial/02_not_zip.docx` | `non_zip_data_treated_as_docx` | 非 zip 数据按 DOCX 解析时清楚失败 | 不误判为正常文档 | passed | detected MIME 提示 |
| `adversarial/03_truncated_zip.docx` | `truncated_zip` | 截断 zip 不 panic，返回非空错误 | 抗损坏文件 | passed | 错误消息稳定性 |
| `adversarial/04_broken.ipynb` | `broken_json_ipynb` | 坏 JSON notebook 返回 JSON 相关错误 | Agent 可判断输入损坏 | passed | 字段路径 |
| `adversarial/05_compression_bomb.docx` | `compression_bomb_rejected_when_capped` | 当前只保证不 panic；完整 cap 尚未接入测试 harness | 安全前置用例 | partial | per-entry cap、ratio cap、total output cap |

## 下一批优先用例

- ZIP entry decompressed-size cap。
- compression-ratio bomb threshold。
- total extracted text cap。
- 嵌套 zip / 巨大 sharedStrings / 巨大 XML text node。
