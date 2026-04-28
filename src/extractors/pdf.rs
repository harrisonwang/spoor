use crate::source::Source;
use anyhow::{anyhow, Result};

/// Text-layer extraction only. Image-only PDFs (scans) will produce empty
/// or garbage output — we detect that and return a clear error.
pub fn extract(source: &Source) -> Result<String> {
    let text = pdf_extract::extract_text_from_mem(source.bytes())
        .map_err(|e| anyhow!("pdf-extract failed: {}", e))?;

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(anyhow!(
            "no extractable text — this PDF appears to be image-only (scanned). \
             OCR is not supported in gist."
        ));
    }

    // pdf-extract gives us paragraphs separated by blank lines; mostly fine.
    // We could add light cleanup here (de-hyphenation, ligature fixing) later.
    Ok(text)
}
