use anyhow::{Context, Result, anyhow};
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInput {
    raw: String,
}

impl SourceInput {
    pub fn new(raw: impl Into<String>) -> Self {
        Self { raw: raw.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl From<String> for SourceInput {
    fn from(raw: String) -> Self {
        Self::new(raw)
    }
}

impl From<&str> for SourceInput {
    fn from(raw: &str) -> Self {
        Self::new(raw)
    }
}

/// A resolved input: either local file bytes or a fetched URL response.
pub struct Source {
    bytes: Vec<u8>,
    origin: Origin,
}

enum Origin {
    File {
        path: PathBuf,
    },
    Url {
        url: String,
        content_type: Option<String>,
        is_markdown: bool,
    },
    /// Bytes read from standard input (the `-` input token). No path, so
    /// format detection falls back to magic bytes / `--format`.
    Stdin,
}

impl Source {
    pub fn resolve(input: &str) -> Result<Self> {
        if input == "-" {
            Self::read_stdin()
        } else if is_url(input) {
            Self::fetch_url(input)
        } else {
            Self::read_file(input)
        }
    }

    fn read_stdin() -> Result<Self> {
        let mut bytes = Vec::new();
        std::io::stdin()
            .read_to_end(&mut bytes)
            .context("failed to read from stdin")?;
        Ok(Self {
            bytes,
            origin: Origin::Stdin,
        })
    }

    fn read_file(path: &str) -> Result<Self> {
        let pb = PathBuf::from(path);
        let bytes = std::fs::read(&pb).with_context(|| format!("failed to read file: {path}"))?;
        Ok(Self {
            bytes,
            origin: Origin::File { path: pb },
        })
    }

    fn fetch_url(url: &str) -> Result<Self> {
        let resp = ureq::get(url)
            .set("User-Agent", concat!("pith/", env!("CARGO_PKG_VERSION")))
            .set("Accept", "text/markdown, text/html;q=0.9")
            .timeout(Duration::from_secs(30))
            .call()
            .map_err(|e| anyhow!("failed to fetch URL: {e}"))?;

        let content_type = resp.header("content-type").map(|s| s.to_string());
        let is_markdown = content_type
            .as_ref()
            .map(|ct| ct.starts_with("text/markdown"))
            .unwrap_or(false);

        let mut bytes = Vec::new();
        resp.into_reader()
            .take(50 * 1024 * 1024) // 50 MB cap
            .read_to_end(&mut bytes)
            .map_err(|e| anyhow!("failed to read response body: {e}"))?;

        Ok(Self {
            bytes,
            origin: Origin::Url {
                url: url.to_string(),
                content_type,
                is_markdown,
            },
        })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn is_url(&self) -> bool {
        matches!(self.origin, Origin::Url { .. })
    }

    pub fn url(&self) -> Option<&str> {
        match &self.origin {
            Origin::Url { url, .. } => Some(url),
            Origin::File { .. } | Origin::Stdin => None,
        }
    }

    pub fn content_type(&self) -> Option<&str> {
        match &self.origin {
            Origin::Url { content_type, .. } => content_type.as_deref(),
            Origin::File { .. } | Origin::Stdin => None,
        }
    }

    pub fn is_markdown(&self) -> bool {
        match &self.origin {
            Origin::Url { is_markdown, .. } => *is_markdown,
            Origin::File { .. } | Origin::Stdin => false,
        }
    }

    pub fn extension(&self) -> Option<String> {
        match &self.origin {
            Origin::File { path } => path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_string()),
            Origin::Url { url, .. } => {
                // Pull last path segment, take part after final '.'
                let parsed = url::Url::parse(url).ok()?;
                let last_seg = parsed
                    .path_segments()
                    .and_then(|mut s| s.next_back().map(|x| x.to_string()))?;
                last_seg.rsplit_once('.').map(|(_, ext)| ext.to_string())
            }
            Origin::Stdin => None,
        }
    }
}

/// Whether `input` should be fetched as a URL rather than read as a file path.
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

#[cfg(test)]
impl Source {
    /// Build a fetched-URL source without network I/O, for detection tests.
    pub(crate) fn from_url_parts(url: &str, bytes: Vec<u8>, content_type: Option<&str>) -> Self {
        let content_type = content_type.map(str::to_string);
        let is_markdown = content_type
            .as_ref()
            .is_some_and(|ct| ct.starts_with("text/markdown"));
        Self {
            bytes,
            origin: Origin::Url {
                url: url.to_string(),
                content_type,
                is_markdown,
            },
        }
    }
}
