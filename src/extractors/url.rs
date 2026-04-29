use crate::source::Source;
use anyhow::Result;

pub fn extract(source: &Source) -> Result<String> {
    super::html::extract(source)
}
