// 模式②MCP：agent 当 MCP client，把 spoor MCP server 的工具桥接进主循环。
// 松耦合、标准协议：同一个 server 也能插进 Claude Desktop / Cursor。引擎对 agent 透明。

import { fileURLToPath } from "node:url";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import type { ChatCompletionTool } from "openai/resources/chat/completions.js";
import { toChatTools, type ToolProvider } from "../provider.js";
import { readFileTool } from "../tools/base.js";

export async function mcpProvider(): Promise<ToolProvider> {
  const serverPath = fileURLToPath(new URL("../mcp/spoor-server.ts", import.meta.url));

  // 以子进程拉起 spoor MCP server（用本地 tsx 跑 .ts）。
  const transport = new StdioClientTransport({
    command: "npx",
    args: ["--no-install", "tsx", serverPath],
    cwd: process.cwd(),
    stderr: "inherit",
  });

  const client = new Client({ name: "mini-agent", version: "0.1.0" }, { capabilities: {} });
  await client.connect(transport);

  // 真实 server pid 由 server 自己在每次调用时打到 stderr（[spoor-mcp pid=…]）——
  // 那才是权威证据；这里的 transport.pid 可能是 npx 包装进程，故标签不放 pid 以免误导。
  const transportLabel = "MCP·独立 server 子进程（stdio 往返；真实 pid 见 [spoor-mcp] 日志）";

  const { tools: mcpTools } = await client.listTools();
  const baseChat = toChatTools([readFileTool]); // agent 自带的 read_file 仍在本地
  const mcpChat: ChatCompletionTool[] = mcpTools.map((t) => ({
    type: "function" as const,
    function: {
      name: t.name,
      description: t.description ?? "",
      parameters: (t.inputSchema ?? { type: "object", properties: {} }) as Record<string, unknown>,
    },
  }));

  return {
    transport: transportLabel,
    async listTools() {
      return [...baseChat, ...mcpChat];
    },
    async execute(name, input) {
      if (name === "read_file") return String(await readFileTool.execute(input));
      try {
        const res = await client.callTool({
          name,
          arguments: (input ?? {}) as Record<string, unknown>,
        });
        const content = (res.content ?? []) as Array<{ type: string; text?: string }>;
        const text = content.filter((c) => c.type === "text").map((c) => c.text ?? "").join("\n");
        return text || JSON.stringify(res);
      } catch (e) {
        return `MCP 工具 ${name} 调用失败: ${e instanceof Error ? e.message : String(e)}`;
      }
    },
    async close() {
      await client.close();
    },
  };
}
