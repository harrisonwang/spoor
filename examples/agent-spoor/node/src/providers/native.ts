// 模式①原生工具：把 spoor 直接写成 Tool，@harrisonwang/spoor 同进程调用。
// 最紧耦合、最低延迟、最强类型；代价是绑死本 agent、要改 agent 代码。

import { staticProvider, type Tool, type ToolProvider } from "../provider.js";
import { runSpoorTool, SPOOR_TOOLS } from "../spoor-tools.js";
import { readFileTool } from "../tools/base.js";

// 把共享的 SPOOR_TOOLS 定义包成带 execute 的 Tool（execute 直连 runSpoorTool）。
const spoorTools: Tool[] = SPOOR_TOOLS.map((def) => ({
  name: def.name,
  description: def.description,
  input_schema: def.inputSchema,
  execute: (input) => runSpoorTool(def.name, input),
}));

export function nativeProvider(): ToolProvider {
  return staticProvider([readFileTool, ...spoorTools], {
    transport: `原生·同进程 Node binding (pid=${process.pid})`,
  });
}
