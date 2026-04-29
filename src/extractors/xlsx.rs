use crate::source::Source;
use anyhow::{Context, Result};
use calamine::{Data, Reader, Xlsx, open_workbook_from_rs};
use std::io::Cursor;

pub fn extract(source: &Source) -> Result<String> {
    let cursor = Cursor::new(source.bytes().to_vec());
    let mut wb: Xlsx<_> = open_workbook_from_rs(cursor).context("failed to open xlsx")?;

    let mut out = String::new();
    let names = wb.sheet_names();

    for name in &names {
        let range = wb
            .worksheet_range(name)
            .with_context(|| format!("could not read sheet: {name}"))?;

        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!("## Sheet: {name}\n\n"));

        let rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| row.iter().map(format_cell).collect())
            .collect();

        if rows.is_empty() {
            continue;
        }

        let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        if cols == 0 {
            continue;
        }

        // First row as header.
        let mut header = rows[0].clone();
        while header.len() < cols {
            header.push(String::new());
        }
        out.push_str(&format!("| {} |\n", header.join(" | ")));
        out.push_str(&format!("|{}\n", " --- |".repeat(cols)));

        for row in rows.iter().skip(1) {
            if row.iter().all(|c| c.is_empty()) {
                continue;
            }
            let mut padded = row.clone();
            while padded.len() < cols {
                padded.push(String::new());
            }
            out.push_str(&format!("| {} |\n", padded.join(" | ")));
        }
    }

    Ok(out)
}

fn format_cell(cell: &Data) -> String {
    let raw = match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => (if *b { "TRUE" } else { "FALSE" }).to_string(),
        Data::DateTime(dt) => format_excel_datetime(dt.as_f64()),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => e.to_string(),
    };
    raw.replace('|', "\\|").replace(['\n', '\r', '\t'], " ")
}

fn format_excel_datetime(value: f64) -> String {
    if !value.is_finite() || value < 0.0 {
        return value.to_string();
    }

    let mut days = value.floor() as i64;
    let mut seconds = ((value - days as f64) * 86_400.0).round() as i64;
    if seconds >= 86_400 {
        days += 1;
        seconds -= 86_400;
    }

    if days == 0 {
        return format_time(seconds);
    }

    let (year, month, day) = civil_from_days(days - 25_569);
    if seconds == 0 {
        format!("{year:04}-{month:02}-{day:02}")
    } else {
        format!("{year:04}-{month:02}-{day:02}T{}", format_time(seconds))
    }
}

fn format_time(seconds: i64) -> String {
    let hour = seconds / 3600;
    let minute = (seconds % 3600) / 60;
    let second = seconds % 60;
    format!("{hour:02}:{minute:02}:{second:02}")
}

// Convert days since 1970-01-01 to Gregorian date.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as i32, m as u32, d as u32)
}
