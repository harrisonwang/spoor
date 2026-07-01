# agent-spoor · rust

一个最小 AI Agent，用三种方式获得 spoor 的文档解析能力：**原生工具 / MCP Server / Skill**。
与 [`../node`](../node)、[`../python`](../python) 是同一套设计的 Rust 实现。概念与对比见上层 [`../README.md`](../README.md)。

> 这是**自包含**的示例，直接依赖 `spoor-core`（同仓库路径依赖）。它不进 spoor 主 workspace（自带 tokio/reqwest/rmcp 等重依赖），有独立 `Cargo.lock`。
> LLM 层不复用 `llm-core`——那是流式 chat 客户端、不支持工具调用；这里自带一个最小的支持 tools 的 reqwest 客户端。

## 依赖

- Rust（cargo）
- 一个 OpenAI 兼容端点（OpenRouter / DeepSeek / z.ai / Cloudflare Workers AI …）
- **skill 模式**额外需要 `spoor` CLI 在 PATH（`brew install harrisonwang/tap/spoor`，或设 `SPOOR_BIN`）

## 起步

```bash
cp .env.example .env       # 填 BASE_URL / OPENAI_API_KEY / OPENAI_MODEL

# 三种模式（REPL）——同一 agent，不同接入方式
cargo run -- --mode native   # ① 原生工具（spoor-core 同进程）
cargo run -- --mode mcp      # ② MCP Server（独立进程，标准协议）
cargo run -- --mode skill    # ③ Skill（SKILL.md + 受限 run_shell 调 spoor CLI）

# 一次性提问
cargo run -- --mode native "用 data/byd.pdf 第 1 页总结比亚迪 2024 的关键财务数据"

# 单独跑 MCP server（供 Claude Desktop 用）
cargo run --bin spoor-mcp-server
```

测试（无需 LLM，覆盖三条 provider 路径）：`cargo test`

## 四个 demo 问题（三种模式结果一致）

1. `用 data/byd.pdf 第 1 页总结比亚迪 2024 的关键财务` — 按页收窄（`pages`）+ 页码作答
2. `data/sales.csv 里金额最高的三个分类是什么` — 表格分支 + 列筛选（`columns`）
3. `data/byd.pdf 有没有扫描件 / 无文本层的页` — agent 转达 `pdf_page_no_text_layer` 等 **warnings**
4. `把 data/with-image.docx 里的第一张图提取出来` — `spoor://` 占位符 → 提取到 `.spoor-media/`

## 怎么看出三种模式"真的"不同

native 与 mcp 可能都调名为 `read_document` 的工具（**故意同名同 schema**，让切模式对模型透明）。差别在**跑在哪**，日志已标出：

- **native**：`⟨跑在: 原生·同进程 spoor-core (pid=NNNN)⟩` —— pid 与 agent **相同**。
- **mcp**：`⟨跑在: MCP·独立 server 子进程…⟩`，并多出 **`[spoor-mcp pid=MMMM] ← 调用 read_document …`** —— **另一个进程**收到了 stdio 往返的调用。
- **skill**：工具名直接是 `run_shell`，底下 fork `spoor` CLI 子进程。

## 源码地图

```
src/
  main.rs             入口（--mode + REPL）
  bin/spoor-mcp-server.rs  ② 独立 MCP server 二进制（也能插别的 agent）
  agent.rs            主循环（只依赖 ToolProvider）
  model.rs            自带的支持 tools 的 reqwest 客户端（llm-core 不支持工具调用）
  provider.rs         ToolProvider trait
  spoor_tools.rs      两个 spoor 工具的单一真相（schema + 派发，spoor-core 直调），native 与 mcp server 共用
  tools_base.rs       read_file
  shell.rs            受限 run_shell（只放行 spoor）
  validate.rs         safe_resolve + 参数取值
  mcp_server.rs       ② rmcp ServerHandler（list_tools / call_tool）
  providers/{native,mcp,skill}.rs  三种接入实现
  skills/spoor/SKILL.md            ③ 技能卡（编译期 include_str! 内嵌）
data/                 byd.pdf · sales.csv · with-image.docx
```

## 复用：把 spoor MCP server 插进 Claude Desktop

先构建 release 二进制：

```bash
cargo build --release --bin spoor-mcp-server
```

再加进 `claude_desktop_config.json`：

```jsonc
{
  "mcpServers": {
    "spoor": {
      "command": "/绝对路径/examples/agent-spoor/rust/target/release/spoor-mcp-server",
      "cwd": "/你想让它能读的文档目录"     // safe_resolve 以此为根，文档不出该目录
    }
  }
}
```

## 边界与说明

- **文件不出项目**：`safe_resolve` 把可读范围锁在进程 cwd（MCP server 则锁在其 `cwd`）。
- **skill 模式的 run_shell 只放行 `spoor …`**：无管道/重定向；`--extract` 的二进制输出由 run_shell 存到 `.spoor-media/`。
- **单次工具结果封顶 ~96 KiB**：超了截断并提示用 `pages/rows/columns/limit` 收窄。
- 引擎：native 与 mcp 走 `spoor-core`（源头），skill 走 CLI；背后是同一套 spoor 引擎，只是耦合度不同。
