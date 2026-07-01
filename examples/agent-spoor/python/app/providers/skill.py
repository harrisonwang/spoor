"""模式③Skill：不写类型化工具，丢一份 SKILL.md，用受限 run_shell 驱动 spoor CLI。
最松耦合、零改 agent 逻辑；渐进式披露（系统提示只给技能目录，用到才 read_skill）。"""

from __future__ import annotations

import re
from pathlib import Path

from ..provider import StaticProvider, Tool
from ..shell import run_shell
from ..tools_base import read_file_tool

_SKILLS_DIR = Path(__file__).resolve().parent.parent / "skills"


def _parse_skill(dir_name: str, raw: str) -> dict:
    name, description, body = dir_name, "", raw
    m = re.match(r"^---\n(.*?)\n---\n?(.*)$", raw, re.DOTALL)
    if m:
        body = m.group(2)
        for line in m.group(1).splitlines():
            kv = re.match(r"^(\w+):\s*(.*)$", line)
            if not kv:
                continue
            if kv.group(1) == "name":
                name = kv.group(2).strip()
            elif kv.group(1) == "description":
                description = kv.group(2).strip()
    return {"name": name, "description": description, "body": body}


def _load_skills() -> list[dict]:
    if not _SKILLS_DIR.exists():
        return []
    cards = []
    for entry in sorted(_SKILLS_DIR.iterdir()):
        skill_md = entry / "SKILL.md"
        if entry.is_dir() and skill_md.exists():
            cards.append(_parse_skill(entry.name, skill_md.read_text(encoding="utf-8")))
    return cards


def skill_provider() -> StaticProvider:
    skills = _load_skills()
    by_name = {s["name"]: s for s in skills}
    catalog = "\n".join(f"- {s['name']}: {s['description']}" for s in skills) or "（无可用技能）"

    async def list_skills(_args: dict) -> str:
        return catalog

    async def read_skill(args: dict) -> str:
        s = by_name.get(args.get("name", ""))
        return s["body"].strip() if s else "没有该名字的技能"

    async def run_shell_tool(args: dict) -> str:
        command = args.get("command")
        if not isinstance(command, str):
            return "command 必须是字符串"
        return await run_shell(command)

    tools = [
        read_file_tool,
        Tool("list_skills", "列出可用技能（名字 + 简介）。", {"type": "object", "properties": {}}, list_skills),
        Tool(
            "read_skill",
            "读取某个技能的完整说明（SKILL.md 正文），据此决定怎么用 run_shell。",
            {"type": "object", "properties": {"name": {"type": "string", "description": "技能名，如 spoor"}}, "required": ["name"]},
            read_skill,
        ),
        Tool(
            "run_shell",
            "执行一条命令（本 demo 只放行 `spoor …`，无管道/重定向）。按技能说明调用 spoor CLI。",
            {
                "type": "object",
                "properties": {"command": {"type": "string", "description": "如 spoor data/byd.pdf --pages 1:1"}},
                "required": ["command"],
            },
            run_shell_tool,
        ),
    ]

    def addendum() -> str:
        return (
            "你有以下**技能**可用（渐进式披露：先用 read_skill 读全文，再按其说明用 run_shell 执行）：\n"
            f"{catalog}\n处理非纯文本文档时，优先查看 spoor 技能。"
        )

    return StaticProvider(tools, transport="Skill·spoor CLI 子进程", addendum=addendum)
