# AWS Lambda：CLI 单二进制装进 Layer，子进程调用

**本示例唯一证明：把 `spoor` CLI 单二进制放进 AWS Lambda Layer（`/opt/bin/spoor`），handler 以子进程方式调用——这是 spoor 的第三种交付形态（既非 WASM、也非链接库，而是无依赖的 CLI 单二进制）在 serverless 的用法。** 解析继承 CLI 的内存/输出上限，与运行时语言无关。

该 handler 期望 Linux 版 `spoor` 位于 Lambda Layer 的 `/opt/bin/spoor`。
调用参数形如：

```json
{ "filename": "报告.pdf", "body": "...", "isBase64Encoded": true }
```

解析运行在独立 CLI 子进程中，继承 CLI 的 64 MiB 共享解析内存上限与 256 KiB
输出上限。Lambda 自身的请求 payload、临时磁盘、内存和超时限制仍需由部署方
配置；大文件更适合通过 S3 事件传递对象位置，而不是直接放入同步请求。

本地集成测试：

```bash
cargo build -p spoor-cli
SPOOR_BIN="$PWD/target/debug/spoor" npm --prefix examples/serverless-lambda test
```
