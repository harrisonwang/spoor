// 配置：模型后端（任意 OpenAI 兼容端点）+ 模型 + token 上限。全走 env。
// AT_BASE_URL+AT_API_KEY 优先，否则回退 Cloudflare Workers AI。

export interface Env {
  /** 语料 KV 命名空间。 */
  CORPUS: KVNamespace;
  /** Pages 自带：取项目静态资源（内置演示的 byd.md / demo.json / byd.pdf）。 */
  ASSETS: Fetcher;
  AT_GEN_MODEL?: string;
  AT_JUDGE_MODEL?: string;
  AT_CONTEXT_LIMIT?: string;
  /** 自定义 OpenAI 兼容端点（OpenRouter / DeepSeek / z.ai…）。设了就优先。 */
  AT_BASE_URL?: string;
  AT_API_KEY?: string;
  /** 便捷后端：Cloudflare Workers AI。 */
  CF_ACCOUNT_ID?: string;
  CF_API_TOKEN?: string;
}

export interface Config {
  genModel: string;
  judgeModel: string;
  baseUrl: string;
  apiKey: string;
  contextLimit: number;
}

const DEFAULT_GEN = "@cf/google/gemma-4-26b-a4b-it";
const DEFAULT_JUDGE = "@cf/qwen/qwen3-30b-a3b-fp8";

export function cfg(env: Env): Config {
  const baseUrl = env.AT_BASE_URL
    ? env.AT_BASE_URL
    : `https://api.cloudflare.com/client/v4/accounts/${env.CF_ACCOUNT_ID ?? ""}/ai/v1`;
  return {
    genModel: env.AT_GEN_MODEL || DEFAULT_GEN,
    judgeModel: env.AT_JUDGE_MODEL || DEFAULT_JUDGE,
    baseUrl,
    apiKey: env.AT_API_KEY || env.CF_API_TOKEN || "",
    contextLimit: Number.parseInt(env.AT_CONTEXT_LIMIT || "32768", 10),
  };
}

export function llmEnabled(env: Env): boolean {
  if (env.AT_BASE_URL && env.AT_API_KEY) return true;
  return Boolean(env.CF_ACCOUNT_ID && env.CF_API_TOKEN);
}
