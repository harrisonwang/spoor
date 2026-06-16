use crate::error::StructuredError;
use crate::limits;
use crate::parse::ExtractedMarkdown;
use crate::parse::pdf_media::{self, PageImage};
use crate::result::{SpoorWarning, WarningCode};
use crate::source::Source;
use anyhow::{Result, anyhow};

pub fn extract(source: &Source<'_>, max_parse_bytes: usize) -> Result<ExtractedMarkdown> {
    let pages =
        super::pdf_engine::extract_text_from_mem_by_pages(source.bytes()).map_err(map_pdf_error)?;

    // Best-effort: locate image XObjects so the renderer can mark their page
    // position and warn. A discovery failure just yields no image markers.
    let images = pdf_media::discover_images(source.bytes(), pages.len());

    // A PDF with no text layer is only a dead end when it also has no images to
    // hand off. When it *does* (a scan or an exported diagram), surface the page
    // skeleton plus image markers/handles and let the agent read them with a
    // vision model — hard-failing here would block exactly that handoff.
    let has_text = pages.iter().any(|page| !page.trim().is_empty());
    let has_images = images.iter().any(|page| !page.is_empty());
    if !has_text && !has_images {
        return Err(StructuredError::image_only_pdf().into());
    }

    let markdown = render_pages(&pages, &images);
    limits::ensure_parse_size(markdown.len(), max_parse_bytes, "PDF Markdown rendering")?;

    Ok(ExtractedMarkdown {
        markdown,
        warnings: page_warnings(&pages, &images),
    })
}

fn render_pages(pages: &[String], images: &[Vec<PageImage>]) -> String {
    let mut markdown = String::new();
    let mut image_number = 0usize;

    for (index, page) in pages.iter().enumerate() {
        if index > 0 {
            markdown.push_str("\n\n");
        }

        let number = index + 1;
        markdown.push_str(&format!("## Page {number}\n\n"));
        markdown.push_str(page.trim());

        for image in images.get(index).map(Vec::as_slice).unwrap_or_default() {
            image_number += 1;
            markdown.push_str("\n\n");
            if image.extractable {
                // A real handle: `--extract` returns the JPEG/JPEG2000 bytes.
                markdown.push_str(&format!(
                    "![PDF image {image_number} (p{number})](spoor-pdf://obj/{}/{})",
                    image.id, image.generation
                ));
            } else {
                // Present but not directly extractable; mark position only so
                // the agent knows the page is more than its text.
                markdown.push_str(&format!(
                    "[PDF image {image_number} (p{number})：内嵌图，编码需外部渲染]"
                ));
            }
        }
    }

    markdown
}

fn page_warnings(pages: &[String], images: &[Vec<PageImage>]) -> Vec<SpoorWarning> {
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

        let page_images = images.get(index).map(Vec::as_slice).unwrap_or_default();
        if !page_images.is_empty() {
            let total = page_images.len();
            let extractable = page_images.iter().filter(|image| image.extractable).count();
            let message = if extractable == total {
                format!(
                    "第 {number} 页含 {total} 张图片，未进入文本；已用 spoor-pdf:// 标注，Agent 可用 --extract 取出交给视觉模型。"
                )
            } else if extractable == 0 {
                format!(
                    "第 {number} 页含 {total} 张图片，未进入文本，且编码 spoor 不能直出；请在外部渲染该页后交给视觉模型。"
                )
            } else {
                format!(
                    "第 {number} 页含 {total} 张图片，未进入文本；其中 {extractable} 张可用 --extract 取出（已标 spoor-pdf://），其余需外部渲染。"
                )
            };
            warnings.push(SpoorWarning::at_page(
                WarningCode::EmbeddedVisualsOmitted,
                message,
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
        let images = vec![Vec::new(); pages.len()];

        assert_eq!(
            render_pages(&pages, &images),
            "## Page 1\n\nfirst\n\n## Page 2\n\n\n\n## Page 3\n\nthird"
        );
    }

    #[test]
    fn extractable_image_renders_handle_others_render_marker() {
        let pages = vec!["text".to_string()];
        let images = vec![vec![
            super::PageImage {
                id: 7,
                generation: 0,
                extractable: true,
            },
            super::PageImage {
                id: 9,
                generation: 0,
                extractable: false,
            },
        ]];

        let markdown = render_pages(&pages, &images);
        assert!(markdown.contains("![PDF image 1 (p1)](spoor-pdf://obj/7/0)"));
        assert!(markdown.contains("[PDF image 2 (p1)：内嵌图，编码需外部渲染]"));
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
        let pages = vec!["text".to_string(), " \n".to_string(), "more".to_string()];
        let warnings = page_warnings(&pages, &vec![Vec::new(); pages.len()]);

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, WarningCode::PdfPageNoTextLayer);
        assert_eq!(
            warnings[0].location,
            Some(WarningLocation::Page { number: 2 })
        );
    }

    #[test]
    fn pages_with_images_warn_with_page_location() {
        let pages = vec!["text".to_string(), "more".to_string()];
        let images = vec![
            Vec::new(),
            vec![super::PageImage {
                id: 5,
                generation: 0,
                extractable: false,
            }],
        ];

        let warnings = page_warnings(&pages, &images);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, WarningCode::EmbeddedVisualsOmitted);
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
