use crate::source::Source;
use anyhow::Result;

pub fn extract(source: &Source, max_parse_bytes: usize) -> Result<String> {
    if source.is_markdown() {
        // Server returned markdown via content negotiation
        super::markdown::extract(source)
    } else {
        // Fall back to HTML parsing
        super::html::extract(source, max_parse_bytes)
    }
}
