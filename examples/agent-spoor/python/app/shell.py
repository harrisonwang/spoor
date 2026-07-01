"""受限 run_shell：只放行单条 `spoor …`，无 shell 元字符，cwd 锁项目根。"""

from __future__ import annotations

import asyncio
import os
import re
import shlex

from .validate import safe_resolve

_FORBIDDEN = re.compile(r"[;&|`$><\n\r()]")


def _spoor_bin() -> str:
    return os.environ.get("SPOOR_BIN", "spoor")


async def run_shell(command: str) -> str:
    cmd = command.strip()
    if _FORBIDDEN.search(cmd):
        raise ValueError("命令含不允许的 shell 元字符（此工具只放行单条 spoor 命令，无管道/重定向）")
    argv = shlex.split(cmd)
    if not argv or argv[0] != "spoor":
        raise ValueError("run_shell 只放行 `spoor …` 命令")
    args = argv[1:]

    proc = await asyncio.create_subprocess_exec(
        _spoor_bin(),
        *args,
        cwd=os.getcwd(),
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    out, err = await proc.communicate()
    err_text = err.decode("utf-8", "replace")

    # --extract：stdout 是二进制媒体；skill 模式没有 > 重定向，这里替它存文件。
    if "--extract" in args:
        if proc.returncode != 0:
            return f"spoor 提取失败（exit {proc.returncode}）:\n{err_text.strip()}"
        i = args.index("--extract")
        uri = args[i + 1] if i + 1 < len(args) else "media"
        out_dir = safe_resolve(".spoor-media")
        os.makedirs(out_dir, exist_ok=True)
        name = re.sub(r"[^a-zA-Z0-9._-]", "_", uri).lstrip("_")[-48:] or "media"
        with open(os.path.join(out_dir, name), "wb") as f:
            f.write(out)
        return f"已提取内嵌资源 → .spoor-media/{name}（{len(out)} bytes）。可交给 VLM。"

    text = out.decode("utf-8", "replace").rstrip()
    warn = f"\n\n〔stderr / warnings〕\n{err_text.strip()}" if err_text.strip() else ""
    if proc.returncode != 0 and not text:
        return f"spoor 执行失败（exit {proc.returncode}）:\n{err_text.strip()}"
    return text + warn
