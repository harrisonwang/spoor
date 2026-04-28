use crate::source::Source;
use anyhow::Result;

/// URL has already been fetched into Source by source::resolve().
/// We just delegate to the HTML extractor here, which knows how to do
/// readability-style main-content extraction.
///
/// Future: protocol-aware dispatch (youtube/arxiv/github/twitter) could go here.
pub fn extract(source: &Source) -> Result<String> {
    super::html::extract(source)
}
