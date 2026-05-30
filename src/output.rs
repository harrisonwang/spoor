//! Common helpers for extractors.

pub struct MarkdownBuilder {
    buf: String,
}

impl MarkdownBuilder {
    pub fn new() -> Self {
        Self { buf: String::new() }
    }

    pub fn heading(&mut self, level: u8, text: &str) {
        let level = level.clamp(1, 6);
        self.ensure_blank_line();
        for _ in 0..level {
            self.buf.push('#');
        }
        self.buf.push(' ');
        self.buf.push_str(text.trim());
        self.buf.push('\n');
    }

    pub fn paragraph(&mut self, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        self.ensure_blank_line();
        self.buf.push_str(trimmed);
        self.buf.push('\n');
    }

    pub fn raw(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    /// Append `rows` as a GFM table (first row = header), preceded by a blank
    /// line. No-op when there are no columns.
    pub fn table(&mut self, rows: &[Vec<String>]) {
        let rendered = gfm_table(rows);
        if rendered.is_empty() {
            return;
        }
        self.blank_line();
        self.buf.push_str(&rendered);
    }

    pub fn blank_line(&mut self) {
        self.ensure_blank_line();
    }

    fn ensure_blank_line(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        if !self.buf.ends_with('\n') {
            self.buf.push('\n');
        }
        if !self.buf.ends_with("\n\n") {
            self.buf.push('\n');
        }
    }

    pub fn build(self) -> String {
        let mut s = self.buf.trim_end().to_string();
        s.push('\n');
        s
    }
}

impl Default for MarkdownBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Decode bytes with charset detection (UTF-8 / GBK / GB18030 / Big5 / etc).
pub fn decode_text(bytes: &[u8]) -> String {
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    let (cow, _, _) = encoding.decode(bytes);
    cow.into_owned()
}

/// Escape one cell for a GFM table: escape pipes and collapse any newline /
/// carriage-return / tab to a single space so the row stays on one line.
pub fn escape_table_cell(cell: &str) -> String {
    cell.replace('|', "\\|").replace(['\n', '\r', '\t'], " ")
}

/// Render `rows` as a GFM table (first row = header). Ragged rows are padded
/// to the widest row, and every cell is escaped via [`escape_table_cell`].
/// Returns an empty string when there are no rows or no columns. Each line
/// ends with `\n`; callers add any surrounding blank lines.
pub fn gfm_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }

    let mut out = String::new();
    push_table_row(&mut out, &rows[0], cols);
    out.push('|');
    for _ in 0..cols {
        out.push_str(" --- |");
    }
    out.push('\n');
    for row in &rows[1..] {
        push_table_row(&mut out, row, cols);
    }
    out
}

fn push_table_row(out: &mut String, row: &[String], cols: usize) {
    out.push_str("| ");
    for col in 0..cols {
        if col > 0 {
            out.push_str(" | ");
        }
        out.push_str(&escape_table_cell(row.get(col).map_or("", String::as_str)));
    }
    out.push_str(" |\n");
}
