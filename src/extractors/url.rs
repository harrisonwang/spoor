use crate::source::Source;
use anyhow::Result;

pub fn extract(source: &Source) -> Result<String> {
    if source.is_markdown() {
        // Server returned markdown via content negotiation
        super::markdown::extract(source)
    } else {
        // Fall back to HTML parsing
        super::html::extract(source)
    }
}
