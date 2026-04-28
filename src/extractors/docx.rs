use crate::output::MarkdownBuilder;
use crate::source::Source;
use anyhow::{anyhow, Context, Result};
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::reader::NsReader;
use std::collections::HashMap;
use std::io::{Cursor, Read};

const NS_W: &[u8] = b"http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const NS_R: &[u8] =
    b"http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Architecture follows Anthropic's extract-text:
///   - Open zip
///   - Read styles.xml → map styleId → heading level
///   - Read document.xml.rels → map rId → hyperlink target
///   - Read footnotes.xml → footnote id → text
///   - Stream document.xml, emit markdown
pub fn extract(source: &Source) -> Result<String> {
    let cursor = Cursor::new(source.bytes());
    let mut zip = zip::ZipArchive::new(cursor)
        .context("failed to open docx as zip archive")?;

    let styles = read_text(&mut zip, "word/styles.xml").unwrap_or_default();
    let rels = read_text(&mut zip, "word/_rels/document.xml.rels").unwrap_or_default();
    let document = read_text(&mut zip, "word/document.xml")
        .ok_or_else(|| anyhow!("docx is missing word/document.xml"))?;

    let style_map = parse_styles(&styles);
    let rel_map = parse_rels(&rels);

    let footnotes = read_text(&mut zip, "word/footnotes.xml").unwrap_or_default();
    let footnote_map = parse_footnotes(&footnotes);

    let mut md = MarkdownBuilder::new();
    render_document(&document, &style_map, &rel_map, &mut md)?;

    // Append footnotes section
    if !footnote_map.is_empty() {
        md.blank_line();
        let mut ids: Vec<&String> = footnote_map.keys().collect();
        ids.sort_by(|a, b| {
            a.parse::<u32>()
                .unwrap_or(0)
                .cmp(&b.parse::<u32>().unwrap_or(0))
        });
        for id in ids {
            md.raw(&format!("[^{}]: {}\n", id, footnote_map[id].trim()));
        }
    }

    Ok(md.build())
}

fn read_text(zip: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str) -> Option<String> {
    let mut file = zip.by_name(name).ok()?;
    let mut s = String::new();
    file.read_to_string(&mut s).ok()?;
    Some(s)
}

/// Parse styles.xml → { styleId → heading_level }
/// For paragraph styles named "heading 1" through "heading 6".
fn parse_styles(xml: &str) -> HashMap<String, u8> {
    let mut map = HashMap::new();
    let mut reader = NsReader::from_str(xml);
    let mut current_id: Option<String> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_resolved_event_into(&mut buf) {
            Ok((ResolveResult::Bound(ns), Event::Start(e))) if ns.as_ref() == NS_W => {
                if e.local_name().as_ref() == b"style" {
                    current_id = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.local_name().as_ref() == b"styleId")
                        .and_then(|a| String::from_utf8(a.value.into_owned()).ok());
                } else if e.local_name().as_ref() == b"name" {
                    let val = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.local_name().as_ref() == b"val")
                        .and_then(|a| String::from_utf8(a.value.into_owned()).ok())
                        .unwrap_or_default();
                    if let Some(id) = &current_id {
                        if let Some(level) = parse_heading_level(&val) {
                            map.insert(id.clone(), level);
                        }
                    }
                }
            }
            Ok((_, Event::End(e))) if e.local_name().as_ref() == b"style" => {
                current_id = None;
            }
            Ok((_, Event::Eof)) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    map
}

fn parse_heading_level(name: &str) -> Option<u8> {
    let lower = name.to_lowercase();
    let prefix = "heading ";
    let rest = lower.strip_prefix(prefix)?;
    rest.parse::<u8>().ok().filter(|&n| n >= 1 && n <= 6)
}

