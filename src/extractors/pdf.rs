use crate::source::Source;
use anyhow::{Result, anyhow};

pub fn extract(source: &Source) -> Result<String> {
    let text = pdf_extract::extract_text_from_mem(source.bytes())
        .map_err(|e| anyhow!("pdf-extract failed: {e}"))?;

    if text.trim().is_empty() {
        return Err(anyhow!(
            "no extractable text — PDF appears to be image-only (scanned). \
             OCR is out of scope."
        ));
    }
    Ok(text)
}
