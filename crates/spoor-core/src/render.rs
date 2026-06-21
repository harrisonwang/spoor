use crate::detect::Format;
use crate::json_schema::JsonOutput;
use crate::result::DocumentResult as ExtractedDocument;
use anyhow::{Result, anyhow};
use std::fmt;

pub const DEFAULT_MAX_OUTPUT_BYTES: usize = 256 * 1024;
pub const MIN_MAX_OUTPUT_BYTES: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Md,
    Json,
}

impl fmt::Display for OutputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            OutputMode::Md => "md",
            OutputMode::Json => "json",
        })
    }
}

/// Pick the default output mode given the detected formats of all inputs.
/// All-table → json, anything else → md.
pub fn default_mode_for(formats: &[Format]) -> OutputMode {
    if !formats.is_empty()
        && formats
            .iter()
            .all(|f| matches!(f, Format::Csv | Format::Xlsx))
    {
        OutputMode::Json
    } else {
        OutputMode::Md
    }
}

pub fn render_documents(documents: &[ExtractedDocument], mode: OutputMode) -> Result<String> {
    match mode {
        OutputMode::Md => Ok(markdown::render(documents)),
        OutputMode::Json => Err(anyhow!("--mode json 仅支持 csv 和 xlsx 表格提取")),
    }
}

pub fn render_json(output: &JsonOutput) -> String {
    format!(
        "{}\n",
        serde_json::to_string(output).expect("serialize table JSON output")
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitedOutput {
    pub content: String,
    pub truncated: bool,
    pub warning: Option<String>,
}

pub fn limit_markdown_output(markdown: String, max_output_bytes: usize) -> LimitedOutput {
    if markdown.len() <= max_output_bytes {
        return LimitedOutput {
            content: markdown,
            truncated: false,
            warning: None,
        };
    }

    let warning = output_limit_warning(max_output_bytes);
    let marker = format!("\n\n> [!WARNING]\n> {warning}\n");
    let body_limit = max_output_bytes.saturating_sub(marker.len());
    let body_end = preferred_markdown_cut(&markdown, body_limit);
    let mut content = markdown[..body_end].trim_end().to_string();
    content.push_str(&marker);

    LimitedOutput {
        content,
        truncated: true,
        warning: Some(warning),
    }
}

pub fn render_json_limited(output: &JsonOutput, max_output_bytes: usize) -> LimitedOutput {
    let content = render_json(output);
    if content.len() <= max_output_bytes {
        return LimitedOutput {
            content,
            truncated: false,
            warning: None,
        };
    }

    let warning = output_limit_warning(max_output_bytes);
    let mut limited = output.clone();
    limited.truncated = true;
    limited.warnings.push(warning.clone());

    let content = loop {
        let rendered = render_json(&limited);
        if rendered.len() <= max_output_bytes {
            break rendered;
        }

        let Some(table) = limited.tables.last_mut() else {
            // The minimum CLI limit is large enough for this fallback envelope.
            limited.usage =
                "输出已截断。请缩小输入范围，或用 --max-output-kib 调高上限。".to_string();
            limited.warnings = vec![warning.clone()];
            break render_json(&limited);
        };

        if table.rows.is_empty() {
            limited.tables.pop();
            continue;
        }

        let keep = table.rows.len() / 2;
        table.rows.truncate(keep);
        table.truncated = true;
        let table_warning =
            format!("部分行因输出上限被省略。使用 --limit 翻页或 --rows 指定区间以获取完整数据。");
        if !table.warnings.contains(&table_warning) {
            table.warnings.push(table_warning);
        }
    };

    LimitedOutput {
        content,
        truncated: true,
        warning: Some(warning),
    }
}

fn output_limit_warning(max_output_bytes: usize) -> String {
    format!(
        "输出在 {} 字节处截断，内容不完整。可缩减范围（如 --pages、--rows），或调高 --max-output-kib。",
        max_output_bytes
    )
}

fn preferred_markdown_cut(markdown: &str, max_bytes: usize) -> usize {
    let mut end = max_bytes.min(markdown.len());
    while !markdown.is_char_boundary(end) {
        end -= 1;
    }

    let prefix = &markdown[..end];
    prefix.rfind('\n').map_or(end, |newline| newline + 1)
}

pub mod markdown {
    use crate::result::DocumentResult as ExtractedDocument;

    pub fn render(documents: &[ExtractedDocument]) -> String {
        let mut out = String::new();

        if let [document] = documents {
            push_markdown_body(&mut out, document);
            return out;
        }

        for (idx, document) in documents.iter().enumerate() {
            if idx > 0 {
                out.push('\n');
            }

            out.push_str("# Source: ");
            out.push_str(&markdown_heading_text(&document.source));
            out.push_str("\n\n");
            push_markdown_body(&mut out, document);
        }

        out
    }

    fn push_markdown_body(out: &mut String, document: &ExtractedDocument) {
        // An empty body must stay distinguishable from a successful read:
        // otherwise "the file has no text" and "extraction found nothing"
        // collapse into the same silent empty stdout + exit 0.
        if document.markdown.trim().is_empty() {
            out.push_str(&format!(
                "> [!NOTE]\n> 未从 {} 提取到文本（格式 {}，正文为空）。\n",
                markdown_heading_text(&document.source),
                document.format
            ));
            return;
        }

        out.push_str(&document.markdown);
        if !document.markdown.ends_with('\n') {
            out.push('\n');
        }
    }

    fn markdown_heading_text(source: &str) -> String {
        source.replace(['\r', '\n'], " ")
    }
}

#[cfg(test)]
mod tests {
    use super::{limit_markdown_output, preferred_markdown_cut};

    #[test]
    fn markdown_limit_preserves_utf8_and_appends_marker() {
        let markdown = "一行内容\n".repeat(500);
        let limited = limit_markdown_output(markdown, 1024);

        assert!(limited.truncated);
        assert!(limited.content.len() <= 1024);
        assert!(limited.content.contains("> [!WARNING]"));
        assert!(limited.content.contains("内容不完整"));
    }

    #[test]
    fn markdown_cut_prefers_complete_line() {
        assert_eq!(preferred_markdown_cut("first\nsecond\nthird", 13), 13);
        assert_eq!(preferred_markdown_cut("first\nsecond\nthird", 12), 6);
    }
}
