"""入口：python -m app [--mode native|mcp|skill] [一次性问题…]"""

from __future__ import annotations

import argparse
import asyncio

from dotenv import load_dotenv

from .agent import Agent
from .provider import ToolProvider
from .providers.mcp_client import mcp_provider
from .providers.native import native_provider
from .providers.skill import skill_provider

EXIT_COMMANDS = {"exit", "quit", "q", "退出", ":q"}

MODE_NOTE = {
    "native": "原生工具（pyspoor，同进程）",
    "mcp": "MCP Server（独立进程，标准协议）",
    "skill": "Skill（SKILL.md + 受限 run_shell 调 spoor CLI）",
}


async def build_provider(mode: str) -> ToolProvider:
    if mode == "mcp":
        return await mcp_provider()
    if mode == "skill":
        return skill_provider()
    return native_provider()


async def run_once(mode: str, message: str) -> None:
    agent = Agent(await build_provider(mode))
    try:
        print(f"[mode] {MODE_NOTE[mode]}\n")
        print(f"\nAgent: {await agent.chat(message)}\n")
    finally:
        await agent.close()


async def run_repl(mode: str) -> None:
    agent = Agent(await build_provider(mode))
    print(f"mini-agent × spoor 已启动 —— 接入模式：{MODE_NOTE[mode]}")
    print("试试：'用 data/byd.pdf 第 1 页总结比亚迪 2024 关键财务' 或 'data/sales.csv 金额最高的三个分类'")
    print("输入 exit / quit / 退出 结束。\n")
    try:
        while True:
            try:
                user_input = await asyncio.to_thread(input, "你: ")
            except EOFError:
                break
            if not user_input.strip():
                continue
            if user_input.strip().lower() in EXIT_COMMANDS:
                break
            try:
                print(f"\nAgent: {await agent.chat(user_input)}\n")
            except Exception as e:  # noqa: BLE001
                print(f"\n错误: {e}\n")
    finally:
        await agent.close()
        print("\n再见！")


def main() -> None:
    load_dotenv()
    parser = argparse.ArgumentParser(prog="agent-spoor")
    parser.add_argument("--mode", default="native", choices=["native", "mcp", "skill"])
    parser.add_argument("prompt", nargs="*", help="一次性问题；留空进入 REPL")
    args = parser.parse_args()

    one_shot = " ".join(args.prompt).strip()
    if one_shot:
        asyncio.run(run_once(args.mode, one_shot))
    else:
        asyncio.run(run_repl(args.mode))


if __name__ == "__main__":
    main()
