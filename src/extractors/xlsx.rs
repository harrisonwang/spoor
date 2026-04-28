use crate::source::Source;
use anyhow::{Context, Result};
use calamine::{open_workbook_from_rs, Data, Reader, Xlsx};
use std::io::Cursor;

/// Each sheet → "## Sheet: <name>" header + GFM markdown table.
/// Empty cells render as empty pipes; whole-empty rows are skipped.
pub fn extract(source: &Source) -> Result<String> {
    let cursor = Cursor::new(source.bytes().to_vec());
    let mut wb: Xlsx<_> =
        open_workbook_from_rs(cursor).context("failed to open xlsx")?;

    let mut out = String::new();
    let names = wb.sheet_names();
    for name in &names {
        let range = wb
            .worksheet_range(name)
            .with_context(|| format!("could not read sheet: {}", name))?;

        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!("## Sheet: {}\n\n", name));

        let rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| row.iter().map(format_cell).collect())
            .collect();

        if rows.is_empty() {
            continue;
        }

        // Render as GFM table: first row treated as header.
        let cols = rows[0].len();
        out.push_str(&format!("| {} |\n", rows[0].join(" | ")));
        out.push_str(&format!("| {} |\n", vec!["---"; cols].join(" | ")));
        for row in rows.iter().skip(1) {
            // Skip rows that are entirely empty.
            if row.iter().all(|c| c.is_empty()) {
                continue;
            }
            out.push_str(&format!("| {} |\n", row.join(" | ")));
        }
    }

    Ok(out)
}

fn format_cell(cell: &Data) -> String {
    let raw = match cell {
        Data::Empty => return String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => if *b { "TRUE".into() } else { "FALSE".into() },
        Data::DateTime(d) => d.to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#{:?}", e),
    };
    // Pipe and newline would break the markdown table.
    raw.replace('|', "\\|").replace('\n', " ").replace('\r', "")
}
