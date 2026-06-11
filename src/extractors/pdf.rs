use crate::error::StructuredError;
use crate::limits;
use crate::source::Source;
use anyhow::{Result, anyhow};

pub fn extract(source: &Source, max_parse_bytes: usize) -> Result<String> {
    let pages = extract_pages_quietly(source.bytes())?;

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

    Ok(render_pages(&pages))
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

/// pdf-extract prints `unknown glyph name '...'` straight to stdout via
/// `println!` (pdf-extract 0.7 src/lib.rs:479) whenever a font glyph name can't
/// be resolved to Unicode. For subset-embedded CJK fonts (e.g. PingFang SC) this
/// emits tens of thousands of lines that would pollute our own markdown output.
/// The text itself is recovered through the font's ToUnicode CMap, so the
/// returned String is unaffected — we just need to gag stdout for the duration
/// of the call. On non-Unix platforms `gag` is unavailable, so we accept the
/// noise there.
#[cfg(unix)]
fn extract_pages_quietly(bytes: &[u8]) -> Result<Vec<String>> {
    // `Gag` redirects fd 1 to /dev/null and restores it on drop. If gagging
    // fails for any reason, fall back to extracting with the noise rather than
    // failing the whole command.
    let _gag = gag::Gag::stdout().ok();
    pdf_extract::extract_text_from_mem_by_pages(bytes).map_err(map_pdf_error)
}

#[cfg(not(unix))]
fn extract_pages_quietly(bytes: &[u8]) -> Result<Vec<String>> {
    pdf_extract::extract_text_from_mem_by_pages(bytes).map_err(map_pdf_error)
}

/// A password-protected PDF is a hard boundary like an image-only one: no
/// retry or flag can succeed, so it gets a structured, branchable error
/// instead of the library's misleading "password is incorrect" text (lopdf
/// probes with an empty password the user never supplied).
fn map_pdf_error(error: pdf_extract::OutputError) -> anyhow::Error {
    match error {
        pdf_extract::OutputError::PdfError(pdf_extract::Error::Decryption(_)) => {
            StructuredError::encrypted_pdf().into()
        }
        error => anyhow!("pdf-extract failed: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{map_pdf_error, render_pages};
    use crate::error::{ErrorCode, StructuredError};

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
        let error = map_pdf_error(pdf_extract::OutputError::PdfError(
            pdf_extract::Error::Decryption(
                pdf_extract::encryption::DecryptionError::IncorrectPassword,
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
        let error = map_pdf_error(pdf_extract::OutputError::FormatError(std::fmt::Error));

        assert!(error.downcast_ref::<StructuredError>().is_none());
        assert!(error.to_string().contains("pdf-extract failed"));
    }
}
