import type { ChatCompletionTool } from "openai/resources/chat/completions.js";

/** 一个工具：给 LLM 看的 schema + 真正执行的函数 */
export type Tool = {
  name: string;
  description: string;
  input_schema: {
    type: "object";
    properties: Record<string, unknown>;
    required?: string[];
  };
  execute: (input: unknown) => Promise<unknown>;
};

/**
 * 能力供给层。三种接入模式（native / mcp / skill）各实现一份，
 * agent 主循环只依赖这个接口 —— "内核不变，只换能力从哪来"。
 */
export interface ToolProvider {
  /** 给模型的工具 schema（OpenAI function 格式） */
  listTools(): Promise<ChatCompletionTool[]>;
  /** 按名字执行一次工具调用，返回给模型看的文本 */
  execute(name: string, input: unknown): Promise<string>;
  /** 工具"跑在哪"的短标签，仅用于日志，让 native/mcp/skill 的差别可见。 */
  transport?: string;
  /** 可选：往系统提示追加内容（skill 模式注入技能目录） */
  systemAddendum?(): string;
  /** 可选：清理（mcp 模式断开子进程） */
  close?(): Promise<void>;
}

export function toChatTools(tools: Tool[]): ChatCompletionTool[] {
  return tools.map(({ name, description, input_schema }) => ({
    type: "function" as const,
    function: { name, description, parameters: input_schema },
  }));
}

/** 由一组静态 Tool 组成的 provider（native 与 skill 用它） */
export function staticProvider(
  tools: Tool[],
  opts: { transport?: string; systemAddendum?: () => string } = {},
): ToolProvider {
  return {
    transport: opts.transport,
    async listTools() {
      return toChatTools(tools);
    },
    async execute(name, input) {
      const tool = tools.find((t) => t.name === name);
      if (!tool) return `不支持的工具: ${name}`;
      try {
        const result = await tool.execute(input);
        return typeof result === "string" ? result : JSON.stringify(result);
      } catch (e) {
        return `工具 ${name} 执行失败: ${e instanceof Error ? e.message : String(e)}`;
      }
    },
    ...(opts.systemAddendum ? { systemAddendum: opts.systemAddendum } : {}),
  };
}
