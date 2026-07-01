"""参数校验与相对路径解析（呼应 spoor 的本地处理：文件不出项目）。"""

from __future__ import annotations

import os


def safe_resolve(user_path: str) -> str:
    """把相对路径解析成项目内绝对路径；拒绝 ../ 越界。"""
    root = os.path.realpath(os.getcwd())
    resolved = os.path.realpath(os.path.join(root, user_path))
    if resolved != root and not resolved.startswith(root + os.sep):
        raise ValueError(f"路径在项目外: {user_path}")
    return resolved


# —— 宽松强制转换（模型给的参数可能缺省或类型不对，取不到就当没传）——
def opt_str(v: object) -> str | None:
    return v if isinstance(v, str) else None


def opt_num(v: object) -> float | None:
    return v if isinstance(v, (int, float)) and not isinstance(v, bool) else None


def pair(v: object) -> tuple[int, int] | None:
    if (
        isinstance(v, (list, tuple))
        and len(v) == 2
        and all(isinstance(x, (int, float)) and not isinstance(x, bool) for x in v)
    ):
        return (int(v[0]), int(v[1]))
    return None


def str_arr(v: object) -> list[str] | None:
    if isinstance(v, (list, tuple)) and all(isinstance(x, str) for x in v):
        return list(v)
    return None
