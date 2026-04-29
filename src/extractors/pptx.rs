use crate::output::MarkdownBuilder;
use crate::source::Source;
use anyhow::{Context, Result, anyhow};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::io::{Cursor, Read};
use std::path::{Component, Path};

pub fn extract(source: &Source) -> Result<String> {
    let cursor = Cursor::new(source.bytes());
    let mut zip = zip::ZipArchive::new(cursor).context("failed to open pptx")?;

    // Collect ppt/slides/slideN.xml entries, sort by N.
    let mut slides: Vec<(u32, String)> = Vec::new();
    for name in zip.file_names() {
        if let Some(n) = slide_number(name) {
            slides.push((n, name.to_string()));
        }
    }
    slides.sort_by_key(|(n, _)| *n);

    let mut md = MarkdownBuilder::new();
    for (n, name) in &slides {
        md.heading(2, &format!("Slide {n}"));
        let xml = read_zip_text(&mut zip, name)?;
        render_slide(&xml, &mut md)?;
        if let Some(notes_name) = notes_slide_for(&mut zip, name)? {
            let notes_xml = read_zip_text(&mut zip, &notes_name)?;
            render_notes(&notes_xml, &mut md)?;
        }
    }
    Ok(md.build())
}

fn read_zip_text<R: std::io::Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<String> {
    let mut file = zip.by_name(name)?;
    let mut xml = String::new();
    file.read_to_string(&mut xml)?;
    Ok(xml)
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
                        row.push(sanitize_cell(&current_cell));
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
                    render_table(md, &table_rows);
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

fn render_table(md: &mut MarkdownBuilder, rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    if cols == 0 {
        return;
    }

    md.blank_line();
    md.raw(&format!("| {} |\n", pad_row(&rows[0], cols).join(" | ")));
    md.raw(&format!("| {} |\n", vec!["---"; cols].join(" | ")));
    for row in rows.iter().skip(1) {
        md.raw(&format!("| {} |\n", pad_row(row, cols).join(" | ")));
    }
}

fn pad_row(row: &[String], cols: usize) -> Vec<String> {
    let mut padded = row.to_vec();
    while padded.len() < cols {
        padded.push(String::new());
    }
    padded
}

fn sanitize_cell(text: &str) -> String {
    text.trim()
        .replace('|', "\\|")
        .replace(['\n', '\r', '\t'], " ")
}

fn notes_slide_for<R: std::io::Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    slide_name: &str,
) -> Result<Option<String>> {
    let Some(file_name) = Path::new(slide_name).file_name().and_then(|s| s.to_str()) else {
        return Ok(None);
    };
    let rels_name = format!("ppt/slides/_rels/{file_name}.rels");
    let rels_xml = match read_zip_text(zip, &rels_name) {
        Ok(xml) => xml,
        Err(_) => return Ok(None),
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

fn attr(e: &BytesStart, local_name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.local_name().as_ref() == local_name)
        .and_then(|a| String::from_utf8(a.value.into_owned()).ok())
}
