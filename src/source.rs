use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

/// A resolved input: either local file bytes or a fetched URL.
pub struct Source {
    bytes: Vec<u8>,
    origin: Origin,
}

enum Origin {
    File { path: PathBuf },
    Url { url: String, content_type: Option<String> },
}

impl Source {
    pub fn resolve(input: &str) -> Result<Self> {
        if is_url(input) {
            Self::fetch_url(input)
        } else {
            Self::read_file(input)
        }
    }

    fn read_file(path: &str) -> Result<Self> {
        let pb = PathBuf::from(path);
        let bytes = std::fs::read(&pb)
            .with_context(|| format!("failed to read file: {}", path))?;
        Ok(Self { bytes, origin: Origin::File { path: pb } })
    }

    fn fetch_url(url: &str) -> Result<Self> {
        let resp = ureq::get(url)
            .set("User-Agent", "gist/0.1 (+https://github.com/yourname/gist)")
            .timeout(std::time::Duration::from_secs(30))
            .call()
            .with_context(|| format!("failed to fetch URL: {}", url))?;

        let content_type = resp.header("content-type").map(|s| s.to_string());
        let mut bytes = Vec::new();
        resp.into_reader()
            .take(50 * 1024 * 1024) // 50 MB cap on URL fetches
            .read_to_end(&mut bytes)
            .map_err(|e| anyhow!("failed to read response body: {}", e))?;

        Ok(Self {
            bytes,
            origin: Origin::Url { url: url.to_string(), content_type },
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
            Origin::File { .. } => None,
        }
    }

    pub fn content_type(&self) -> Option<&str> {
        match &self.origin {
            Origin::Url { content_type, .. } => content_type.as_deref(),
            Origin::File { .. } => None,
        }
    }

    pub fn extension(&self) -> Option<String> {
        match &self.origin {
            Origin::File { path } => path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_string()),
            Origin::Url { url, .. } => url::Url::parse(url)
                .ok()
                .and_then(|u| {
                    u.path_segments()
                        .and_then(|segs| segs.last().map(|s| s.to_string()))
                })
                .and_then(|seg| {
                    seg.rsplit('.').next().map(|s| s.to_string())
                }),
        }
    }
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

use std::io::Read;
