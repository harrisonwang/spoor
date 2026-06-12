use anyhow::{Context, Result, anyhow};
use spoor_core::SpoorError;
use std::io::{Read, Take};
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct ResolvedInput {
    pub(crate) label: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) content_type: Option<String>,
}

impl ResolvedInput {
    pub(crate) fn len(&self) -> usize {
        self.bytes.len()
    }
}

pub(crate) fn resolve_input(input: &str, max_bytes: usize) -> Result<ResolvedInput> {
    if input == "-" {
        let bytes = read_limited(std::io::stdin(), max_bytes, "stdin read")
            .context("failed to read from stdin")?;
        return Ok(ResolvedInput {
            label: input.to_string(),
            bytes,
            content_type: None,
        });
    }

    if is_url(input) {
        return fetch_url(input, max_bytes);
    }

    let metadata =
        std::fs::metadata(input).with_context(|| format!("failed to inspect file: {input}"))?;
    if metadata.len() > max_bytes as u64 {
        return Err(SpoorError::parse_memory_limit(max_bytes, "local file read").into());
    }
    let file =
        std::fs::File::open(input).with_context(|| format!("failed to open file: {input}"))?;
    let bytes = read_limited(file, max_bytes, "local file read")
        .with_context(|| format!("failed to read file: {input}"))?;
    Ok(ResolvedInput {
        label: input.to_string(),
        bytes,
        content_type: None,
    })
}

fn fetch_url(url: &str, max_bytes: usize) -> Result<ResolvedInput> {
    let response = ureq::get(url)
        .set("User-Agent", concat!("spoor/", env!("CARGO_PKG_VERSION")))
        .set("Accept", "text/markdown, text/html;q=0.9")
        .timeout(Duration::from_secs(30))
        .call()
        .map_err(|error| anyhow!("failed to fetch URL: {error}"))?;
    let content_type = response.header("content-type").map(str::to_string);

    if let Some(length) = response
        .header("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        && length > max_bytes
    {
        return Err(SpoorError::parse_memory_limit(max_bytes, "URL response read").into());
    }

    let bytes = read_limited(response.into_reader(), max_bytes, "URL response read")
        .context("failed to read response body")?;
    Ok(ResolvedInput {
        label: url.to_string(),
        bytes,
        content_type,
    })
}

fn read_limited(reader: impl Read, max_bytes: usize, stage: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(max_bytes.min(1024 * 1024));
    let mut limited: Take<_> = reader.take(
        u64::try_from(max_bytes)
            .unwrap_or(u64::MAX)
            .saturating_add(1),
    );
    limited.read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(SpoorError::parse_memory_limit(max_bytes, stage).into());
    }
    Ok(bytes)
}

pub(crate) fn is_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}
