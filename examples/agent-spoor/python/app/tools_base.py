"""agent 自带的基础工具：读纯文本/代码文件（与 spoor 的 read_document 形成对照）。"""

from __future__ import annotations

from .provider import Tool
from .validate import safe_resolve


async def _read_file(args: dict) -> str:
    file_path = args.get("file_path")
    if not isinstance(file_path, str):
        return "file_path 必须是字符串"
    try:
        with open(safe_resolve(file_path), encoding="utf-8") as f:
            content = f.read()
        return "\n".join(
            f"{str(i + 1).rjust(4)}: {line}" for i, line in enumerate(content.split("\n"))
        )
    except Exception as e:  # noqa: BLE001
        return f"读取文件失败: {e}"


read_file_tool = Tool(
    name="read_file",
    description="从当前项目读取一个文本文件，返回带行号的内容。二进制文档（PDF/Word/Excel…）请用 read_document。",
    input_schema={
        "type": "object",
        "properties": {
            "file_path": {"type": "string", "description": "相对项目根目录的文件路径，如 app/agent.py"},
        },
        "required": ["file_path"],
    },
    execute=_read_file,
)
