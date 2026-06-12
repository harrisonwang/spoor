# 能力与限制

本文是 spoor `v0.8.3` 的能力边界。它描述代码当前实际行为，而不是未来路线图。

## 总体资源限制

| 限制 | 默认值 | 作用范围 | 说明 |
| --- | ---: | --- | --- |
| core 共享解析预算 | 64 MiB | 单次 `ParseRequest` | 输入、ZIP 声明总解压量、中间文本与保留结果共享预算；最小可配置值 1 KiB |
| CLI 共享解析预算 | 64 MiB | 整次命令 | 多个输入共同消耗，不是每个文件各 64 MiB |
| CLI stdout 上限 | 256 KiB | 整次命令 | Markdown 会附截断标记；表格 JSON 保持合法并写入 warning；最小可配置值 1 KiB |
| ZIP 条目数 | 10,000 | DOCX/PPTX/EPUB 等 ZIP 容器 | 不提供公开覆写接口 |
| ZIP 单条目解压大小 | 50 MiB | 单个 ZIP entry | 仍受更小的共享解析预算约束 |
| ZIP 压缩比 | 200x | 单个 ZIP entry | 超出即拒绝 |
| ZIP 声明总解压量 | 共享解析预算 | 整个 ZIP | 在真正解压前检查中央目录 |
| core 进程内超时 | 无 | 所有原生/WASM 调用 | 调用方必须在 worker、容器、宿主 Runtime 或独立进程设置超时 |
| 严格 RSS 上限 | 无 | 所有原生调用 | 数据量预算不等于操作系统内存上限；第三方解析器与跨语言传输可能产生额外副本 |

解析复杂度不只由压缩文件大小决定。小型 PDF、XLSX 或高压缩比 Office 文件也可能
比同等字节数的纯文本消耗更多 CPU 和内存。

## 重点格式

| 格式 | 当前保留 | 当前明确不保留或限制 |
| --- | --- | --- |
| DOCX | 标题 1-6、段落、粗体/斜体、列表、小型 GFM 表格、链接、脚注、Unicode、插入型 tracked changes | 样式与版式、图片、删除型 tracked changes、comments/endnotes、复杂编号重启、复杂合并表格、图表与嵌入对象 |
| XLSX | sheet、range、标题/表头/preamble、文本/数字/布尔/日期、缓存公式结果、错误单元格、合并单元格左上值 | 默认每个 sheet 仅前 100 条数据行；不计算公式、不保留公式表达式/样式/图表；Excel 1904 date system 尚未完整处理；一个 sheet 按一个逻辑 table 输出 |
| PDF | text layer、页顺序、`## Page N` 边界 | 不做 OCR；不恢复版面/标题语义；不保证复杂多栏阅读顺序；不去重页眉页脚、不修复断词；纯图片与加密 PDF 返回结构化错误 |
| PPTX | 按数字 slide 顺序输出文本、小型表格、speaker notes | 不按 shape 坐标恢复视觉阅读顺序；不保留 bullet 层级、主题、动画、图表、图片或嵌入对象 |
| HTML / URL | 优先 `article`、其次 `main`、最后 `body`；标题、段落、列表、链接、表格、引用块、代码块、image alt、粗体/斜体；跳过常见导航与脚本噪声 | 不是完整 readability 引擎；不解析相对链接；caption 与嵌套列表仍有限；core 不抓 URL，只有 CLI 会发起网络请求 |
| EPUB | OPF spine 阅读顺序、章节边界，并复用 HTML Markdown renderer | 不处理 DRM、固定版式视觉结构、图片/音视频、复杂导航与 CSS 布局 |
| IPYNB | markdown cell、code cell、cell 顺序、kernelspec language hint | 从不执行代码；跳过 raw cell、outputs、widgets、HTML output 与 base64 图片 |

## 基础格式

- CSV/TSV：第 1 行视为 header，自动识别常见 delimiter 与编码；默认只返回前
  100 条数据行，不做强类型推断。
- Markdown、纯文本、常见代码与配置文件：做字符集解码后保留文本；不是语法分析器。
- 不支持旧版二进制 Office `.doc` / `.xls` / `.ppt`、密码保护 Office、OCR、
  宏执行、公式执行、notebook 执行、脚本执行或内嵌二进制提取。

## 格式检测限制

检测顺序大致为：

1. PDF、ZIP、OLE/CFB 等 magic bytes；
2. ZIP 内部结构区分 DOCX/XLSX/PPTX/EPUB；
3. 文件名或 URL 扩展名；
4. URL 的部分 `content-type` 辅助判断；
5. 可解码文本回退为纯文本。

