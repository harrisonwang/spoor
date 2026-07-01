# agent-spoor

把 **spoor** 接入一个最小 AI Agent 的**三种方式**，放在同一个 agent 内核上对照：

| 模式 | 接入方式 | 用的 spoor 形态 | 进程 | 谁能复用 | 改 agent 代码 | 何时选它 |
| --- | --- | --- | --- | --- | --- | --- |
| ① **原生工具** | 直接写成 `Tool` | Node binding `@harrisonwang/spoor`（同进程） | 同进程 | 只这个 agent | 要 | 要最低延迟、最强类型、最紧集成 |
| ② **MCP Server** | agent 当 MCP client | Node binding（独立 MCP server 进程） | 独立 stdio | **任意 MCP agent**（本 agent / Claude Desktop / Cursor） | 只加 client，不碰核心 | 要标准化、跨工具复用 |
| ③ **Skill** | 丢一份 `SKILL.md` | CLI `spoor`（子进程） | 子进程 | 任意能跑 shell 的 agent | 零改逻辑 | 要渐进式扩展、非工程师也能加 |

三种模式**共享同一个 agent 内核**（同一循环、同一 LLM 层），差别只在一层 `ToolProvider`——一条命令 `--mode native|mcp|skill` 切换。同样的四个 demo 问题，三种模式跑出**一样的结果**；差异只在"文档能力是怎么进来的"。这正好把 spoor 的"一套引擎、多形态"落成开发者最关心的接入决策题。

## 目录命名规则

按 `agent-spoor/<语言>/` 组织，为多语言实现预留：

```
agent-spoor/
├── README.md      ← 概念与三模式对比（本文件）
├── node/          ✅ 已实现（TypeScript，@harrisonwang/spoor + CLI）
├── python/        ✅ 已实现（pyspoor + CLI）
└── rust/          ✅ 已实现（spoor-core + rmcp + CLI）
```

每个语言目录是一个自包含、可独立跑的实现，暴露相同的三种模式与同样的 demo。三种模式各对应一种 spoor 交付形态，Rust 版尤其直白：native 直接链 `spoor-core`（源头）、MCP 用 `rmcp` 写 Rust server、skill 走 `spoor` CLI。

## 从哪开始

- **Node**：[`node/README.md`](node/README.md) —— `npm run native|mcp|skill`
- **Python**：[`python/README.md`](python/README.md) —— `uv run python -m app --mode native|mcp|skill`
- **Rust**：[`rust/README.md`](rust/README.md) —— `cargo run -- --mode native|mcp|skill`

三者都附把同一个 spoor MCP server 插进 Claude Desktop 的配置。
