"""模式②MCP：agent 当 MCP client，把 spoor MCP server 的工具桥接进主循环。
松耦合、标准协议：同一个 server 也能插进 Claude Desktop / Cursor。引擎对 agent 透明。"""

from __future__ import annotations

import os
import sys
from contextlib import AsyncExitStack

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

from ..provider import ToolProvider
from ..tools_base import read_file_tool


class McpProvider(ToolProvider):
    def __init__(self):
        # 真实 server pid 由 server 自己在每次调用时打到 stderr（[spoor-mcp pid=…]）。
        self.transport = "MCP·独立 server 子进程（stdio 往返；真实 pid 见 [spoor-mcp] 日志）"
        self._stack = AsyncExitStack()
        self._session: ClientSession | None = None
        self._mcp_tools: list = []

    async def start(self) -> None:
        # 用当前解释器以子进程跑 spoor MCP server。
        params = StdioServerParameters(
            command=sys.executable,
            args=["-m", "app.mcp_server.spoor_server"],
            cwd=os.getcwd(),
            env=os.environ.copy(),
        )
        read, write = await self._stack.enter_async_context(stdio_client(params))
        session = await self._stack.enter_async_context(ClientSession(read, write))
        await session.initialize()
        self._session = session
        self._mcp_tools = (await session.list_tools()).tools

    async def list_tools(self) -> list[dict]:
        base = [
            {
                "type": "function",
                "function": {
                    "name": read_file_tool.name,
                    "description": read_file_tool.description,
                    "parameters": read_file_tool.input_schema,
                },
            }
        ]
        bridged = [
            {
                "type": "function",
                "function": {"name": t.name, "description": t.description or "", "parameters": t.inputSchema},
            }
            for t in self._mcp_tools
        ]
        return base + bridged

    async def execute(self, name: str, args: dict) -> str:
        if name == "read_file":
            return await read_file_tool.execute(args)
        assert self._session is not None
        result = await self._session.call_tool(name, args or {})
        texts = [c.text for c in result.content if getattr(c, "type", None) == "text"]
        return "\n".join(texts) if texts else str(result)

    async def close(self) -> None:
        await self._stack.aclose()


async def mcp_provider() -> McpProvider:
    provider = McpProvider()
    await provider.start()
    return provider
