use crate::error::StructuredError;
use crate::limits;
use crate::parse::ExtractedMarkdown;
use crate::result::{SpoorWarning, WarningCode};
use crate::source::Source;
use anyhow::{Result, anyhow};

pub fn extract(source: &Source<'_>, max_parse_bytes: usize) -> Result<ExtractedMarkdown> {
    let pages =
        super::pdf_engine::extract_text_from_mem_by_pages(source.bytes()).map_err(map_pdf_error)?;

    if pages.iter().all(|page| page.trim().is_empty()) {
        return Err(StructuredError::image_only_pdf().into());
    }

    let rendered_bytes = pages
        .iter()
        .enumerate()
        .fold(0usize, |total, (index, page)| {
            total
                .saturating_add(if index > 0 { "\n\n".len() } else { 0 })
                .saturating_add(format!("## Page {}\n\n", index + 1).len())
                .saturating_add(page.trim().len())
        });
    limits::ensure_parse_size(rendered_bytes, max_parse_bytes, "PDF Markdown rendering")?;

    Ok(ExtractedMarkdown {
        markdown: render_pages(&pages),
        warnings: page_warnings(&pages),
    })
}

fn render_pages(pages: &[String]) -> String {
    let mut markdown = String::new();

    for (index, page) in pages.iter().enumerate() {
        if index > 0 {
            markdown.push_str("\n\n");
        }

        markdown.push_str(&format!("## Page {}\n\n", index + 1));
        markdown.push_str(page.trim());
    }

    markdown
}

fn page_warnings(pages: &[String]) -> Vec<SpoorWarning> {
    let mut warnings = Vec::new();
    for (index, page) in pages.iter().enumerate() {
        let number = index + 1;
        if page.trim().is_empty() {
            warnings.push(SpoorWarning::at_page(
                WarningCode::PdfPageNoTextLayer,
                format!(
                    "第 {number} 页没有可提取文本层；输出保留了页边界，但 Agent 不应把该页视为完整内容。"
                ),
                number,
            ));
        } else if suspicious_text_layer(page) {
            warnings.push(SpoorWarning::at_page(
                WarningCode::PdfPageSuspiciousTextLayer,
                format!(
                    "第 {number} 页文本层包含替换字符、控制字符或重复 glyph 占位符；Agent 应避免直接信任该页文本，并按需转交外部 OCR/VLM。"
                ),
                number,
            ));
        }
    }
    warnings
}

fn suspicious_text_layer(page: &str) -> bool {
    if page.chars().any(|character| {
        character == '\u{fffd}' || (character.is_control() && !character.is_whitespace())
    }) {
        return true;
    }

    let lower = page.to_ascii_lowercase();
    lower.match_indices("glyph<").nth(1).is_some()
}

/// A password-protected PDF is a hard boundary like an image-only one: no
/// retry or flag can succeed, so it gets a structured, branchable error
/// instead of the library's misleading "password is incorrect" text (lopdf
/// probes with an empty password the user never supplied).
fn map_pdf_error(error: super::pdf_engine::OutputError) -> anyhow::Error {
    match error {
        super::pdf_engine::OutputError::PdfError(super::pdf_engine::Error::Decryption(_)) => {
            StructuredError::encrypted_pdf().into()
        }
        error => anyhow!("pdf-extract failed: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{map_pdf_error, page_warnings, render_pages, suspicious_text_layer};
    use crate::error::{ErrorCode, StructuredError};
    use crate::result::{WarningCode, WarningLocation};

    #[test]
    fn page_boundaries_preserve_blank_pages() {
        let pages = vec!["first".into(), " \n".into(), "third".into()];

        assert_eq!(
            render_pages(&pages),
            "## Page 1\n\nfirst\n\n## Page 2\n\n\n\n## Page 3\n\nthird"
        );
    }

    #[test]
    fn decryption_failure_maps_to_encrypted_pdf() {
        let error = map_pdf_error(super::super::pdf_engine::OutputError::PdfError(
            super::super::pdf_engine::Error::Decryption(
                super::super::pdf_engine::encryption::DecryptionError::IncorrectPassword,
            ),
        ));

        let structured = error
            .downcast_ref::<StructuredError>()
            .expect("structured error");
        assert_eq!(structured.code, ErrorCode::EncryptedPdf);
        assert!(!structured.recoverable);
    }

    #[test]
    fn other_pdf_errors_stay_unstructured() {
        let error = map_pdf_error(super::super::pdf_engine::OutputError::FormatError(
            std::fmt::Error,
        ));

        assert!(error.downcast_ref::<StructuredError>().is_none());
        assert!(error.to_string().contains("pdf-extract failed"));
    }

    #[test]
    fn mixed_pdf_reports_blank_page_without_failing_document() {
        let warnings = page_warnings(&["text".into(), " \n".into(), "more".into()]);

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, WarningCode::PdfPageNoTextLayer);
        assert_eq!(
            warnings[0].location,
            Some(WarningLocation::Page { number: 2 })
        );
    }

    #[test]
    fn suspicious_text_detection_is_conservative() {
        assert!(suspicious_text_layer("GLYPH<28> GLYPH<27>"));
        assert!(suspicious_text_layer("valid text \u{fffd}"));
        assert!(!suspicious_text_layer("正常中文、代码 glyph<T> 与正文"));
    }
}
