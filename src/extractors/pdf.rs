use crate::source::Source;
use anyhow::{Result, anyhow};

pub fn extract(source: &Source) -> Result<String> {
    let text = extract_text_quietly(source.bytes())?;

    if text.trim().is_empty() {
        return Err(anyhow!(
            "no extractable text — PDF appears to be image-only (scanned). \
             OCR is out of scope."
        ));
    }
    Ok(text)
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
fn extract_text_quietly(bytes: &[u8]) -> Result<String> {
    // `Gag` redirects fd 1 to /dev/null and restores it on drop. If gagging
    // fails for any reason, fall back to extracting with the noise rather than
    // failing the whole command.
    let _gag = gag::Gag::stdout().ok();
    pdf_extract::extract_text_from_mem(bytes).map_err(|e| anyhow!("pdf-extract failed: {e}"))
}

#[cfg(not(unix))]
fn extract_text_quietly(bytes: &[u8]) -> Result<String> {
    pdf_extract::extract_text_from_mem(bytes).map_err(|e| anyhow!("pdf-extract failed: {e}"))
}
