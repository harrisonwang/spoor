# @answer-trace/pages

answer-trace 的 **Cloudflare Pages** 形态：把 **web（前端）+ edge（后端）合成一个 Pages 项目**——
前端是 SvelteKit 静态 SPA，后端是顶层 `functions/` 里的 **Pages Functions**，**同源、一次部署**。
相比 `apps/web`(纯前端) + `apps/edge`(独立 Worker) 的两处分离，这里前后端合一。

在这一版里同时落地了我们讨论的**分级核验**（比 `apps/edge` 的二元 located/未定位更聪明）：

- **金档（确定性，无额外 LLM）**：逐字 / 空白归一 / 表格单元格锚点 / **新增数值·单位归一**
  （`7771亿 = 777102百万`、`531亿 ≈ 53128百万`），治"真事实因非逐字被拒"的假阴性。
- **银档（一次轻量 LLM 蕴含）**：对"判定支持但金档没定位到"的 claim 批量做蕴含检索；
  返回的 quote **仍要确定性 `locate` 到才采纳**（不洗白幻觉），命中降一档为 `~` 并标注"语义匹配（非逐字）"。
- 仍定位不到 → 「无法核验」。反幻觉闸门不变，只是把召回做上去、且不牺牲"宁可不洗白"的安全侧。

模型走**任意 OpenAI 兼容端点**（`AT_BASE_URL`+`AT_API_KEY`，或 Cloudflare Workers AI），与现状一致。

## 端点（Pages Functions，同 apps/edge 契约）

`GET /api/demo` · `POST /api/ask {question,corpusId?}` · `POST /api/upload` · `GET /api/media?uri=&doc=&corpus=`

## 源码

```
src/                      SvelteKit 前端（静态 SPA；loadTrace 同源相对调 /api/*）
static/_demo/             内置演示资产（byd.md · demo.json · byd.pdf），Functions 用 env.ASSETS 取
functions/
  api/{demo,ask,upload,media}.ts   Pages Functions（onRequest*）
  _lib/
    config.ts cf.ts spoor.ts tokens.ts http.ts parse.ts   模型/解析/工具
    mediaUrls.ts          spoor:// → 同源 /api/media 相对链接
    store.ts corpus.ts    内置演示(ASSETS) + KV 语料
    locate.ts             ★ 分级核验金档（含数值/单位归一）
    matcher.ts            ★ 生成→判定→金档→银档 装配
    __tests__/locate.test.ts   金档 + 安全性回归（vitest）
```

## 跑起来

需要 Node + pnpm + Cloudflare 账号（`wrangler login`）。

```bash
pnpm install                      # 在 answer-trace 根

# 建/复用 KV，把 id 填进 wrangler.toml（可复用 apps/edge 那个）
pnpm --filter @answer-trace/pages exec wrangler kv namespace create CORPUS

# 配模型密钥（本地）：cp .dev.vars.example .dev.vars 填 AT_BASE_URL+AT_API_KEY（或 CF_*）

# 全栈本地（前端 + Functions 同源）
pnpm --filter @answer-trace/pages run dev:pages     # = build + wrangler pages dev

# 仅前端（Functions 打不通 → loadTrace 回退内置 fixture）
pnpm --filter @answer-trace/pages dev
```

检查与测试：

```bash
pnpm --filter @answer-trace/pages run check            # 前端 svelte-check
pnpm --filter @answer-trace/pages run check:functions  # 后端 tsc
pnpm --filter @answer-trace/pages test                 # 金档 locate 回归
```

## 部署

```bash
pnpm --filter @answer-trace/pages exec wrangler pages secret put AT_API_KEY   # 及 AT_BASE_URL / CF_*
pnpm --filter @answer-trace/pages run deploy                                  # = build + wrangler pages deploy
```

一个 Pages 项目同时上前端 + `/api/*`，同域、无 CORS、文档不出端。

## 边界与说明

- **前后端同源**：前端相对调 `/api/*`，图片是 `/api/media?...` 相对链接，无需 CORS。
- **内置演示资产走 env.ASSETS**（静态 `/_demo/*`），避免把 .md/.pdf 打进 bundle 的规则不确定性；上传语料仍走 KV，按 `corpusId` 隔离，TTL 24h。
- **分级核验的代价**：银档是"仅对未命中 claim 触发一次批量 LLM 调用"，无未命中则零额外开销；命中一律标 `~`，透明不洗白。
- 已验证：`vitest`(金档) + `svelte-check` + `tsc functions` + `vite build` + `wrangler pages functions build` 全绿；银档需带模型密钥部署后在线体验。
