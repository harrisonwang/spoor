use crate::output::decode_text;
use crate::source::Source;
use anyhow::{anyhow, Result};
use serde_json::Value;

/// markdown cell → passthrough
/// code cell     → fenced ``` block (with kernel language if available)
/// raw cell      → skipped
/// outputs       → skipped (intentionally — they're noisy for LLMs)
pub fn extract(source: &Source) -> Result<String> {
    let text = decode_text(source.bytes());
    let v: Value = serde_json::from_str(&text).map_err(|e| anyhow!("invalid ipynb JSON: {}", e))?;

    let lang = v
        .get("metadata")
        .and_then(|m| m.get("kernelspec"))
        .and_then(|k| k.get("language"))
        .and_then(|l| l.as_str())
        .unwrap_or("");

    let cells = v
        .get("cells")
        .and_then(|c| c.as_array())
        .ok_or_else(|| anyhow!("ipynb missing 'cells' array"))?;

    let mut out = String::new();
    for cell in cells {
        let kind = cell.get("cell_type").and_then(|v| v.as_str()).unwrap_or("");
        let source_text = read_source(cell.get("source"));

        match kind {
            "markdown" => {
                if !source_text.trim().is_empty() {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(&source_text);
                    if !source_text.ends_with('\n') {
                        out.push('\n');
                    }
                }
            }
            "code" => {
                if source_text.trim().is_empty() {
                    continue;
                }
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str("```");
                out.push_str(lang);
                out.push('\n');
                out.push_str(&source_text);
                if !source_text.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```\n");
            }
            // "raw" and unknown types are silently skipped.
            _ => {}
        }
    }
    Ok(out)
}

/// `source` may be either a single string or a list of lines (each line
/// already includes its trailing '\n' per nbformat spec).
fn read_source(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}
