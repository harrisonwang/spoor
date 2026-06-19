# Python 确定性索引摄取示例

这个轻依赖 Python 示例把混合文档目录转换为确定性的 JSONL，可交给搜索索引、
向量数据库或后续 RAG 流水线。它刻意不包含 embedding、retrieval 和 Agent
行为，避免掩盖 spoor 的提取契约。

它展示：

- 使用原生 `pyspoor`，不产生子进程启动开销；
- 自动分派文档型与表格型结果；
- 按段落切分文档，并为表格输出 schema 与行记录；
- 内容派生的稳定 chunk ID；
- 局部失败、结构化错误与带位置的完整性 warnings；
- 确定性的 `chunks.jsonl` 和 `manifest.json`。

```bash
cd examples/rag-ingestion
python -m venv .venv
.venv/bin/python -m pip install "pyspoor>=0.8.3,<0.9"
.venv/bin/python ingest.py ./documents --output-dir ./spoor-index
```

常用参数：

```bash
.venv/bin/python ingest.py ./documents \
  --chunk-chars 1200 \
  --overlap-chars 120 \
  --max-parse-bytes 67108864
```

输出：

```text
spoor-index/
├── chunks.jsonl   # 每行一个文档块、表格 schema 或表格行
└── manifest.json  # 每个文件的格式、统计、警告与错误
```

默认按已知扩展名发现文件并跳过隐藏文件；`--all-files` 会把其他文件也交给
spoor 检测。每个文件默认使用 64 MiB 解析预算。XLSX/CSV 只摄取默认的前
100 条数据行，并在 manifest 中记录 `table_preview_truncated`；需要完整表格时，
直接给 `pyspoor` 的 `parse_bytes` / `parse_path` 传 `rows` / `limit` / `offset` /
`columns` / `sheet` 做分页摄取，无需再 shell 调用 CLI。

```bash
python test_ingest.py
```
