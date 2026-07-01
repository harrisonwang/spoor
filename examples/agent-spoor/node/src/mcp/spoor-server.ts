// 模式②的服务端：一个独立的 spoor MCP Server（stdio）。
// 它不只服务本 demo —— 把它配进 Claude Desktop / Cursor，那些 agent 也立刻能读本地文档。
// 引擎用 @harrisonwang/spoor（同进程），但对 MCP 客户端完全透明、可随时替换成 CLI/WASM。

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { CallToolRequestSchema, ListToolsRequestSchema } from "@modelcontextprotocol/sdk/types.js";
import { runSpoorTool, SPOOR_TOOLS } from "../spoor-tools.js";

const server = new Server(
  { name: "spoor", version: "0.1.0" },
  { capabilities: { tools: {} } },
);

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: SPOOR_TOOLS.map((t) => ({
    name: t.name,
    description: t.description,
    inputSchema: t.inputSchema,
  })),
}));

server.setRequestHandler(CallToolRequestSchema, async (req) => {
  const { name, arguments: args } = req.params;
  // 日志走 stderr（stdout 归协议）：让你在 MCP 模式下亲眼看到"独立 server 进程"收到了调用。
  console.error(`[spoor-mcp pid=${process.pid}] ← 调用 ${name} ${JSON.stringify(args ?? {})}`);
  try {
    const text = await runSpoorTool(name, args ?? {});
    return { content: [{ type: "text", text }] };
  } catch (e) {
    return {
      content: [{ type: "text", text: `错误: ${e instanceof Error ? e.message : String(e)}` }],
      isError: true,
    };
  }
});

const transport = new StdioServerTransport();
await server.connect(transport);
// stdout 归 MCP 协议专用；日志只能走 stderr。
console.error("[spoor-mcp] server ready on stdio");
