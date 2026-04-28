use crate::output::decode_text;
use crate::source::Source;
use anyhow::Result;

/// Markdown passthrough. We optionally normalize via pulldown-cmark
/// (re-render through a sanitizing pipeline) but for the skeleton we
/// simply decode and return.
pub fn extract(source: &Source) -> Result<String> {
    Ok(decode_text(source.bytes()))
}
