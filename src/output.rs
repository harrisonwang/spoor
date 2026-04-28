/// Tiny helpers shared by extractors when assembling markdown output.
/// We deliberately don't introduce a typed AST — extractors push strings.

pub struct MarkdownBuilder {
    buf: String,
}

impl MarkdownBuilder {
    pub fn new() -> Self {
        Self { buf: String::new() }
    }

    pub fn heading(&mut self, level: u8, text: &str) {
        let level = level.min(6).max(1);
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

    pub fn newline(&mut self) {
        if !self.buf.ends_with('\n') {
            self.buf.push('\n');
        }
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
        // Trim trailing whitespace, ensure single final newline.
        let mut s = self.buf.trim_end().to_string();
        s.push('\n');
        s
    }
}

/// Decode bytes with charset detection (handles GBK/GB18030/Big5/Shift_JIS/UTF-8/etc).
pub fn decode_text(bytes: &[u8]) -> String {
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    let (cow, _, _) = encoding.decode(bytes);
    cow.into_owned()
}

/// Replace control characters that would break TSV/markdown output.
pub fn sanitize_cell(s: &str) -> String {
    s.replace('\t', " ").replace('\n', " ").replace('\r', "")
}
