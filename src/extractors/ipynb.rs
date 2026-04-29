use crate::output::decode_text;
use crate::source::Source;
use anyhow::{Result, anyhow};
use serde_json::Value;

pub fn extract(source: &Source) -> Result<String> {
    let text = decode_text(source.bytes());
    let v: Value = serde_json::from_str(&text).map_err(|e| anyhow!("invalid ipynb JSON: {e}"))?;

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
        let src = read_source(cell.get("source"));

        match kind {
            "markdown" => {
                if !src.trim().is_empty() {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(&src);
                    if !src.ends_with('\n') {
                        out.push('\n');
                    }
                }
            }
            "code" => {
                if src.trim().is_empty() {
                    continue;
                }
                if !out.is_empty() {
                    out.push('\n');
                }
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
    }
    Ok(out)
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
