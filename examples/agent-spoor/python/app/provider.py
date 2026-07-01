"""能力供给层。三种接入模式各实现一份，agent 主循环只依赖它 ——
"内核不变，只换能力从哪来"。"""

from __future__ import annotations

from collections.abc import Awaitable, Callable
from dataclasses import dataclass


@dataclass
class Tool:
    name: str
    description: str
    input_schema: dict
    execute: Callable[[dict], Awaitable[str]]


def to_chat_tools(tools: list[Tool]) -> list[dict]:
    return [
        {
            "type": "function",
            "function": {"name": t.name, "description": t.description, "parameters": t.input_schema},
        }
        for t in tools
    ]


class ToolProvider:
    """基类。transport 只用于日志，让 native/mcp/skill 的差别可见。"""

    transport: str | None = None

    async def list_tools(self) -> list[dict]:
        raise NotImplementedError

    async def execute(self, name: str, args: dict) -> str:
        raise NotImplementedError

    def system_addendum(self) -> str | None:
        return None

    async def close(self) -> None:
        return None


class StaticProvider(ToolProvider):
    """由一组静态 Tool 组成的 provider（native 与 skill 用它）。"""

    def __init__(self, tools: list[Tool], transport: str | None = None, addendum: Callable[[], str] | None = None):
        self._tools = {t.name: t for t in tools}
        self.transport = transport
        self._addendum = addendum

    async def list_tools(self) -> list[dict]:
        return to_chat_tools(list(self._tools.values()))

    async def execute(self, name: str, args: dict) -> str:
        tool = self._tools.get(name)
        if tool is None:
            return f"不支持的工具: {name}"
        try:
            return await tool.execute(args)
        except Exception as e:  # noqa: BLE001 — 工具错误回给模型，不崩主循环
            return f"工具 {name} 执行失败: {e}"

    def system_addendum(self) -> str | None:
        return self._addendum() if self._addendum else None
