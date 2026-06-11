use crate::error::StructuredError;
use anyhow::{Context, Result, bail};
use std::io::{Cursor, Read, Seek};
use zip::read::ZipArchive;
use zip::result::ZipError;

pub(crate) const DEFAULT_MAX_ZIP_ENTRIES: usize = 10_000;
pub(crate) const DEFAULT_MAX_ZIP_ENTRY_BYTES: usize = 50 * 1024 * 1024;
pub(crate) const DEFAULT_MAX_ZIP_COMPRESSION_RATIO: u64 = 200;
pub const DEFAULT_MAX_PARSE_BYTES: usize = 64 * 1024 * 1024;
pub const MIN_MAX_PARSE_BYTES: usize = 1024;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Limits {
    pub max_zip_entries: usize,
    pub max_zip_entry_bytes: usize,
    pub max_zip_compression_ratio: u64,
    pub max_zip_total_bytes: usize,
}

impl Limits {
    fn for_parse_budget(max_parse_bytes: usize) -> Self {
        Self {
            max_zip_entries: DEFAULT_MAX_ZIP_ENTRIES,
            max_zip_entry_bytes: DEFAULT_MAX_ZIP_ENTRY_BYTES,
            max_zip_compression_ratio: DEFAULT_MAX_ZIP_COMPRESSION_RATIO,
            max_zip_total_bytes: max_parse_bytes,
        }
    }
}

pub(crate) type ZipReader<'a> = ZipArchive<Cursor<&'a [u8]>>;

pub(crate) fn open_zip_archive<'a>(
    bytes: &'a [u8],
    label: &str,
    max_parse_bytes: usize,
) -> Result<ZipReader<'a>> {
    // An unreadable archive (empty file, truncated download, fake extension)
    // gets a structured, branchable error. Safety-check failures below keep
    // their own errors — notably the structured parse-budget one — and must
    // not be collapsed into `invalid_container`.
    let mut zip = ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| StructuredError::invalid_container(label))?;
    validate_zip_archive(&mut zip, Limits::for_parse_budget(max_parse_bytes))
        .with_context(|| format!("{label} archive failed safety checks"))?;
    Ok(zip)
}

pub(crate) fn read_zip_text<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    name: &str,
    max_parse_bytes: usize,
) -> Result<String> {
    let bytes = read_zip_bytes(zip, name, max_parse_bytes)?;
    String::from_utf8(bytes).with_context(|| format!("zip entry is not UTF-8 text: {name}"))
}

pub(crate) fn read_zip_text_optional<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    name: &str,
    max_parse_bytes: usize,
) -> Result<Option<String>> {
    read_zip_bytes_optional(zip, name, max_parse_bytes)?
        .map(|bytes| {
            String::from_utf8(bytes).with_context(|| format!("zip entry is not UTF-8 text: {name}"))
        })
        .transpose()
}

pub(crate) fn read_zip_bytes<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    name: &str,
    max_parse_bytes: usize,
) -> Result<Vec<u8>> {
    read_zip_bytes_optional(zip, name, max_parse_bytes)?
        .with_context(|| format!("zip entry not found: {name}"))
}

pub(crate) fn read_zip_bytes_optional<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    name: &str,
    max_parse_bytes: usize,
) -> Result<Option<Vec<u8>>> {
    let file = match zip.by_name(name) {
        Ok(file) => file,
        Err(ZipError::FileNotFound) => return Ok(None),
        Err(e) => return Err(e).with_context(|| format!("failed to open zip entry: {name}")),
    };

    read_limited_zip_file(file, Limits::for_parse_budget(max_parse_bytes))
        .map(Some)
        .with_context(|| format!("failed to read zip entry: {name}"))
}

