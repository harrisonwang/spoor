use crate::output::decode_text;
use crate::source::Source;
use anyhow::Result;

/// CSV → markdown table. Steps:
///   1. Decode bytes (handles GBK/GB18030/UTF-8 with chardetng).
///   2. Sniff delimiter from a sample (',' '\t' ';' '|').
///   3. Parse with `csv` crate, render as GFM table.
///   4. If file is huge, truncate to first N rows + a "(truncated)" line.
pub fn extract(source: &Source) -> Result<String> {
    let text = decode_text(source.bytes());
    let delimiter = sniff_delimiter(&text);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .flexible(true)
        .from_reader(text.as_bytes());

    const MAX_ROWS: usize = 1000;
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut truncated = false;

    for (i, rec) in rdr.records().enumerate() {
        if i >= MAX_ROWS {
            truncated = true;
            break;
        }
        let rec = rec?;
        rows.push(rec.iter().map(|s| sanitize(s)).collect());
    }

    if rows.is_empty() {
        return Ok(String::new());
    }

    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    // Pad short rows.
    for r in &mut rows {
        while r.len() < cols {
            r.push(String::new());
        }
    }

    let mut out = String::new();
    out.push_str(&format!("| {} |\n", rows[0].join(" | ")));
    out.push_str(&format!("| {} |\n", vec!["---"; cols].join(" | ")));
    for row in rows.iter().skip(1) {
        out.push_str(&format!("| {} |\n", row.join(" | ")));
    }
    if truncated {
        out.push_str(&format!(
            "\n_(truncated at {} rows)_\n",
            MAX_ROWS
        ));
    }
    Ok(out)
}

fn sniff_delimiter(text: &str) -> u8 {
    let sample: String = text.lines().take(20).collect::<Vec<_>>().join("\n");
    let candidates = [b',', b'\t', b';', b'|'];
    let mut best = (b',', 0usize);
    for &d in &candidates {
        let count = sample.bytes().filter(|&b| b == d).count();
        if count > best.1 {
            best = (d, count);
        }
    }
    best.0
}

fn sanitize(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ").replace('\r', "")
}
