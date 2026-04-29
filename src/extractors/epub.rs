use crate::output::decode_text;
use crate::source::Source;
use anyhow::{Context, Result, anyhow};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

pub fn extract(source: &Source) -> Result<String> {
    let cursor = Cursor::new(source.bytes());
    let mut zip = zip::ZipArchive::new(cursor).context("failed to open epub")?;

    let container_xml = read_zip_text(&mut zip, "META-INF/container.xml")
        .ok_or_else(|| anyhow!("epub: missing META-INF/container.xml"))?;
    let opf_path = parse_rootfile_path(&container_xml)
        .ok_or_else(|| anyhow!("epub: missing OPF rootfile in container.xml"))?;
    let opf_xml = read_zip_text(&mut zip, &opf_path)
        .ok_or_else(|| anyhow!("epub: missing OPF package: {opf_path}"))?;

    let spine = parse_spine_paths(&opf_xml, &opf_path);
    if spine.is_empty() {
        return Err(anyhow!("epub: no readable XHTML items in OPF spine"));
    }

    let mut out = String::new();
    for name in spine {
        let mut f = zip.by_name(&name)?;
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes)?;
        let html = decode_text(&bytes);
        out.push_str(&strip_html(&html));
        out.push('\n');
    }
    Ok(out)
}

fn read_zip_text<R: std::io::Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
) -> Option<String> {
    let mut file = zip.by_name(name).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some(decode_text(&bytes))
}

fn parse_rootfile_path(xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) if e.local_name().as_ref() == b"rootfile" => {
                if let Some(path) = attr(&e, b"full-path") {
                    return Some(path);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

fn parse_spine_paths(xml: &str, opf_path: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut manifest: HashMap<String, String> = HashMap::new();
    let mut spine_ids = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => match e.local_name().as_ref() {
                b"item" => {
                    if let (Some(id), Some(href), Some(media_type)) =
                        (attr(&e, b"id"), attr(&e, b"href"), attr(&e, b"media-type"))
                    {
                        if media_type == "application/xhtml+xml"
                            || href.ends_with(".xhtml")
                            || href.ends_with(".html")
                        {
                            manifest.insert(id, href);
                        }
                    }
                }
                b"itemref" => {
                    if let Some(idref) = attr(&e, b"idref") {
                        spine_ids.push(idref);
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    spine_ids
        .into_iter()
        .filter_map(|id| {
            manifest
                .get(&id)
                .map(|href| join_opf_relative(opf_path, href))
        })
        .collect()
}

fn join_opf_relative(opf_path: &str, href: &str) -> String {
    let base = Path::new(opf_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(PathBuf::new);
    base.join(href).to_string_lossy().replace('\\', "/")
}

fn strip_html(html: &str) -> String {
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

fn attr(e: &quick_xml::events::BytesStart, local_name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.local_name().as_ref() == local_name)
        .and_then(|a| String::from_utf8(a.value.into_owned()).ok())
}
