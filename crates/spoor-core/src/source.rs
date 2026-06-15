/// In-memory source metadata consumed by the core parser.
///
/// This type deliberately owns no file handle, URL client, stdin reader, or
/// process state. Adapters resolve inputs and pass bytes plus metadata here.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Source<'a> {
    bytes: &'a [u8],
    source_name: Option<&'a str>,
    content_type: Option<&'a str>,
}

impl<'a> Source<'a> {
    pub(crate) fn new(
        bytes: &'a [u8],
        source_name: Option<&'a str>,
        content_type: Option<&'a str>,
    ) -> Self {
        Self {
            bytes,
            source_name,
            content_type,
        }
    }

    pub(crate) fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    pub(crate) fn is_url(&self) -> bool {
        self.source_name.is_some_and(is_url)
    }

    /// The document's base for resolving relative links, available only when
    /// the source is an absolute http(s) URL. Local paths, stdin (`-`), and
    /// raw library/WASM byte calls return `None`, so relative links are left
    /// verbatim instead of being resolved against a fabricated `file://` base.
    pub(crate) fn url_base(&self) -> Option<&'a str> {
        self.source_name.filter(|name| is_url(name))
    }

    pub(crate) fn content_type(&self) -> Option<&'a str> {
        self.content_type
    }

    pub(crate) fn is_markdown(&self) -> bool {
        self.content_type
            .is_some_and(|content_type| content_type.starts_with("text/markdown"))
    }

    pub(crate) fn extension(&self) -> Option<String> {
        let name = self.source_name?;
        let without_fragment = name.split('#').next().unwrap_or(name);
        let without_query = without_fragment
            .split('?')
            .next()
            .unwrap_or(without_fragment);
        let segment = without_query.rsplit('/').next().unwrap_or(without_query);
        segment
            .rsplit_once('.')
            .map(|(_, extension)| extension.to_string())
    }
}

fn is_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}
