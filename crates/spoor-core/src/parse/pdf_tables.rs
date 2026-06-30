//! Recover real Markdown tables from PDF-extracted text.
//!
//! spoor's PDF text layer keeps a table's columns as runs of 2+ spaces
//! (`流动资产  370,572  542,724`). Rendered as Markdown that becomes a single
//! run-on paragraph — unreadable for a person and a number-soup for an LLM (so
//! the agent that reads this output mis-attributes cells and hallucinates). This
//! pass detects blocks of consecutive multi-column lines and re-emits them as
//! GFM tables, leaving everything else byte-for-byte unchanged.
//!
//! Conservative by design — it favours precision over recall. A block only
//! becomes a table when several consecutive lines each split into several
//! space-separated cells; ordinary prose almost never has repeated 2+-space gaps
//! on consecutive lines, so it passes through untouched. We would rather miss a
//! borderline table than wreck a paragraph.

/// Minimum cells (split on 2+ spaces) for a line to count as a table row.
const MIN_CELLS: usize = 3;
/// Minimum rows for a run to be re-emitted as a table.
const MIN_ROWS: usize = 3;

/// Reflow detected column blocks in `text` into GFM tables; leave the rest as-is.
pub(crate) fn tableize(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;

    while i < lines.len() {
        if !is_table_row(lines[i]) {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }

        // Collect a run of table rows, bridging a *single* blank line between two
        // rows (financial tables often double-space their rows). Two blank lines
        // end the run, which conveniently splits stacked sub-tables apart.
        let mut rows: Vec<usize> = vec![i];
        let mut last = i;
        let mut j = i + 1;
        while j < lines.len() {
            if is_table_row(lines[j]) {
                rows.push(j);
                last = j;
                j += 1;
            } else if is_blank(lines[j]) && j + 1 < lines.len() && is_table_row(lines[j + 1]) {
                j += 1; // skip the single blank; the next row is picked up above
            } else {
                break;
            }
        }

        if rows.len() >= MIN_ROWS {
            let run: Vec<&str> = rows.iter().map(|&k| lines[k]).collect();
            if !out.is_empty() {
                out.push(String::new()); // blank line so GFM sees a table block
            }
            out.push(render_table(&run));
            if last + 1 < lines.len() {
                out.push(String::new()); // blank line after the table
            }
            i = last + 1;
        } else {
            out.push(lines[i].to_string());
            i += 1;
        }
    }

    out.join("\n")
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

fn is_table_row(line: &str) -> bool {
    split_cells(line).len() >= MIN_CELLS
}

/// Split a line into trimmed cells on runs of 2+ spaces, dropping empties and
/// PDF tag markers like `[Table_Finance]` that would misalign the header.
fn split_cells(line: &str) -> Vec<&str> {
    line.split("  ")
        .map(str::trim)
        .filter(|s| !s.is_empty() && !is_tag_marker(s))
        .collect()
}

fn is_tag_marker(s: &str) -> bool {
    s.starts_with("[Table") && s.ends_with(']')
}

fn render_table(rows: &[&str]) -> String {
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|r| split_cells(r).iter().map(|c| escape_cell(c)).collect())
        .collect();
    let cols = cells.iter().map(Vec::len).max().unwrap_or(0);

    let mut lines: Vec<String> = Vec::with_capacity(cells.len() + 1);
    lines.push(format_row(&cells[0], cols)); // first row as header
    lines.push(separator(cols));
    for row in &cells[1..] {
        lines.push(format_row(row, cols));
    }
    lines.join("\n")
}

fn format_row(cells: &[String], cols: usize) -> String {
    let mut s = String::from("|");
    for c in 0..cols {
        s.push(' ');
        s.push_str(cells.get(c).map(String::as_str).unwrap_or(""));
        s.push_str(" |");
    }
    s
}

fn separator(cols: usize) -> String {
    let mut s = String::from("|");
    for _ in 0..cols {
        s.push_str(" --- |");
    }
    s
}

fn escape_cell(c: &str) -> String {
    // A literal `|` inside a cell would break the GFM table; escape it.
    c.trim().replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::tableize;

    #[test]
    fn prose_is_left_untouched() {
        let text = "这是一段正常中文，没有表格。\n第二行也是普通文字，只有单个空格分隔。\n第三行同样。";
        assert_eq!(tableize(text), text);
    }

    #[test]
    fn a_single_double_space_line_is_not_a_table() {
        // 2 cells < MIN_CELLS, and it's isolated — must stay as prose.
        let text = "市场数据收盘价(元)  375.36\n一年最低/最高价  201.17/403.40";
        assert_eq!(tableize(text), text);
    }

    #[test]
    fn three_column_block_becomes_a_gfm_table() {
        let text = "科目  2024A  2025E\n营收  100  120\n利润  10  12";
        let out = tableize(text);
        assert!(out.contains("| 科目 | 2024A | 2025E |"), "header: {out}");
        assert!(out.contains("| --- | --- | --- |"), "separator: {out}");
        assert!(out.contains("| 营收 | 100 | 120 |"), "row: {out}");
        // A blank line is unnecessary at the very start but required between rows;
        // here the table is the whole input, so no leading blank.
        assert!(out.starts_with("| 科目"), "no spurious leading blank: {out}");
    }

    #[test]
    fn two_rows_is_too_few_to_be_a_table() {
        let text = "科目  2024A  2025E\n营收  100  120";
        assert_eq!(tableize(text), text);
    }

    #[test]
    fn single_blank_line_between_rows_is_bridged() {
        let text = "科目  2024A  2025E\n\n营收  100  120\n\n利润  10  12";
        let out = tableize(text);
        assert!(out.contains("| --- | --- | --- |"), "{out}");
        assert!(out.contains("| 利润 | 10 | 12 |"), "{out}");
        // the bridged blanks must not survive inside the table
        assert!(!out.contains("| 营收 | 100 | 120 |\n\n"), "{out}");
    }

    #[test]
    fn double_blank_splits_into_two_tables() {
        let text =
            "甲  1  2\n乙  3  4\n丙  5  6\n\n\n丁  7  8\n戊  9  10\n己  11  12";
        let out = tableize(text);
        // two separator rows == two tables
        assert_eq!(out.matches("| --- | --- | --- |").count(), 2, "{out}");
    }

    #[test]
    fn table_tag_marker_is_dropped_so_header_aligns() {
        let text =
            "[Table_Finance]    资产负债表  2024A  2025E\n流动资产  370  542\n存货  116  141";
        let out = tableize(text);
        assert!(!out.contains("Table_Finance"), "marker dropped: {out}");
        assert!(out.contains("| 资产负债表 | 2024A | 2025E |"), "{out}");
    }

    #[test]
    fn ragged_rows_are_padded_to_the_widest() {
        let text = "a  b  c  d\ne  f\ng  h  i";
        let out = tableize(text);
        // widest row has 4 cells → every row padded to 4 (5 pipes per line)
        for line in out.lines().filter(|l| l.starts_with('|')) {
            assert_eq!(line.matches('|').count(), 5, "line not padded to 4 cols: {line}");
        }
    }

    #[test]
    fn prose_around_a_table_is_preserved_and_separated() {
        let text = "前言段落。\n科目  2024A  2025E\n营收  100  120\n利润  10  12\n结语段落。";
        let out = tableize(text);
        assert!(out.contains("前言段落。\n\n|"), "blank before table: {out}");
        assert!(out.contains("|\n\n结语段落。"), "blank after table: {out}");
    }
}
