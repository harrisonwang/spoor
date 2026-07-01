# agent-spoor · python

一个最小 AI Agent，用三种方式获得 spoor 的文档解析能力：**原生工具 / MCP Server / Skill**。
与 [`../node`](../node) 是同一套设计的 Python 实现。概念与对比见上层 [`../README.md`](../README.md)。

## 依赖

- Python ≥ 3.11、[`uv`](https://docs.astral.sh/uv/)
- 一个 OpenAI 兼容端点（OpenRouter / DeepSeek / z.ai / Cloudflare Workers AI …）
- **skill 模式**额外需要 `spoor` CLI 在 PATH（`brew install harrisonwang/tap/spoor`，或设 `SPOOR_BIN`）

## 起步

```bash
uv sync
cp .env.example .env       # 填 BASE_URL / OPENAI_API_KEY / OPENAI_MODEL

# 三种模式（REPL）——同一 agent，不同接入方式
uv run python -m app --mode native   # ① 原生工具（pyspoor 同进程）
uv run python -m app --mode mcp      # ② MCP Server（独立进程，标准协议）
uv run python -m app --mode skill    # ③ Skill（SKILL.md + 受限 run_shell 调 spoor CLI）

# 一次性提问
uv run python -m app --mode native "用 data/byd.pdf 第 1 页总结比亚迪 2024 的关键财务数据"
```

## 四个 demo 问题（三种模式结果一致）

1. `用 data/byd.pdf 第 1 页总结比亚迪 2024 的关键财务` — 按页收窄（`pages`）+ 页码作答
2. `data/sales.csv 里金额最高的三个分类是什么` — 表格分支 + 列筛选（`columns`）
3. `data/byd.pdf 有没有扫描件 / 无文本层的页` — agent 转达 `pdf_page_no_text_layer` 等 **warnings**
4. `把 data/with-image.docx 里的第一张图提取出来` — `spoor://` 占位符 → 提取到 `.spoor-media/`

## 怎么看出三种模式"真的"不同

native 与 mcp 可能都调名为 `read_document` 的工具（**故意同名同 schema**，让切模式对模型透明）。差别在**跑在哪**，日志已标出：

- **native**：`⟨跑在: 原生·同进程 pyspoor (pid=NNNN)⟩` —— pid 与 agent **相同**，同进程直调。
- **mcp**：`⟨跑在: MCP·独立 server 子进程…⟩`，并多出 **`[spoor-mcp pid=MMMM] ← 调用 read_document …`** —— **另一个进程**收到了 stdio 往返的调用。
- **skill**：工具名直接是 `run_shell`，底下 fork `spoor` CLI 子进程。

## 源码地图

```
app/
  __main__.py         入口（--mode + 异步 REPL）
  agent.py            主循环（只依赖 ToolProvider，三模式共用）
  model.py            AsyncOpenAI LLM 层
  provider.py         ToolProvider 基类 + StaticProvider
  spoor_tools.py      两个 spoor 工具的单一真相（schema + 派发），native 与 mcp server 共用
  tools_base.py       read_file
  shell.py            受限 run_shell（只放行 spoor）
  validate.py         safe_resolve + 参数强制转换
  providers/
    native.py         ① pyspoor 同进程
    mcp_client.py     ② MCP client：拉起 server、桥接工具
    skill.py          ③ 技能加载 + list_skills / read_skill / run_shell
  mcp_server/spoor_server.py  ② 独立 stdio MCP server（也能插别的 agent）
  skills/spoor/SKILL.md       ③ 技能卡
data/                 byd.pdf · sales.csv · with-image.docx
```

## 复用：把同一个 spoor MCP server 插进 Claude Desktop

`app/mcp_server/spoor_server.py` 不只服务本 demo。加进 `claude_desktop_config.json`：

```jsonc
{
  "mcpServers": {
    "spoor": {
      "command": "uv",
      "args": ["run", "python", "-m", "app.mcp_server.spoor_server"],
      "cwd": "/绝对路径/examples/agent-spoor/python"   // server 以此为根读文件
    }
  }
}
```

## 边界与说明

- **文件不出项目**：`safe_resolve` 把可读范围锁在 cwd。
- **skill 模式的 run_shell 只放行 `spoor …`**：无管道/重定向；`--extract` 的二进制输出由 run_shell 存到 `.spoor-media/`。
- **单次工具结果封顶 ~96 KiB**：超了截断并提示用 `pages/rows/columns/limit` 收窄。
- 引擎：native 与 mcp 走 pyspoor，skill 走 CLI；背后是同一套 spoor 引擎，只是耦合度不同。