/// Parse document.xml.rels → { rId → target_url }
fn parse_rels(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut reader = NsReader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_resolved_event_into(&mut buf) {
            Ok((_, Event::Empty(e))) | Ok((_, Event::Start(e)))
                if e.local_name().as_ref() == b"Relationship" =>
            {
                let mut id = None;
                let mut target = None;
                for a in e.attributes().flatten() {
                    match a.key.local_name().as_ref() {
                        b"Id" => id = String::from_utf8(a.value.into_owned()).ok(),
                        b"Target" => {
                            target = String::from_utf8(a.value.into_owned()).ok()
                        }
                        _ => {}
                    }
                }
                if let (Some(i), Some(t)) = (id, target) {
                    map.insert(i, t);
                }
            }
            Ok((_, Event::Eof)) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    map
}

fn parse_footnotes(_xml: &str) -> HashMap<String, String> {
    // TODO: walk <w:footnote w:id="N"> blocks, collect their text
    HashMap::new()
}

/// Walk document.xml and emit markdown. This is the heart of the extractor
/// and most of your iteration will happen here.
fn render_document(
    xml: &str,
    style_map: &HashMap<String, u8>,
    rel_map: &HashMap<String, String>,
    md: &mut MarkdownBuilder,
) -> Result<()> {
    let mut reader = NsReader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    // Per-paragraph state
    let mut in_paragraph = false;
    let mut paragraph_heading: Option<u8> = None;
    let mut paragraph_is_list = false;
    let mut paragraph_text = String::new();

    // Per-run state
    let mut in_run = false;
    let mut run_bold = false;
    let mut run_italic = false;

    // Hyperlink state (a hyperlink wraps one or more runs)
    let mut hyperlink_target: Option<String> = None;

    loop {
        match reader.read_resolved_event_into(&mut buf) {
            Ok((ResolveResult::Bound(ns), Event::Start(e))) if ns.as_ref() == NS_W => {
                match e.local_name().as_ref() {
                    b"p" => {
                        in_paragraph = true;
                        paragraph_heading = None;
                        paragraph_is_list = false;
                        paragraph_text.clear();
                    }
                    b"r" => {
                        in_run = true;
                        run_bold = false;
                        run_italic = false;
                    }
                    b"b" => run_bold = true,
                    b"i" => run_italic = true,
                    b"pStyle" => {
                        if let Some(val) = attr(&e, b"val") {
                            if let Some(level) = style_map.get(&val) {
                                paragraph_heading = Some(*level);
                            }
                        }
                    }
                    b"numPr" => paragraph_is_list = true,
                    b"hyperlink" => {
                        if let Some(rid) = attr_ns(&e, NS_R, b"id") {
                            hyperlink_target = rel_map.get(&rid).cloned();
                        }
                    }
                    _ => {}
                }
            }
            Ok((ResolveResult::Bound(ns), Event::Empty(e))) if ns.as_ref() == NS_W => {
                // <w:b/>, <w:i/>, <w:numPr.../> etc as empty elements
                match e.local_name().as_ref() {
                    b"b" => run_bold = true,
                    b"i" => run_italic = true,
                    b"pStyle" => {
                        if let Some(val) = attr(&e, b"val") {
                            if let Some(level) = style_map.get(&val) {
                                paragraph_heading = Some(*level);
                            }
                        }
                    }
                    b"numPr" => paragraph_is_list = true,
                    _ => {}
                }
            }
            Ok((ResolveResult::Bound(ns), Event::Text(t))) if ns.as_ref() == NS_W => {
                if in_run {
                    let s = t.unescape().unwrap_or_default().into_owned();
                    let wrapped = if let Some(target) = &hyperlink_target {
                        format!("[{}]({})", s, target)
                    } else if run_bold && run_italic {
                        format!("***{}***", s)
                    } else if run_bold {
                        format!("**{}**", s)
                    } else if run_italic {
                        format!("*{}*", s)
                    } else {
                        s
                    };
                    paragraph_text.push_str(&wrapped);
                }
            }
            Ok((_, Event::End(e))) => match e.local_name().as_ref() {
                b"r" => in_run = false,
                b"hyperlink" => hyperlink_target = None,
                b"p" => {
                    if in_paragraph {
                        if let Some(level) = paragraph_heading {
                            md.heading(level, &paragraph_text);
                        } else if paragraph_is_list {
                            md.blank_line();
                            md.raw(&format!("- {}\n", paragraph_text.trim()));
                        } else {
                            md.paragraph(&paragraph_text);
                        }
                    }
                    in_paragraph = false;
                }
                _ => {}
            },
            Ok((_, Event::Eof)) => break,
            Err(e) => return Err(anyhow!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

fn attr(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == name)
        .and_then(|a| String::from_utf8(a.value.into_owned()).ok())
}

fn attr_ns(
    e: &quick_xml::events::BytesStart,
    _ns: &[u8],
    name: &[u8],
) -> Option<String> {
    // Simplified — full implementation would resolve the attribute namespace.
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == name)
        .and_then(|a| String::from_utf8(a.value.into_owned()).ok())
}
