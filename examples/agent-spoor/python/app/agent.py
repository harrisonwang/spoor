"""Agent 主循环（异步）。只依赖 ToolProvider —— 三种模式共用。"""

from __future__ import annotations

import json

from .model import call_model
from .provider import ToolProvider

SYSTEM_PROMPT = """你是一个能读本地文档的智能体，运行在当前项目目录中。
- 查看纯文本/代码文件用 read_file；读取 PDF/Word/Excel/PPT/EPUB/网页等文档时，用文档工具或技能，不要臆测内容。
- 文档解析可能带完整性 warnings（如某页是扫描件、无文本层），要如实转达用户。
- 回答简洁、准确；涉及具体数字时尽量给出处（页码）。"""


class Agent:
    def __init__(self, provider: ToolProvider):
        self.provider = provider
        add = provider.system_addendum()
        content = f"{SYSTEM_PROMPT}\n\n{add}" if add else SYSTEM_PROMPT
        self.messages: list[dict] = [{"role": "system", "content": content}]
        self._tools: list[dict] | None = None

    async def chat(self, user_message: str) -> str:
        if self._tools is None:
            self._tools = await self.provider.list_tools()
        self.messages.append({"role": "user", "content": user_message})

        while True:
            message, usage = await call_model(self.messages, self._tools)

            # 把 assistant 消息（含 tool_calls）显式拼回历史。
            assistant: dict = {"role": "assistant", "content": message.content}
            tool_calls = message.tool_calls or []
            if tool_calls:
                assistant["tool_calls"] = [
                    {
                        "id": c.id,
                        "type": "function",
                        "function": {"name": c.function.name, "arguments": c.function.arguments},
                    }
                    for c in tool_calls
                ]
            self.messages.append(assistant)

            if not tool_calls:
                if usage:
                    print(f"[tokens] 输入: {usage.prompt_tokens}, 输出: {usage.completion_tokens}")
                return message.content or ""

            for call in tool_calls:
                if call.type != "function":
                    continue
                name = call.function.name
                try:
                    args = json.loads(call.function.arguments)
                except json.JSONDecodeError:
                    args = {}

                via = f"  ⟨跑在: {self.provider.transport}⟩" if self.provider.transport else ""
                print(f"\n🔧 调用工具: {name}{via}")
                print(f"  参数: {json.dumps(args, ensure_ascii=False)}")

                result = await self.provider.execute(name, args)
                preview = result[:300] + "\n..." if len(result) > 300 else result
                print(f"  结果: {preview}")

                self.messages.append({"role": "tool", "tool_call_id": call.id, "content": result})

    async def close(self) -> None:
        await self.provider.close()
