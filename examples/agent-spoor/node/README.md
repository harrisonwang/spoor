# agent-spoor · node

一个最小 AI Agent（改自 mini-agent），用三种方式获得 spoor 的文档解析能力：
**原生工具 / MCP Server / Skill**。概念与对比见上层 [`../README.md`](../README.md)。

## 依赖

- Node ≥ 18
- 一个 OpenAI 兼容端点（OpenRouter / DeepSeek / z.ai / Cloudflare Workers AI …）

## 起步

```bash
npm install
cp .env.example .env      # 填 BASE_URL / OPENAI_API_KEY / OPENAI_MODEL

# 三种模式（REPL）——同一 agent，不同接入方式
npm run native            # ① 原生工具（@harrisonwang/spoor 同进程）
npm run mcp               # ② MCP Server（独立进程，标准协议）
npm run skill             # ③ Skill（SKILL.md + 受限 run_shell 调 spoor CLI）

# 一次性提问（追加在 -- 之后）
npm run native -- "用 data/byd.pdf 第 1 页总结比亚迪 2024 的关键财务数据"
```

## 四个 demo 问题（三种模式结果一致）

在任意 `--mode` 下依次问：

1. `用 data/byd.pdf 第 1 页总结比亚迪 2024 的关键财务` — 触发按页收窄（`pages`）+ 页码作答
2. `data/sales.csv 里金额最高的三个分类是什么` — 表格分支 + 列筛选（`columns`）
3. `data/byd.pdf 有没有扫描件 / 无文本层的页` — agent 转达 `pdf_page_no_text_layer` 等 **warnings**
4. `把 data/with-image.docx 里的第一张图提取出来` — `spoor://` 占位符 → 提取到 `.spoor-media/`（交 VLM）

## 怎么看出三种模式"真的"不同

三种模式下 agent 可能都调用名为 `read_document` 的工具（native 与 mcp **故意同名同 schema**，好让切模式对模型透明）。差别在**工具跑在哪**，日志已标出来：

- **native**：`⟨跑在: 原生·同进程 Node binding (pid=NNNN)⟩` —— pid 和 agent 自己**相同**，即同进程直调 binding。
- **mcp**：`⟨跑在: MCP·独立 server 子进程（stdio 往返…）⟩`，且会多出一行 **`[spoor-mcp pid=MMMM] ← 调用 read_document …`** —— 这是**另一个进程**收到了 stdio 往返的调用（pid 与 agent 不同）。
- **skill**：工具名直接就是 `run_shell`，底下 fork 出 `spoor` CLI 子进程。

所以 native 与 mcp 看起来像"同一个工具"，是设计使然；它们的**执行进程不同**，看 pid 与 `[spoor-mcp]` 日志即可分辨。

## 三种模式怎么接的（源码地图）

```
src/
  agent.ts            主循环（只依赖 ToolProvider，三模式共用）
  model.ts            OpenAI 兼容 LLM 层
  provider.ts         ToolProvider 接口 + staticProvider
  spoor-tools.ts      两个 spoor 工具的单一真相（schema + 派发），native 与 mcp server 共用
  providers/
    native.ts         ① read_file + read_document + extract_document_image（binding 直连）
    mcp.ts            ② MCP client：拉起 server、桥接工具进主循环
    skill.ts          ③ 技能加载 + list_skills / read_skill / 受限 run_shell
  mcp/spoor-server.ts ② 独立 stdio MCP server（也能插别的 agent）
  skills/spoor/SKILL.md ③ 教模型用 spoor CLI 的技能卡
  util/{validate,shell}.ts  参数校验、safeResolve、受限 spoor 执行
data/                 byd.pdf（含无文本层页）· sales.csv · with-image.docx
```

## 复用：把同一个 spoor MCP server 插进 Claude Desktop

`src/mcp/spoor-server.ts` 不只服务本 demo。加进 `claude_desktop_config.json`，Claude Desktop 立刻会读本地文档：

```jsonc
{
  "mcpServers": {
    "spoor": {
      "command": "npx",
      "args": ["tsx", "/绝对路径/examples/agent-spoor/node/src/mcp/spoor-server.ts"],
      "cwd": "/你想让它能读的文档目录"     // safeResolve 以此为根，文档不出该目录
    }
  }
}
```

Cursor 等其它 MCP 客户端同理。

## 边界与说明

- **文件不出项目**：`safeResolve` 把可读范围锁在 agent 的 cwd（MCP server 则锁在其 `cwd`）——呼应 spoor 的本地处理。
- **skill 模式的 run_shell 只放行 `spoor …`**：无管道/重定向/子命令；`--extract` 的二进制输出由 run_shell 替你存到 `.spoor-media/`。
- **单次工具结果封顶 ~96 KiB**：超了会截断并提示用 `pages/rows/columns/limit` 收窄——正好演示 spoor 的收窄能力。
- **MCP 模式**用本地 `tsx` 以子进程跑 server（`npx --no-install tsx src/mcp/spoor-server.ts`）。
- 引擎：native 与 mcp 走 Node binding，skill 走 CLI；三者背后是同一套 spoor 引擎，只是耦合度不同。