fn validate_zip_archive<R: Read + Seek>(zip: &mut ZipArchive<R>, limits: Limits) -> Result<()> {
    if zip.len() > limits.max_zip_entries {
        bail!(
            "zip entry count {} exceeds limit {}",
            zip.len(),
            limits.max_zip_entries
        );
    }

    let mut total_uncompressed = 0u64;
    for idx in 0..zip.len() {
        let file = zip
            .by_index(idx)
            .with_context(|| format!("failed to inspect zip entry #{idx}"))?;
        validate_zip_entry(file.name(), file.compressed_size(), file.size(), limits)?;
        total_uncompressed = total_uncompressed.checked_add(file.size()).ok_or_else(|| {
            StructuredError::parse_memory_limit(
                limits.max_zip_total_bytes,
                "ZIP archive inspection",
            )
        })?;
        if total_uncompressed > limits.max_zip_total_bytes as u64 {
            return Err(StructuredError::parse_memory_limit(
                limits.max_zip_total_bytes,
                "ZIP archive decompression",
            )
            .into());
        }
    }

    Ok(())
}

pub(crate) fn ensure_parse_size(actual: usize, max_bytes: usize, stage: &str) -> Result<()> {
    if actual > max_bytes {
        return Err(StructuredError::parse_memory_limit(max_bytes, stage).into());
    }
    Ok(())
}

fn read_limited_zip_file(file: zip::read::ZipFile<'_>, limits: Limits) -> Result<Vec<u8>> {
    validate_zip_entry(file.name(), file.compressed_size(), file.size(), limits)?;

    let mut bytes = Vec::new();
    let read_limit = limits.max_zip_entry_bytes.min(limits.max_zip_total_bytes);
    let mut limited = file.take(
        u64::try_from(read_limit)
            .unwrap_or(u64::MAX)
            .saturating_add(1),
    );
    limited.read_to_end(&mut bytes)?;
    if bytes.len() > limits.max_zip_total_bytes {
        return Err(StructuredError::parse_memory_limit(
            limits.max_zip_total_bytes,
            "ZIP entry decompression",
        )
        .into());
    }
    if bytes.len() > limits.max_zip_entry_bytes {
        bail!(
            "zip entry exceeds decompressed size limit of {} bytes",
            limits.max_zip_entry_bytes
        );
    }
    Ok(bytes)
}

fn validate_zip_entry(
    name: &str,
    compressed_size: u64,
    uncompressed_size: u64,
    limits: Limits,
) -> Result<()> {
    if uncompressed_size > limits.max_zip_total_bytes as u64 {
        return Err(StructuredError::parse_memory_limit(
            limits.max_zip_total_bytes,
            "ZIP entry inspection",
        )
        .into());
    }
    if uncompressed_size > limits.max_zip_entry_bytes as u64 {
        bail!(
            "zip entry {name} size {uncompressed_size} exceeds limit {}",
            limits.max_zip_entry_bytes
        );
    }

    if compressed_size == 0 {
        if uncompressed_size > 0 {
            bail!("zip entry {name} has non-empty content with zero compressed size");
        }
        return Ok(());
    }

    let ratio = uncompressed_size / compressed_size;
    if ratio > limits.max_zip_compression_ratio {
        bail!(
            "zip entry {name} compression ratio {ratio} exceeds limit {}",
            limits.max_zip_compression_ratio
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::open_zip_archive;
    use crate::error::StructuredError;
    use std::io::{Cursor, Write};

    #[test]
    fn zip_total_uncompressed_size_respects_parse_budget() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer.start_file("a.txt", options).unwrap();
        writer.write_all(&vec![b'a'; 700]).unwrap();
        writer.start_file("b.txt", options).unwrap();
        writer.write_all(&vec![b'b'; 700]).unwrap();
        let bytes = writer.finish().unwrap().into_inner();

        let error = match open_zip_archive(&bytes, "test", 1024) {
            Ok(_) => panic!("expected ZIP total size limit error"),
            Err(error) => error,
        };
        assert!(error.downcast_ref::<StructuredError>().is_some());
    }
}
