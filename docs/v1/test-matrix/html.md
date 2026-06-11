# HTML 测试矩阵

HTML 目前不是最高优先级，但已有基础 readability 和 Markdown 渲染契约。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `html/01_article.html` | `semantic_article` | 优先抽 `article`，heading/list/link 转 Markdown，过滤 nav/footer | HTML 转 LLM 文本的基础结构 | passed | nested list 缩进 |
| `html/02_cluttered.html` | `cluttered_page_main_content_isolated` | main/article 内容保留，广告/导航/侧栏丢弃 | 降低网页噪声 | passed | 更完整 readability scoring |
| `html/03_table.html` | `html_table_to_gfm` | HTML table 转 GFM table | 保留表格行列语义 | passed | rowspan/colspan |
| `html/04_gbk_no_meta.html` | `gbk_html_without_meta_charset` | 无 charset meta 的 GBK HTML 可解码 | 中文网页兼容 | passed | HTTP charset 优先级 |
| `html/05_scripts_styles.html` | `script_and_style_tags_stripped` | script/style 内容绝不进入输出 | 安全与降噪 | passed | noscript 策略 |
| `html/06_links.html` | `links_preserve_href` | link text 和 href 都保留 | Agent 需要 URL 目标 | passed | 相对 URL resolve |

## 下一批优先用例

- `<pre>` / `<code>` 输出 fenced block。
- blockquote。
- image alt/caption placeholder。
- nested lists。
- relative URL resolution。
