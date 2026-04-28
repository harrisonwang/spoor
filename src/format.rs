use crate::source::Source;
use anyhow::{anyhow, Result};
use std::fmt;

/// Supported formats. Keep this in sync with extractors::extract().
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    Url,        // remote HTML page (after fetch, falls through to Html)
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

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
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
        };
        f.write_str(s)
    }
}

/// Detection priority:
///   1. URL → Format::Url
///   2. Magic bytes (zip, %PDF, etc.)
///   3. File extension
///   4. Fall back to PlainText for unknown text-looking content
pub fn detect(source: &Source) -> Result<Format> {
    if source.is_url() {
        return Ok(Format::Url);
    }

    let bytes = source.bytes();

    // Magic bytes (most reliable)
    if let Some(f) = detect_by_magic(bytes) {
        // For ZIP, we need extension/content sniffing to disambiguate.
        if f == Format::Epub
            || matches!(f, Format::Docx | Format::Xlsx | Format::Pptx)
        {
            // Already disambiguated inside detect_by_magic.
            return Ok(f);
        }
        return Ok(f);
    }

    // Extension fallback
    if let Some(ext) = source.extension() {
        if let Some(f) = detect_by_ext(&ext) {
            return Ok(f);
        }
    }

    // Last resort: looks like text? → PlainText
    if looks_like_text(bytes) {
        return Ok(Format::PlainText);
    }

    Err(anyhow!(
        "unsupported or unknown format (use --format to override)"
    ))
}

fn detect_by_magic(bytes: &[u8]) -> Option<Format> {
    // PDF: %PDF-
    if bytes.starts_with(b"%PDF-") {
        return Some(Format::Pdf);
    }

    // ZIP container (PK\x03\x04). Could be docx/xlsx/pptx/epub.
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        return disambiguate_zip(bytes);
    }

    None
}

/// Peek into a ZIP archive to figure out which OOXML/EPUB format it is.
/// Looks for sentinel files: word/, xl/, ppt/, mimetype (epub).
fn disambiguate_zip(bytes: &[u8]) -> Option<Format> {
    use std::io::Cursor;
    let cursor = Cursor::new(bytes);
    let archive = zip::ZipArchive::new(cursor).ok()?;
    for i in 0..archive.len() {
        // We only need the file names, not contents.
        let name = archive
            .name_for_index(i)
            .or_else(|| Some(""))
            .unwrap_or("");
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
        "txt" | "log" | "rs" | "py" | "js" | "ts" | "go" | "json" | "yaml" | "yml"
        | "toml" | "xml" | "sh" => Some(Format::PlainText),
        _ => None,
    }
}

fn looks_like_text(bytes: &[u8]) -> bool {
    // Sample first 4 KB; if no NUL bytes and mostly ASCII/UTF-8 valid, treat as text.
    let sample = &bytes[..bytes.len().min(4096)];
    if sample.contains(&0) {
        return false;
    }
    std::str::from_utf8(sample).is_ok()
        || encoding_rs::UTF_8.decode_without_bom_handling(sample).0.len() > 0
}