`source_name` / `x-filename` 很重要。没有扩展名的 CSV、IPYNB、Markdown 与普通
JSON/文本可能无法可靠区分；错误扩展名或错误 `content-type` 也可能导致错误分派。
对 stdin 中的 CSV 应显式传 `--format csv`。OLE/CFB 会在扩展名回退前被拒绝，
避免把旧版或加密 Office 当成普通文本。

## 运行形态与示例限制

| 形态 / 示例 | spoor 限制 | 额外宿主限制 |
| --- | --- | --- |
| CLI | 默认整次命令 64 MiB 解析预算、256 KiB stdout；URL 读取 30 秒 timeout | URL 抓取不是爬虫：未提供 host allowlist、robots、登录态或完整 redirect 策略 |
| Rust / Python / Node | 默认每次调用 64 MiB；无输出封顶、无进程内 timeout | 在调用方进程内执行；恶意多租户输入应使用进程或容器隔离 |
| 默认发布 WASM | `v0.8.3` 起包含全部重点格式；默认每次调用 64 MiB | 浏览器/WASM 线性内存、JS heap 和 bytes/字符串副本会降低实际可用上限 |
| Cloudflare Worker / Pages Function 示例 | 显式拒绝超过 16 MiB 的请求并把解析预算设为 16 MiB | Worker isolate 内存 128 MB，包含 JS 与 WASM；Free HTTP CPU 10 ms，Paid 默认 30 s、最高 5 min；公开演示没有认证、限流或租户隔离 |
| Cloudflare Pages 本地模式 | 显式 16 MiB 单文件上限 | 文件留在浏览器，但仍受浏览器内存与主线程响应性约束 |
| 本地语料库示例 | 16 MiB/文件；局部失败不阻断批次 | 没有文件数量、总字节数、总输出或取消上限；会同时保留原文件、解析结果和 chunk，不能当作无限语料库 |
| Python 摄取示例 | 64 MiB/文件；文档按字符窗口切块 | CSV/XLSX 只摄取默认前 100 行并写入 `table_preview_truncated`；未做 embedding、向量化或增量索引 |
| Electron 示例 | 显式 64 MiB/文件 | 解析在主进程内；生产应用应使用 Utility Process/worker 做隔离 |
| Tauri 示例 | core 默认 64 MiB/文件 | 当前 `Array.from(Uint8Array)` 到 `Vec<u8>` 会复制数据，不适合超大文件 |
| Lambda 示例 | CLI 默认 64 MiB 解析预算、256 KiB 输出 | 同步请求/响应各 6 MB，异步 payload 1 MB，最长 15 分钟；大文件应通过 S3 |

Cloudflare 官方当前还限制 Worker 压缩后体积为 Free 3 MB / Paid 10 MB，Pages
单个静态资源最大 25 MiB。CI 会对 `full` 与 `core-formats` WASM 都执行 3 MiB gzip
门禁。平台限制会变化，部署前应重新检查官方文档：

- [Cloudflare Workers limits](https://developers.cloudflare.com/workers/platform/limits/)
- [Cloudflare Pages limits](https://developers.cloudflare.com/pages/platform/limits/)
- [AWS Lambda quotas](https://docs.aws.amazon.com/lambda/latest/dg/gettingstarted-limits.html)

## 内容与安全边界

- spoor 只提取现有文本和结构，不判断事实真伪，也不会识别 prompt injection。
- 文档中的恶意指令会作为普通文本进入输出；交给 LLM 前必须按不可信数据处理。
- “本地解析”只表示 spoor 不主动上传；宿主应用、浏览器扩展、日志、剪贴板和后续
  LLM 调用仍可能泄露内容。
- `ParseResult` 的 `warnings`、表格的 `truncated` / `warnings` 和稳定错误 `code`
  是调用方必须处理的契约，不能只判断调用是否成功。
- 边缘公开示例用于展示能力，不是生产文档服务。生产部署至少需要认证、限流、
  请求审计、内容保留策略、超时与并发隔离。

## 当前能力短板

近期最值得增强的不是继续增加格式数量，而是：

1. 为 HTML/EPUB 补齐嵌套列表、caption、相对链接与更稳定的 readability。
2. 改善 PDF 多栏顺序、页眉页脚去重与断词修复；OCR 保持外置。
3. 为 PPTX 按 shape 坐标恢复阅读顺序并保留 bullet 层级。
4. 为 Python/Node 暴露表格分页筛选能力，避免 RAG 管道只能摄取默认预览。
5. 在批处理宿主中增加超时、取消和总批次预算；core 内不承诺硬超时。
