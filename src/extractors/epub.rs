use crate::output::decode_text;
use crate::source::Source;
use anyhow::{anyhow, Context, Result};
use std::io::{Cursor, Read};

/// Skeleton implementation: read all xhtml files in spine order,
/// strip tags, concatenate. Real implementation should:
///   1. Parse META-INF/container.xml → find OPF path
///   2. Parse OPF → read <spine> for ordering, <manifest> for files
///   3. Parse each xhtml with proper markdown conversion (headings,
///      lists, links preserved like docx does)
pub fn extract(source: &Source) -> Result<String> {
    let cursor = Cursor::new(source.bytes());
    let mut zip = zip::ZipArchive::new(cursor).context("failed to open epub")?;

    // Step 1: container.xml → rootfile path
    let opf_path = {
        let mut f = zip
            .by_name("META-INF/container.xml")
            .map_err(|_| anyhow!("epub: missing META-INF/container.xml"))?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        find_rootfile_path(&s)
            .ok_or_else(|| anyhow!("epub: no <rootfile full-path> in container.xml"))?
    };

    // Step 2: parse OPF for spine order (TODO — for now just enumerate
    // all xhtml files alphabetically as a placeholder).
    let _ = opf_path;
    let mut xhtml_names: Vec<String> = (0..zip.len())
        .filter_map(|i| {
            let name = zip.by_index(i).ok()?.name().to_string();
            if name.ends_with(".xhtml") || name.ends_with(".html") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    xhtml_names.sort();

    let mut out = String::new();
    for name in xhtml_names {
        let mut f = zip.by_name(&name)?;
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes)?;
        let html = decode_text(&bytes);
        out.push_str(&strip_html(&html));
        out.push('\n');
    }
    Ok(out)
}

fn find_rootfile_path(container_xml: &str) -> Option<String> {
    // Simple regex-free search: locate full-path="..."
    let needle = "full-path=\"";
    let start = container_xml.find(needle)? + needle.len();
    let rest = &container_xml[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn strip_html(html: &str) -> String {
    use scraper::{Html, Selector};
    let doc = Html::parse_document(html);
    let body = Selector::parse("body").unwrap();
    let mut out = String::new();
    for b in doc.select(&body) {
        for t in b.text() {
            let t = t.trim();
            if !t.is_empty() {
                out.push_str(t);
                out.push('\n');
            }
        }
    }
    out
}
