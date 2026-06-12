use crate::output::decode_text;
use crate::source::Source;
use anyhow::Result;

pub fn extract(source: &Source<'_>) -> Result<String> {
    Ok(decode_text(source.bytes()))
}
