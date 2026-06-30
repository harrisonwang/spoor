# @answer-trace/edge

answer-trace 的**边缘后端**：一个独立的 Cloudflare Worker，复刻原 `apps/api`（FastAPI）的 `/api/*` 契约，但把解析换成 **spoor-wasm**、把语料状态放到 **KV**、用任意 **OpenAI 兼容端点 / Cloudflare Workers AI** 做问答。可部署到 Workers，让用户在线试用，文档不出端（解析全在边缘 WASM 里）。

与 `apps/api` 是**平行替代**关系，不共用 `apps/web` 目录：前端照旧用 `VITE_API_URL` 指向本 Worker。

## 端点（与 apps/api 等价）

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/demo` | 内置三轮对话 + 其依据的原文 markdown |
| POST | `/api/ask` | `{question, corpusId?}` → AnswerTrace（生成 + 判定 + 确定性定位） |
| POST | `/api/upload` | 多文件 → spoor-wasm 解析 → 写入 KV 语料，返回 `corpusId` |
| GET | `/api/media?uri=&doc=&corpus=` | 按 spoor:// 安全 URI 取回内嵌图原始字节 |
| GET | `/api/health` | 健康检查 |

## 源码

```
src/
  index.ts      路由 + CORS（fetch handler）
  spoor.ts      spoor-wasm 初始化 + parse/extract 封装
  config.ts     Env + 模型/端点配置（AT_* 优先，否则 Workers AI）
  cf.ts         OpenAI 兼容 chat 客户端（fetch，零依赖）
  matcher.ts    生成 → 判定 → 确定性装配 AnswerTrace（移植自 matcher.py）
  locate.ts     三档确定性证据定位（移植自 locate.py，行为对齐）
  corpus.ts     KV 语料：setDocs/markdown/sourceRef/getDoc
  mediaUrls.ts  spoor:// 图片 → 绝对 /api/media 链接
  tokens.ts     token 近似估算（启发式，替代 tiktoken）
  store.ts      内置演示资产（byd.md / demo.json / byd.pdf 打进 Worker）
test/locate.test.ts  locate 的回归测试（移植自 test_locate.py）
```

## 跑起来

需要 Node + pnpm + 一个 Cloudflare 账号（`wrangler login`）。

```bash
# 1) 装依赖（在 answer-trace 根目录）
pnpm install

# 2) 建 KV 命名空间，把输出的 id 填进 wrangler.toml
pnpm --filter @answer-trace/edge exec wrangler kv namespace create CORPUS
pnpm --filter @answer-trace/edge exec wrangler kv namespace create CORPUS --preview

# 3) 配模型密钥（本地）：复制 .dev.vars.example → .dev.vars 填值
#    二选一：CF_ACCOUNT_ID+CF_API_TOKEN（Workers AI），或 AT_BASE_URL+AT_API_KEY

# 4) 本地起后端（http://localhost:8787）
pnpm --filter @answer-trace/edge dev

# 5) 前端指向它
VITE_API_URL=http://localhost:8787 pnpm --filter @answer-trace/web dev
#   或在仓库根：task dev:edge（记得先 export VITE_API_URL）
```

类型检查与测试：

```bash
pnpm --filter @answer-trace/edge typecheck
pnpm --filter @answer-trace/edge test
```

## 部署

```bash
# 生产密钥用 secret（不要写进 wrangler.toml）
pnpm --filter @answer-trace/edge exec wrangler secret put CF_API_TOKEN
pnpm --filter @answer-trace/edge exec wrangler secret put CF_ACCOUNT_ID   # 或 AT_BASE_URL / AT_API_KEY

pnpm --filter @answer-trace/edge deploy
```

部署后把前端构建时的 `VITE_API_URL` 设为 Worker 的公开地址（`*.workers.dev` 或自定义域）。`/api/media` 图片地址由后端按请求 origin 拼成绝对路径，跨源 `<img>` 显示无需额外 CORS。

## 边界与约束

- **请求上限 16 MiB**（演示）。大文档解析吃 CPU，用 spoor 的预算/分页参数压；免费版 CPU/时长有限。
- **语料存 KV**，按 `corpusId` 隔离多用户，TTL 24h 自清理。上传响应用本地结果直接拼，不依赖 KV 的 read-after-write（KV 最终一致）。上传后**紧接着** `/ask` 极小概率读到尚未同步的语料而回退到内置文档；正常人手输入问题的间隔足够同步。要强一致可改用 Durable Object。
- **多文件上传**的内嵌图统一按 `doc=0` 取（第 2+ 个文件的图取不到）——沿用原 `apps/api` 的行为，未额外处理。
- **token 数为近似值**（CJK≈1/字，其余≈4 字符/token），仅用于"是否超上下文"提示。
- **不在边缘的**：原 `apps/api` 用的 tiktoken 精确计数、Python 原生 pyspoor —— 边缘分别用启发式与 spoor-wasm 取代。
- `locate.ts` 的第③档表格兜底默认与 Python 端行为一致；如需更强反幻觉，可按文件内注释把非表格行改为不兜底。
