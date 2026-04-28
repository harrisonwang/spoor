use crate::output::decode_text;
use crate::source::Source;
use anyhow::Result;

/// Strategy:
///   1. Decode bytes to text with charset detection.
///   2. Run readability to isolate the main article content.
///   3. Convert the resulting HTML fragment to markdown
///      (basic walker — we don't need full fidelity).
///
/// For now, falls back to a naive tag-stripper if readability fails.
pub fn extract(source: &Source) -> Result<String> {
    let html = decode_text(source.bytes());
    let base_url = source.url().unwrap_or("");

    // TODO: hook up `readability` crate here. For the skeleton we ship
    // a minimal text extraction so the crate compiles end-to-end.
    let plain = strip_tags(&html);
    Ok(plain)
}

/// Bare-minimum HTML → text fallback. Replace with proper readability + html2md
/// before shipping.
fn strip_tags(html: &str) -> String {
    use scraper::{Html, Selector};
    let doc = Html::parse_document(html);
    let body = Selector::parse("body").unwrap();

    let mut out = String::new();
    if let Some(b) = doc.select(&body).next() {
        for text in b.text() {
            let t = text.trim();
            if !t.is_empty() {
                out.push_str(t);
                out.push('\n');
            }
        }
    }
    out
}
