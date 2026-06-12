use crate::error::StructuredError;
use crate::source::Source;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    #[serde(rename = "text")]
    PlainText,
}

impl Format {
    pub fn is_table(self) -> bool {
        matches!(self, Self::Csv | Self::Xlsx)
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

impl FromStr for Format {
    type Err = StructuredError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "html" | "htm" => Ok(Self::Html),
            "markdown" | "md" => Ok(Self::Markdown),
            "pdf" => Ok(Self::Pdf),
            "docx" => Ok(Self::Docx),
            "xlsx" | "xlsm" => Ok(Self::Xlsx),
            "pptx" => Ok(Self::Pptx),
            "csv" | "tsv" => Ok(Self::Csv),
            "ipynb" => Ok(Self::Ipynb),
            "epub" => Ok(Self::Epub),
            "text" | "txt" => Ok(Self::PlainText),
            _ => Err(StructuredError::unsupported_format()),
        }
    }
}

pub(crate) fn detect(source: &Source<'_>) -> Result<Format> {
    if source.is_url() {
        if let Some(content_type) = source.content_type() {
            if content_type.contains("application/pdf") {
                return Ok(Format::Pdf);
            }
            if content_type.contains("application/json") {
                return Ok(Format::PlainText);
            }
        }
    }

    if let Some(format) = detect_by_magic(source.bytes()) {
        return Ok(format);
    }

    if source.bytes().starts_with(CFB_MAGIC) {
        return Err(StructuredError::legacy_or_encrypted_office().into());
    }

    if let Some(extension) = source.extension()
        && let Some(format) = detect_by_ext(&extension)
    {
        return Ok(format);
    }

    if source.is_url() {
        return Ok(Format::Url);
    }

    if looks_like_text(source.bytes()) {
        return Ok(Format::PlainText);
    }

    Err(StructuredError::unsupported_format().into())
}

const CFB_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

fn detect_by_magic(bytes: &[u8]) -> Option<Format> {
    if bytes.starts_with(b"%PDF-") {
        return Some(Format::Pdf);
    }
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        return disambiguate_zip(bytes);
    }
    None
}

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

fn detect_by_ext(extension: &str) -> Option<Format> {
    match extension.to_ascii_lowercase().as_str() {
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
    !sample.contains(&0) && std::str::from_utf8(sample).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_names_match_display_names() {
        for format in [
            Format::Url,
            Format::Html,
            Format::Markdown,
            Format::Pdf,
            Format::Docx,
            Format::Xlsx,
            Format::Pptx,
            Format::Csv,
            Format::Ipynb,
            Format::Epub,
            Format::PlainText,
        ] {
            assert_eq!(
                serde_json::to_string(&format).unwrap(),
                format!("\"{format}\"")
            );
        }
    }

    fn source<'a>(name: &'a str, bytes: &'a [u8], content_type: Option<&'a str>) -> Source<'a> {
        Source::new(bytes, Some(name), content_type)
    }

    #[test]
    fn url_pdf_content_type_wins() {
        let source = source("https://x/doc", b"%PDF-1.7\n", Some("application/pdf"));
        assert_eq!(detect(&source).unwrap(), Format::Pdf);
    }

    #[test]
    fn url_xlsx_detected_by_magic_despite_generic_content_type() {
        let bytes = include_bytes!("../../../spoor-cli/tests/fixtures/xlsx/01_basic.xlsx");
        let source = source(
            "https://x/download",
            bytes,
            Some("application/octet-stream"),
        );
        assert_eq!(detect(&source).unwrap(), Format::Xlsx);
    }

    #[test]
    fn url_extension_detected_when_no_magic() {
        let source = source("https://x/data.csv", b"a,b\n1,2\n", Some("text/plain"));
        assert_eq!(detect(&source).unwrap(), Format::Csv);
    }

    #[test]
    fn url_html_page_falls_back_to_url_format() {
        let source = source(
            "https://x/article",
            b"<html><body>hi</body></html>",
            Some("text/html"),
        );
        assert_eq!(detect(&source).unwrap(), Format::Url);
    }
}
