import type {
  ChatCompletionMessageParam,
  ChatCompletionTool,
} from "openai/resources/chat/completions.js";
import { callModel } from "./model.js";
import type { ToolProvider } from "./provider.js";

const SYSTEM_PROMPT = `你是一个能读本地文档的智能体，运行在当前项目目录中。
- 查看纯文本/代码文件用 read_file；读取 PDF/Word/Excel/PPT/EPUB/网页等文档时，用文档工具或技能，不要臆测内容。
- 文档解析可能带完整性 warnings（如某页是扫描件、无文本层），要如实转达用户。
- 回答简洁、准确；涉及具体数字时尽量给出处（页码）。`;

function throwIfAborted(signal: AbortSignal): void {
  if (!signal.aborted) return;
  const err = new Error("The operation was aborted");
  err.name = "AbortError";
  throw err;
}

export class Agent {
  private messages: ChatCompletionMessageParam[];
  private tools: ChatCompletionTool[] | null = null;
  private abortController: AbortController | null = null;

  constructor(private provider: ToolProvider) {
    const add = provider.systemAddendum?.();
    this.messages = [
      { role: "system", content: add ? `${SYSTEM_PROMPT}\n\n${add}` : SYSTEM_PROMPT },
    ];
  }

  abort(): boolean {
    if (!this.abortController) return false;
    this.abortController.abort();
    return true;
  }

  async close(): Promise<void> {
    await this.provider.close?.();
  }

  async chat(userMessage: string): Promise<string> {
    if (!this.tools) this.tools = await this.provider.listTools();
    this.abortController = new AbortController();
    const { signal } = this.abortController;

    try {
      this.messages.push({ role: "user", content: userMessage });

      while (true) {
        throwIfAborted(signal);

        const { message, usage } = await callModel(this.messages, this.tools, signal);
        this.messages.push(message);

        const toolCalls = message.tool_calls ?? [];
        if (toolCalls.length === 0) {
          if (usage) {
            console.log(`[tokens] 输入: ${usage.input_tokens}, 输出: ${usage.output_tokens}`);
          }
          return message.content ?? "";
        }

        for (const call of toolCalls) {
          throwIfAborted(signal);
          if (call.type !== "function") continue;

          const name = call.function.name;
          let input: unknown;
          try {
            input = JSON.parse(call.function.arguments);
          } catch {
            input = {};
          }

          const via = this.provider.transport ? `  ⟨跑在: ${this.provider.transport}⟩` : "";
          console.log(`\n🔧 调用工具: ${name}${via}`);
          console.log(`  参数: ${JSON.stringify(input)}`);

          const result = await this.provider.execute(name, input);
          const preview = result.length > 300 ? result.slice(0, 300) + "\n..." : result;
          console.log(`  结果: ${preview}`);

          this.messages.push({ role: "tool", tool_call_id: call.id, content: result });
        }
      }
    } finally {
      this.abortController = null;
    }
  }
}
