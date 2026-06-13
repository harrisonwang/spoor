# DOCX 测试矩阵

DOCX 测试重点是 WordprocessingML 的语义结构能否稳定转成 LLM-friendly Markdown-like 文本，而不是复刻 Word 的视觉样式。

| Fixture | Test | 验证契约 | 价值 | 状态 | 后续缺口 |
| --- | --- | --- | --- | --- | --- |
| `docx/01_basic.docx` | `basic_headings_and_inline_formatting` | 标题、普通段落、bold/italic/bold-italic 输出合法 Markdown | 最基础的阅读结构和 inline formatting | passed | 更复杂 run 交错、超链接内格式 |
| `docx/02c_lists_pstyle_only.docx` | `list_via_pstyle_only` | `pStyle=ListBullet/ListNumber` 识别为列表 | Word 常见列表可能没有 `<w:numPr>` | passed | 自定义 list style 名称 |
| `docx/02b_lists_numpr.docx` | `list_via_real_numpr` | `<w:numPr>` 列表、嵌套层级、有序/无序 marker | 防止列表被压平成普通段落 | passed | restart numbering、多级 decimal 模板 |
| `docx/03_tables.docx` | `tables_render_as_gfm` | 小表输出 GFM table，cell 内 `|` 被 Markdown 转义 | LLM 能保留行列结构 | passed | merged cells、caption、宽表 compact |
| `docx/04_hyperlinks.docx` | `hyperlinks_use_rels_lookup` | 通过 rels 把 hyperlink 渲染成 `[text](url)` | 链接目标对 Agent 很重要 | passed | bookmark/internal link |
| `docx/05_footnotes.docx` | `footnotes_collected_and_appended` | inline footnote marker + 文末 footnote body | 保留引用和来源说明 | passed | endnotes、comments |
| `docx/06_unicode.docx` | `unicode_passthrough` | CJK、RTL、数学符号、emoji、smart quotes 原样保留 | 多语言文档不能被转码破坏 | passed | 双向文本视觉顺序 |
| `docx/07_custom_prefix.docx` | `custom_namespace_prefix` | 按 namespace URI 解析，不依赖 `w:` 前缀 | OOXML prefix 不稳定 | passed | namespace 缺失/错误时的错误信息 |
| `docx/08_empty.docx` | `empty_document` | 空文档输出稳定 | 边界输入不 panic | passed | 空文档是否应有 warning |
| `docx/09_whitespace.docx` | `whitespace_only_paragraphs_skipped` | 纯空白段落不产生伪内容 | 降低噪声和 token | passed | 保留 intentional blank line 的策略 |
| `docx/10_heading_levels.docx` | `heading_levels_one_through_six` | Heading 1..6 映射到 Markdown heading | 标题层级是 LLM 分块核心 | passed | Heading 7+ fallback |
| `docx/11_whitespace_runs.docx` | `xml_space_preserve_runs` | `xml:space=preserve` 的空格语义被保留 | 防止 run 拼接破坏词边界 | passed | tab、soft hyphen、line break |
| `docx/12_tracked_changes.docx` | `tracked_changes_accept_inserts_drop_deletes` | 默认接受插入、丢弃删除 | 和多数抽取服务一致，输出低噪声 | passed | `--show-changes` 模式 |
| `docx/13_formatted_whitespace_only_runs.docx` | `formatted_whitespace_only_runs_no_panic` | 仅空白 run 上带粗斜体/链接时不 panic；不叠 Markdown 噪声 | Word 常见；对齐 md 输出降噪契约 | passed | — |
| `docx/14_merged_table.docx` | `merged_table_and_visual_omissions_are_explicit` | 合并单元格返回 `merged_table_structure_not_preserved` | Agent 不把降级 GFM 当原始表格结构 | passed | span 模型与 HTML 降级 |
| `docx/15_embedded_visual.docx` | `merged_table_and_visual_omissions_are_explicit` | 绘图/视觉对象省略返回 `embedded_visuals_omitted` | Agent 知道文本结果不完整 | passed | 稳定 visual id、alt/caption、外部 VLM 回填 |
| `docx/16_image_placeholders.docx` | `image_placeholders_follow_document_order_and_only_reference_safe_entries` | 图片按正文顺序输出安全 `spoor-docx://word/media/*` 占位符；过滤 fallback、HD Photo、外链和路径穿越 | Agent 可准确选择并自行解压需要交给 VLM 的图片 | passed | alt/caption、非 shell 宿主提取接口 |

## 下一批优先用例

- comments/endnotes：应作为引用块还是文末注释。
- 图片 alt/caption：在现有安全占位符上补充描述信息。
- 复杂表格：在现有合并 warning 上增加 span 模型、HTML 降级、空 header 和宽表 compact。
- numbering restart：同一文档多个列表的编号重启。
