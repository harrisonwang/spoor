use crate::source::Source;
use anyhow::{Result, anyhow};
use std::fmt;

/// Internal format enum (includes Url).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Url,
    Html,
    Markdown,
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Csv,
    Ipynb,
    Epub,
    PlainText,
}

/// User-facing subset for `--format`. Url isn't here — it's automatic.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum FormatArg {
    Html,
    Markdown,
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Csv,
    Ipynb,
    Epub,
    Text,
}

impl From<FormatArg> for Format {
    fn from(a: FormatArg) -> Self {
        match a {
            FormatArg::Html => Format::Html,
            FormatArg::Markdown => Format::Markdown,
            FormatArg::Pdf => Format::Pdf,
            FormatArg::Docx => Format::Docx,
            FormatArg::Xlsx => Format::Xlsx,
            FormatArg::Pptx => Format::Pptx,
            FormatArg::Csv => Format::Csv,
            FormatArg::Ipynb => Format::Ipynb,
            FormatArg::Epub => Format::Epub,
            FormatArg::Text => Format::PlainText,
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Format::Url => "url",
            Format::Html => "html",
            Format::Markdown => "markdown",
            Format::Pdf => "pdf",
            Format::Docx => "docx",
            Format::Xlsx => "xlsx",
            Format::Pptx => "pptx",
            Format::Csv => "csv",
            Format::Ipynb => "ipynb",
            Format::Epub => "epub",
            Format::PlainText => "text",
        })
    }
}

/// Detection priority: URL > magic bytes > extension > text fallback.
pub fn detect(source: &Source) -> Result<Format> {
    if source.is_url() {
        // Could refine here later via Content-Type sniffing.
        if let Some(ct) = source.content_type() {
            if ct.contains("application/pdf") {
                return Ok(Format::Pdf);
            }
            if ct.contains("application/json") {
                return Ok(Format::PlainText);
            }
        }
        return Ok(Format::Url);
    }

    let bytes = source.bytes();

    if let Some(f) = detect_by_magic(bytes) {
        return Ok(f);
    }

    if let Some(ext) = source.extension() {
        if let Some(f) = detect_by_ext(&ext) {
            return Ok(f);
        }
    }

    if looks_like_text(bytes) {
        return Ok(Format::PlainText);
    }

    Err(anyhow!(
        "unsupported or unknown format (use --format to override)"
    ))
}

fn detect_by_magic(bytes: &[u8]) -> Option<Format> {
    if bytes.starts_with(b"%PDF-") {
        return Some(Format::Pdf);
    }
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        return disambiguate_zip(bytes);
    }
    None
}

/// Peek into a ZIP archive: which OOXML/EPUB is it?
/// We use `file_names()` which returns an iterator of &str.
fn disambiguate_zip(bytes: &[u8]) -> Option<Format> {
    use std::io::Cursor;
    let archive = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    for name in archive.file_names() {
        if name.starts_with("word/") {
            return Some(Format::Docx);
        }
        if name.starts_with("xl/") {
            return Some(Format::Xlsx);
        }
        if name.starts_with("ppt/") {
            return Some(Format::Pptx);
        }
        if name == "mimetype" || name.starts_with("META-INF/") {
            return Some(Format::Epub);
        }
    }
    None
}

fn detect_by_ext(ext: &str) -> Option<Format> {
    match ext.to_ascii_lowercase().as_str() {
        "html" | "htm" => Some(Format::Html),
        "md" | "markdown" => Some(Format::Markdown),
        "pdf" => Some(Format::Pdf),
        "docx" => Some(Format::Docx),
        "xlsx" | "xlsm" => Some(Format::Xlsx),
        "pptx" => Some(Format::Pptx),
        "csv" | "tsv" => Some(Format::Csv),
        "ipynb" => Some(Format::Ipynb),
        "epub" => Some(Format::Epub),
        "txt" | "log" | "rs" | "py" | "js" | "ts" | "go" | "json" | "yaml" | "yml" | "toml"
        | "xml" | "sh" | "c" | "cpp" | "h" | "java" => Some(Format::PlainText),
        _ => None,
    }
}

fn looks_like_text(bytes: &[u8]) -> bool {
    let sample = &bytes[..bytes.len().min(4096)];
    if sample.contains(&0) {
        return false;
    }
    std::str::from_utf8(sample).is_ok()
}
