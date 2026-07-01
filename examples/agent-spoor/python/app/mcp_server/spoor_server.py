"""模式②的服务端：一个独立的 spoor MCP Server（stdio）。
把它配进 Claude Desktop / Cursor，那些 agent 也立刻能读本地文档。
引擎用 pyspoor（同进程），但对 MCP 客户端透明、可随时替换成 CLI/WASM。

单独跑：`python -m app.mcp_server.spoor_server`
"""

from __future__ import annotations

import asyncio
import os
import sys

import mcp.types as types
from mcp.server.lowlevel import Server
from mcp.server.stdio import stdio_server

from ..spoor_tools import SPOOR_TOOLS, run_spoor_tool

server = Server("spoor")


@server.list_tools()
async def list_tools() -> list[types.Tool]:
    return [
        types.Tool(name=t["name"], description=t["description"], inputSchema=t["inputSchema"])
        for t in SPOOR_TOOLS
    ]


@server.call_tool()
async def call_tool(name: str, arguments: dict | None) -> list[types.TextContent]:
    # 日志走 stderr（stdout 归协议）：让你在 MCP 模式下亲眼看到"独立 server 进程"收到了调用。
    print(f"[spoor-mcp pid={os.getpid()}] ← 调用 {name} {arguments or {}}", file=sys.stderr, flush=True)
    text = await asyncio.to_thread(run_spoor_tool, name, arguments or {})
    return [types.TextContent(type="text", text=text)]


async def main() -> None:
    async with stdio_server() as (read, write):
        print(f"[spoor-mcp pid={os.getpid()}] server ready on stdio", file=sys.stderr, flush=True)
        await server.run(read, write, server.create_initialization_options())


if __name__ == "__main__":
    asyncio.run(main())
