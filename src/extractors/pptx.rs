use crate::extractors::xml::attr;
use crate::limits;
use crate::output::MarkdownBuilder;
use crate::source::Source;
use anyhow::{Result, anyhow};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::path::{Component, Path};

pub fn extract(source: &Source, max_parse_bytes: usize) -> Result<String> {
    let mut zip = limits::open_zip_archive(source.bytes(), "pptx", max_parse_bytes)?;

    // Collect ppt/slides/slideN.xml entries, sort by N.
    let mut slides: Vec<(u32, String)> = Vec::new();
    for name in zip.file_names() {
        if let Some(n) = slide_number(name) {
            slides.push((n, name.to_string()));
        }
    }
    slides.sort_by_key(|(n, _)| *n);

    let mut md = MarkdownBuilder::with_max_bytes(max_parse_bytes);
    for (n, name) in &slides {
        md.heading(2, &format!("Slide {n}"));
        let xml = limits::read_zip_text(&mut zip, name, max_parse_bytes)?;
        render_slide(&xml, &mut md)?;
        if let Some(notes_name) = notes_slide_for(&mut zip, name, max_parse_bytes)? {
            let notes_xml = limits::read_zip_text(&mut zip, &notes_name, max_parse_bytes)?;
            render_notes(&notes_xml, &mut md)?;
        }
    }
    md.build()
}

fn slide_number(name: &str) -> Option<u32> {
    name.strip_prefix("ppt/slides/slide")?
        .strip_suffix(".xml")?
        .parse::<u32>()
        .ok()
}

fn render_slide(xml: &str, md: &mut MarkdownBuilder) -> Result<()> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut paragraph = String::new();
    let mut in_table = false;
    let mut in_table_cell = false;
    let mut current_row: Option<Vec<String>> = None;
    let mut current_cell = String::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"tbl" => {
                    in_table = true;
                    table_rows.clear();
                }
                b"tr" if in_table => current_row = Some(Vec::new()),
                b"tc" if in_table => {
                    in_table_cell = true;
                    current_cell.clear();
                }
                _ => {}
            },
            Ok(Event::Text(t)) => {
                let s = t.unescape().map(|c| c.into_owned()).unwrap_or_default();
                if in_table {
                    if in_table_cell {
                        current_cell.push_str(&s);
                    }
                } else {
                    paragraph.push_str(&s);
                }
            }
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"p" if in_table_cell => {
                    current_cell.push(' ');
                }
                b"p" if !in_table && !paragraph.trim().is_empty() => {
                    md.paragraph(&paragraph);
                    paragraph.clear();
                }
                b"tc" if in_table => {
                    if let Some(row) = &mut current_row {
                        row.push(current_cell.trim().to_string());
                    }
                    current_cell.clear();
                    in_table_cell = false;
                }
                b"tr" if in_table => {
                    if let Some(row) = current_row.take() {
                        if !row.is_empty() {
                            table_rows.push(row);
                        }
                    }
                }
                b"tbl" => {
                    md.table(&table_rows);
                    table_rows.clear();
                    in_table = false;
                }
                _ => {}
            },
            Ok(Event::Eof) => {
                if !paragraph.trim().is_empty() {
                    md.paragraph(&paragraph);
                }
                break;
            }
            Err(e) => return Err(anyhow!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

fn render_notes(xml: &str, md: &mut MarkdownBuilder) -> Result<()> {
    let paragraphs = extract_paragraphs(xml)?;
    if paragraphs.is_empty() {
        return Ok(());
    }
    md.paragraph("Notes:");
    for paragraph in paragraphs {
        md.paragraph(&paragraph);
    }
    Ok(())
}

fn extract_paragraphs(xml: &str) -> Result<Vec<String>> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut paragraph = String::new();
    let mut paragraphs = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(t)) => {
                let s = t.unescape().map(|c| c.into_owned()).unwrap_or_default();
                paragraph.push_str(&s);
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"p" => {
                let trimmed = paragraph.trim();
                if !trimmed.is_empty() {
                    paragraphs.push(trimmed.to_string());
                }
                paragraph.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok(paragraphs)
}

fn notes_slide_for<R: std::io::Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    slide_name: &str,
    max_parse_bytes: usize,
) -> Result<Option<String>> {
    let Some(file_name) = Path::new(slide_name).file_name().and_then(|s| s.to_str()) else {
        return Ok(None);
    };
    let rels_name = format!("ppt/slides/_rels/{file_name}.rels");
    let rels_xml = match limits::read_zip_text_optional(zip, &rels_name, max_parse_bytes)? {
        Some(xml) => xml,
        None => return Ok(None),
    };
    let Some(target) = parse_notes_target(&rels_xml) else {
        return Ok(None);
    };
    let base = Path::new(slide_name)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    Ok(Some(normalize_zip_path(base.join(target))))
}

fn parse_notes_target(xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e))
                if e.local_name().as_ref() == b"Relationship" =>
            {
                let rel_type = attr(&e, b"Type")?;
                if rel_type.ends_with("/notesSlide") {
                    return attr(&e, b"Target");
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

fn normalize_zip_path(path: impl AsRef<Path>) -> String {
    let mut parts = Vec::new();
    for component in path.as_ref().components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::ParentDir => {
                parts.pop();
            }
            Component::CurDir => {}
            _ => {}
        }
    }
    parts.join("/")
}
