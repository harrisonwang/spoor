# Core 与 Python 绑定架构

## 结论

第二阶段已采用 Cargo workspace，并交付 PyO3、napi-rs 与 WASM 绑定。拆分依据是稳定的嵌入边界，而非未经测量的"子进程更慢"假设；基准与体积脚本位于 `benchmarks/`。

目标不是维护两套产品，而是维护一个确定性的 Rust 引擎和多个薄适配层：

```text
spoor/
├── Cargo.toml
├── crates/
│   ├── spoor-core/       # 检测、解析、数据量限制、结构化结果/错误
│   └── spoor-cli/        # clap、glob、stdin、URL、stdout/stderr、exit code
└── bindings/
    └── python/           # PyO3 extension + 薄 Python 封装层
```

Rust packages 为 `spoor-core`、`spoor-cli`，CLI binary 为 `spoor`。Python distribution 为 `pyspoor`，Node 为 `@harrisonwang/spoor`，WASM 为 `@harrisonwang/spoor-wasm`。

## Core 边界

`spoor-core` 当前满足：

- 接受 bytes 和显式 metadata（source name、content type、format hint）。
- 负责 format 检测、解析、限制单次解析的数据量、ZIP 安全检查，以及返回结构化结果。
- 通过格式无关的 `extract_media` 按安全资源 URI 提取单个内嵌媒体；当前实现 `spoor://{pdf,docx,pptx}/...`。
- 不依赖 `clap`、glob、stdin/stdout/stderr、进程退出和网络请求。
- 不依赖 Python，也不把 CLI 字符串错误暴露为 API 契约。
- 保持每次调用独立，不使用可变全局状态，便于并发和测试。
- PDF 内存提取器已移除上游直接 stdout/stderr 诊断与 path API；core 不重定向或写入宿主进程流。

核心 API：

```rust
pub struct ParseRequest<'a> {
    pub bytes: &'a [u8],
    pub source_name: Option<&'a str>,
    pub content_type: Option<&'a str>,
    pub format_hint: Option<Format>,
    pub table_filter: TableFilter,
    pub limits: ParseLimits,
}

pub enum ParseContent {
    Document(DocumentResult),
    Tables(TableResult),
}

pub struct ParseResult {
    pub content: ParseContent,
    pub warnings: Vec<SpoorWarning>,
    pub stats: ParseStats,
}

pub struct SpoorWarning {
    pub code: WarningCode,
    pub message: String,
    pub location: Option<WarningLocation>,
}

pub struct SpoorError {
    pub code: ErrorCode,
    pub reason: String,
    pub hint: String,
    pub recoverable: bool,
    pub stage: Option<ParseStage>,
}

pub fn extract_media(
    request: &ParseRequest<'_>,
    resource: &str,
) -> Result<Vec<u8>, SpoorError>;
```

`max_parse_bytes` 属于 core，也约束单资源媒体提取。`max_output_bytes`、Markdown truncation marker、stderr warning 数量和 exit code 属于 CLI adapter；二进制 `--extract` 不应用文本输出截断。Python 应优先拿到结构化结果，而不是被迫消费截断后的 CLI 字符串。Agent 应调用 `parse` 或 `parse_document_result` 并处理 warnings；`parse_document` 是只取 Markdown 的兼容便捷接口。

路径读取可由 CLI/Python adapter 提供 `parse_path` 便捷 API；URL、glob 和 stdin 保持在 CLI 层。这样 core 保持离线、确定性，也避免绑定层继承爬虫策略。

## 错误契约

公共边界已经从 `anyhow::Result` 收敛到稳定的 `Result<T, SpoorError>`。内部仍用 `anyhow` 补充上下文，出口统一规范化为结构化错误。

CLI 已实现的稳定错误码（core 化时原样继承）：

- `image_only_pdf`
- `parse_budget_exceeded`
- `unsupported_format`
- `encrypted_pdf`
- `legacy_or_encrypted_office`
- `invalid_container`
- `parse_failed`

后续按需补充（如 `archive_safety_limit`），新增即文档化并纳入 doc-sync 测试。

CLI 将 `SpoorError` 渲染为当前机器可读 JSON；Python exception 保存同样字段。Agent 因此不需要解析自然语言错误信息。

## PyO3 接口层

Python distribution 使用私有 native module + 公共 Python 封装层：

```text
bindings/python/
├── Cargo.toml
├── pyproject.toml        # maturin
├── src/lib.rs            # module: spoor._native
└── spoor/
    ├── __init__.py
    └── models.py         # 稳定 dataclass / exception 封装层
```

首版 Python API 保持精简：

```python
parse_bytes(data, *, source_name=None, format=None, max_parse_bytes=None)
parse_path(path, *, format=None, max_parse_bytes=None)
detect_format(data, *, source_name=None, content_type=None)
```

已实现：

- parsing 使用 `py.detach(...)` 释放 GIL。
- Rust panic 不跨 FFI；所有失败映射为带字段的 Python exception。
- 不强制 JSON round-trip；公共 Python 封装层返回 dataclass / 类型化对象。
- 不宣传 zero-copy。Python buffer 到 Rust 可以减少一次复制，但各解析器仍可能分配中间结构。
- 并发限制交给调用方 semaphore；core 保持 per-call limits，不内置隐式全局线程池。

## 并发与隔离

PyO3 不是高并发场景的自动最优解。它消除了进程启动和 stdout/JSON IPC 开销，但 native parser 的 OOM、abort、segfault 或依赖缺陷也会直接影响 Python 编排器进程。结构化错误和 `max_parse_bytes` 能处理可预期失败，但不能替代进程级故障隔离。

建议保留两种运行模式：

- **进程内 PyO3**：可信输入、低延迟、小文件高频调用。
- **长期 worker 进程池**：不可信输入、大文件、多租户或需要 OS/container memory limit 的场景；复用 worker，避免每文件冷启动。

性能基准应比较 CLI 冷启动、长期 worker IPC 和 PyO3 暖调用三者，而不是只比较"每文件启动一次 CLI"与 PyO3。

## 迁移顺序

1. **稳定接口层**：引入 `ParseRequest`、`ParseResult`、`SpoorError`、`ParseLimits`；补齐错误码和等价测试。
2. **保持行为不变的代码拆分**：移动解析模块到 `crates/spoor-core`；`spoor-cli` 仅调用 core；现有 CLI 快照必须不变。
3. **PyO3 MVP**：使用 maturin 发布私有 native module，提供 `parse_bytes` / `parse_path`，复用同一套测试用例集。
4. **按测量优化**：只有 benchmark 证明必要时，再做 reader/streaming API、减少复制或专门的并发调度。

## 验收与基准

拆分前后必须同时满足：

- CLI 全量测试用例和快照的等价性。
- Rust core 与 Python 绑定对同一测试用例返回等价的结构化结果/错误。
- image-only PDF、数据量限制、ZIP safety 和截断契约有跨入口测试。
- benchmark 分开报告冷启动、暖调用、解析耗时、峰值 RSS 和并发吞吐。
- PyO3 相对 CLI 的收益必须在"小文件高频调用"场景可测；大 PDF/DOCX 通常由解析成本主导，不能把收益归因于绑定。

建议发布叙事：

> 一个确定性的 Rust 文档引擎，提供 CLI 和 Python 包两种形态，具备稳定的分页边界、机器可读的失败信息和有上限的输出/解析预算。

这比"Python 版更快"更稳健，也不会削弱单二进制、离线、无 OCR/云依赖的现有定位。
