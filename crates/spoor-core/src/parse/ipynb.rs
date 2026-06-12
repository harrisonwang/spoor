use crate::limits;
use crate::output::decode_text;
use crate::source::Source;
use anyhow::{Result, anyhow};
use serde_json::Value;

pub fn extract(source: &Source<'_>, max_parse_bytes: usize) -> Result<String> {
    let text = decode_text(source.bytes());
    let v: Value = serde_json::from_str(&text).map_err(|e| anyhow!("invalid ipynb JSON: {e}"))?;

    let lang = v
        .get("metadata")
        .and_then(|m| m.get("kernelspec"))
        .and_then(|k| k.get("language"))
        .and_then(|l| l.as_str())
        .unwrap_or("");

    let cells = notebook_cells(&v)?;

    let mut out = String::new();
    for cell in cells {
        let kind = cell.get("cell_type").and_then(|v| v.as_str()).unwrap_or("");
        let src = read_source(cell.get("source").or_else(|| cell.get("input")));

        match kind {
            "heading" => {
                let heading = src.trim();
                if !heading.is_empty() {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    let level = cell
                        .get("level")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1)
                        .clamp(1, 6);
                    out.push_str(&"#".repeat(level as usize));
                    out.push(' ');
                    out.push_str(heading);
                    out.push('\n');
                }
            }
            "markdown" if !src.trim().is_empty() => {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&src);
                if !src.ends_with('\n') {
                    out.push('\n');
                }
            }
            "code" => {
                if src.trim().is_empty() {
                    continue;
                }
                if !out.is_empty() {
                    out.push('\n');
                }
                let lang = cell
                    .get("language")
                    .and_then(|v| v.as_str())
                    .filter(|v| !v.is_empty())
                    .unwrap_or(lang);
                out.push_str("```");
                out.push_str(lang);
                out.push('\n');
                out.push_str(&src);
                if !src.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```\n");
            }
            _ => {}
        }
        limits::ensure_parse_size(out.len(), max_parse_bytes, "IPYNB Markdown rendering")?;
    }
    Ok(out)
}

fn notebook_cells(v: &Value) -> Result<Vec<&Value>> {
    if let Some(cells) = v.get("cells").and_then(|c| c.as_array()) {
        return Ok(cells.iter().collect());
    }

    if let Some(worksheets) = v.get("worksheets").and_then(|w| w.as_array()) {
        let mut cells = Vec::new();
        let mut saw_cells_array = worksheets.is_empty();
        for worksheet in worksheets {
            if let Some(sheet_cells) = worksheet.get("cells").and_then(|c| c.as_array()) {
                saw_cells_array = true;
                cells.extend(sheet_cells);
            }
        }
        if saw_cells_array {
            return Ok(cells);
        }
    }

    Err(anyhow!(
        "ipynb missing 'cells' array or 'worksheets[].cells' arrays"
    ))
}

fn read_source(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}
