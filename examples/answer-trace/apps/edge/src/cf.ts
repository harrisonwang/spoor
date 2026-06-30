// LLM 客户端：fetch 任意 OpenAI 兼容端点（/chat/completions）。
// 等价于 apps/api/app/services/cf.py（那边用 openai SDK，这里直接 fetch，零依赖）。

import { cfg, type Env } from "./config";

// qwen3 这类带 thinking 的模型会先吐 <think>…</think>，污染 JSON，统一剥掉。
const THINK = /<think>[\s\S]*?<\/think>/g;

export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

interface ChatOptions {
  temperature?: number;
  jsonMode?: boolean;
}

interface ChatBody {
  model: string;
  messages: ChatMessage[];
  temperature: number;
  response_format?: { type: "json_object" };
}

async function post(url: string, apiKey: string, body: ChatBody): Promise<Response> {
  return fetch(url, {
    method: "POST",
    headers: {
      authorization: `Bearer ${apiKey}`,
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });
}

/** 调用一次 chat，返回纯文本（已剥离 thinking）。jsonMode 尽力要求 JSON 输出。 */
export async function chat(
  env: Env,
  model: string,
  messages: ChatMessage[],
  opts: ChatOptions = {},
): Promise<string> {
  const c = cfg(env);
  const url = `${c.baseUrl.replace(/\/$/, "")}/chat/completions`;
  const body: ChatBody = {
    model,
    messages,
    temperature: opts.temperature ?? 0,
  };
  if (opts.jsonMode) body.response_format = { type: "json_object" };

  let resp = await post(url, c.apiKey, body);
  if (!resp.ok && opts.jsonMode) {
    // 个别端点不接受 response_format，去掉重试一次（靠 prompt 约束 JSON）。
    const { response_format: _omit, ...rest } = body;
    resp = await post(url, c.apiKey, rest as ChatBody);
  }
  if (!resp.ok) {
    throw new Error(`模型端点返回 ${resp.status}：${(await resp.text()).slice(0, 300)}`);
  }

  const data = (await resp.json()) as { choices?: { message?: { content?: string } }[] };
  const text = data.choices?.[0]?.message?.content ?? "";
  return text.replace(THINK, "").trim();
}
