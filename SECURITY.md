# 安全模型

`spoor` 使用一个小型、确定性的 Rust core 来解析不可信文档。core 的设计目标是失败时返回结构化错误，但它**不能替代**操作系统或 WASM runtime 的内存限制。

格式保留内容、运行形态与示例的完整边界见
[能力与限制](docs/v1/design/limitations.md)。

## 信任边界

- `spoor-core` 只接收 bytes 和显式 metadata，不执行任何文件、网络、stdin/stdout、环境变量或进程 I/O。
- CLI、Python、Node 和 WASM 适配层负责获取 bytes。
- 原生解析器在调用方进程内执行。需要崩溃隔离或 RSS 限制时，应把 CLI 放在受限的 worker/容器中运行。
- 所有解析器均不执行文档宏、notebook 代码、脚本、公式或内嵌二进制。

## 威胁与防御

| 威胁 | 防御措施 | 默认值 | 可配置 |
| --- | --- | --- | --- |
| 输入过大 | core 在检测/解析前检查输入字节量 | 64 MiB 共享解析预算 | `ParseLimits.max_parse_bytes`；CLI `--max-parse-mib` |
| ZIP 炸弹：条目过多 | 在中央目录检查阶段拒绝存档 | 10,000 条 | 无公开覆写接口 |
| ZIP 炸弹：单条目过大 | 拒绝声明或实测超大的条目 | 每条目 50 MiB | 无公开覆写接口 |
| ZIP 炸弹：压缩比异常 | 拒绝可疑的声明压缩比 | 200× | 无公开覆写接口 |
| ZIP 炸弹：总解压量膨胀 | 将声明未压缩大小累计计入解析预算 | 共享解析预算 | `max_parse_bytes` |
| 输出/token 耗尽 | CLI 截断 stdout，附加带内 marker 或 JSON warning | 256 KiB | CLI `--max-output-kib` |
| 加密/旧版 Office 混淆 | 在扩展名回退前拦截 OLE/CFB | 稳定错误 `legacy_or_encrypted_office` | 否 |
| 加密 PDF | 将解密失败映射为稳定错误 | 稳定错误 `encrypted_pdf` | 否 |
| 无文本无图片 PDF 幻觉风险 | 拒绝无文本层且无图片的 PDF，而非静默返回成功 | 稳定错误 `pdf_no_extractable_content` | 否 |
| 损坏的容器 | 拒绝无法读取的 ZIP 类格式 | 稳定错误 `invalid_container` | 否 |
| 未知解析失败或 Rust panic | 在所有公共 core 边界捕获 unwind，归一化为带 stage 的 `parse_failed` | 结构化 `SpoorError` | 否 |
| 解析无限/极慢 | 无进程内超时；调用方必须自行设置时限 | 不提供 | worker/容器/WASM host |
| 原生依赖 abort/segfault | 进程内不可恢复 | 不提供 | 对恶意多租户场景优先用 CLI worker 隔离 |

## 稳定失败契约

所有公共入口统一暴露 `code`、`reason`、`hint`、`recoverable` 和 `stage` 字段。消费者**必须按 `code` 分支**，不得依赖本地化的自然语言文本。当前稳定错误码：

- `pdf_no_extractable_content`
- `parse_budget_exceeded`
- `unsupported_format`
- `encrypted_pdf`
- `legacy_or_encrypted_office`
- `invalid_container`
- `parse_failed`

core、CLI、Python、Node 和 WASM 的测试路径均覆盖共享预算、无效容器、压缩炸弹和 CFB/OLE 拦截行为。

## 报告漏洞

**不要**为疑似漏洞创建公开 issue。请通过仓库的 Security 标签页提交私有安全通告，附上最小复现步骤、受影响版本和观察到的实际影响。
