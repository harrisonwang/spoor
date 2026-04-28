use crate::output::MarkdownBuilder;
use crate::source::Source;
use anyhow::{anyhow, Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::NsReader;
use std::io::{Cursor, Read};

const NS_A: &[u8] = b"http://schemas.openxmlformats.org/drawingml/2006/main";

/// One "## Slide N" header per slide. All <a:t> text nodes are concatenated
/// per text frame, then frames separated by blank lines.
pub fn extract(source: &Source) -> Result<String> {
    let cursor = Cursor::new(source.bytes());
    let mut zip = zip::ZipArchive::new(cursor).context("failed to open pptx")?;

    // Collect ppt/slides/slideN.xml entries, sort by N.
    let mut slides: Vec<(u32, String)> = Vec::new();
    for i in 0..zip.len() {
        let name = zip.by_index(i)?.name().to_string();
        if let Some(n) = slide_number(&name) {
            slides.push((n, name));
        }
    }
    slides.sort_by_key(|(n, _)| *n);

    let mut md = MarkdownBuilder::new();
    for (n, name) in &slides {
        md.heading(2, &format!("Slide {}", n));
        let mut file = zip.by_name(name)?;
        let mut xml = String::new();
        file.read_to_string(&mut xml)?;
        render_slide(&xml, &mut md)?;
    }
    Ok(md.build())
}

fn slide_number(name: &str) -> Option<u32> {
    let stem = name
        .strip_prefix("ppt/slides/slide")
        .and_then(|s| s.strip_suffix(".xml"))?;
    stem.parse::<u32>().ok()
}

fn render_slide(xml: &str, md: &mut MarkdownBuilder) -> Result<()> {
    let mut reader = NsReader::from_str(xml);
    let mut buf = Vec::new();
    let mut current_frame_text = String::new();

    loop {
        match reader.read_resolved_event_into(&mut buf) {
            Ok((quick_xml::name::ResolveResult::Bound(ns), Event::Text(t)))
                if ns.as_ref() == NS_A =>
            {
                let s = t.unescape().unwrap_or_default().into_owned();
                current_frame_text.push_str(&s);
            }
            Ok((_, Event::End(e))) => {
                // Each <a:p> is a paragraph within a text frame; emit a newline.
                if e.local_name().as_ref() == b"p" {
                    if !current_frame_text.is_empty() {
                        md.paragraph(&current_frame_text);
                        current_frame_text.clear();
                    }
                }
            }
            Ok((_, Event::Eof)) => break,
            Err(e) => return Err(anyhow!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}
