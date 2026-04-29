/// Common helpers for extractors.

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
