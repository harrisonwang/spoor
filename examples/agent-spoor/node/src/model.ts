import { OpenAI } from "openai";
import type {
  ChatCompletionMessageParam,
  ChatCompletionTool,
} from "openai/resources/chat/completions.js";

function requireEnv(name: string): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(`缺少环境变量 ${name}，请在 .env 文件中设置`);
  }
  return value;
}

const client = new OpenAI({
  baseURL: requireEnv("BASE_URL"),
  apiKey: requireEnv("OPENAI_API_KEY"),
});

export type ModelResponse = {
  message: OpenAI.Chat.Completions.ChatCompletionMessage;
  usage?: { input_tokens: number; output_tokens: number };
};

export async function callModel(
  messages: ChatCompletionMessageParam[],
  tools: ChatCompletionTool[],
  signal?: AbortSignal,
): Promise<ModelResponse> {
  const response = await client.chat.completions.create(
    {
      model: requireEnv("OPENAI_MODEL"),
      messages,
      tools,
      tool_choice: "auto",
    },
    { signal },
  );

  const message = response.choices[0]?.message;
  if (!message) throw new Error("模型返回的消息为空");
  return {
    message,
    ...(response.usage
      ? {
          usage: {
            input_tokens: response.usage.prompt_tokens,
            output_tokens: response.usage.completion_tokens,
          },
        }
      : {}),
  };
}
