"""模式①原生工具：把 spoor 直接写成 Tool，pyspoor 同进程调用。"""

from __future__ import annotations

import asyncio
import os

from ..provider import StaticProvider, Tool
from ..spoor_tools import SPOOR_TOOLS, run_spoor_tool
from ..tools_base import read_file_tool


def _make(defn: dict) -> Tool:
    async def execute(args: dict) -> str:
        # pyspoor 是同步原生扩展；丢到线程池避免阻塞事件循环。
        return await asyncio.to_thread(run_spoor_tool, defn["name"], args)

    return Tool(
        name=defn["name"],
        description=defn["description"],
        input_schema=defn["inputSchema"],
        execute=execute,
    )


def native_provider() -> StaticProvider:
    tools = [read_file_tool, *[_make(d) for d in SPOOR_TOOLS]]
    return StaticProvider(tools, transport=f"原生·同进程 pyspoor (pid={os.getpid()})")
