# EPUB 测试矩阵

EPUB 测试重点是 OPF spine 定义的阅读顺序。正文 Markdown 渲染目前仍是明显待补项。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `epub/01_basic.epub` | `basic_book_chapters_in_spine_order` | 解析 `META-INF/container.xml` 和 OPF，按 spine 顺序输出章节 | EPUB 文件名顺序不等于阅读顺序 | passed | 复用 HTML renderer 保留 heading/list/link |

## 下一批优先用例

- 非字典序 chapter 文件名，继续验证 spine。
- chapter 内 Markdown 渲染：heading、list、link、blockquote、table。
- 多 rootfile 或异常 OPF 的错误信息。
- 跳过 cover/css/nav 等非正文资源。
