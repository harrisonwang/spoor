use crate::output::MarkdownBuilder;
use crate::source::Source;
use anyhow::{Context, Result, anyhow};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// DOCX → Markdown-like text (md output mode).
///
/// We deliberately match by local name and ignore namespace prefixes. This
/// keeps custom-prefix OOXML fixtures working without relying on version-
/// sensitive namespace-reader APIs.
pub fn extract(source: &Source) -> Result<String> {
    let cursor = Cursor::new(source.bytes());
    let mut zip = zip::ZipArchive::new(cursor).context("failed to open docx archive")?;

    let styles_xml = read_zip_text(&mut zip, "word/styles.xml").unwrap_or_default();
    let numbering_xml = read_zip_text(&mut zip, "word/numbering.xml").unwrap_or_default();
    let footnotes_xml = read_zip_text(&mut zip, "word/footnotes.xml").unwrap_or_default();
    let rels_xml = read_zip_text(&mut zip, "word/_rels/document.xml.rels").unwrap_or_default();
    let document_xml = read_zip_text(&mut zip, "word/document.xml")
        .ok_or_else(|| anyhow!("docx missing word/document.xml"))?;

    let style_map = parse_styles(&styles_xml);
    let numbering = parse_numbering(&numbering_xml);
    let footnotes = parse_footnotes(&footnotes_xml);
    let rel_map = parse_rels(&rels_xml);

    let mut md = MarkdownBuilder::new();
    render_document(
        &document_xml,
        &style_map,
        &numbering,
        &footnotes,
        &rel_map,
        &mut md,
    )?;
    Ok(md.build())
}

fn read_zip_text<R: std::io::Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
) -> Option<String> {
    let mut file = zip.by_name(name).ok()?;
    let mut s = String::new();
    file.read_to_string(&mut s).ok()?;
    Some(s)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StyleKind {
    Heading(u8),
    List(ListKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListKind {
    Bullet,
    Ordered,
}

impl ListKind {
    fn marker(self, number: usize) -> String {
        match self {
            ListKind::Bullet => "-".to_string(),
            ListKind::Ordered => format!("{number}."),
        }
    }
}

#[derive(Debug, Clone)]
struct ParagraphState {
    text: String,
    heading: Option<u8>,
    list: Option<ListInfo>,
}

impl ParagraphState {
    fn new() -> Self {
        Self {
            text: String::new(),
            heading: None,
            list: None,
        }
    }
}

#[derive(Debug, Clone)]
struct ListInfo {
    kind: Option<ListKind>,
    num_id: Option<String>,
    level: usize,
}

impl ListInfo {
    fn new(kind: Option<ListKind>) -> Self {
        Self {
            kind,
            num_id: None,
            level: 0,
        }
    }
}

#[derive(Debug, Default)]
struct TableState {
    rows: Vec<Vec<String>>,
    current_row: Option<Vec<String>>,
    current_cell: Option<String>,
}

/// styleId -> heading/list metadata.
fn parse_styles(xml: &str) -> HashMap<String, StyleKind> {
    let mut map = HashMap::new();
    if xml.is_empty() {
        return map;
    }
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut current_id: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"style" => current_id = attr(&e, b"styleId"),
                b"name" => {
                    if let (Some(id), Some(name)) = (&current_id, attr(&e, b"val")) {
                        if let Some(kind) = parse_style_kind(&name) {
                            map.insert(id.clone(), kind);
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(e)) if e.local_name().as_ref() == b"name" => {
                if let (Some(id), Some(name)) = (&current_id, attr(&e, b"val")) {
                    if let Some(kind) = parse_style_kind(&name) {
                        map.insert(id.clone(), kind);
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"style" => {
                current_id = None;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    map
}

fn parse_style_kind(name: &str) -> Option<StyleKind> {
    if let Some(level) = parse_heading_level(name) {
        return Some(StyleKind::Heading(level));
    }
    let normalized = name.to_ascii_lowercase().replace([' ', '-'], "");
    if normalized.contains("listbullet") || normalized.contains("bullet") {
        return Some(StyleKind::List(ListKind::Bullet));
    }
    if normalized.contains("listnumber") || normalized.contains("number") {
        return Some(StyleKind::List(ListKind::Ordered));
    }
    None
}

fn parse_heading_level(name: &str) -> Option<u8> {
    let lower = name.to_lowercase();
    let rest = lower.strip_prefix("heading ")?;
    rest.parse::<u8>().ok().filter(|&n| (1..=6).contains(&n))
}

/// numId -> ilvl -> list kind.
fn parse_numbering(xml: &str) -> HashMap<String, HashMap<usize, ListKind>> {
    let mut resolved = HashMap::new();
    if xml.is_empty() {
        return resolved;
    }

    let mut abstract_levels: HashMap<String, HashMap<usize, ListKind>> = HashMap::new();
    let mut num_to_abstract: HashMap<String, String> = HashMap::new();
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut current_abstract: Option<String> = None;
    let mut current_level: Option<usize> = None;
    let mut current_num: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => match e.local_name().as_ref() {
                b"abstractNum" => current_abstract = attr(&e, b"abstractNumId"),
                b"lvl" => current_level = attr(&e, b"ilvl").and_then(|v| v.parse().ok()),
                b"numFmt" => {
                    if let (Some(abs), Some(level), Some(val)) =
                        (&current_abstract, current_level, attr(&e, b"val"))
                    {
                        abstract_levels
                            .entry(abs.clone())
                            .or_default()
                            .insert(level, parse_num_fmt(&val));
                    }
                }
                b"num" => current_num = attr(&e, b"numId"),
                b"abstractNumId" => {
                    if let (Some(num), Some(abs)) = (&current_num, attr(&e, b"val")) {
                        num_to_abstract.insert(num.clone(), abs);
                    }
                }
                _ => {}
            },
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"abstractNum" => current_abstract = None,
                b"lvl" => current_level = None,
                b"num" => current_num = None,
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    for (num_id, abstract_id) in num_to_abstract {
        if let Some(levels) = abstract_levels.get(&abstract_id) {
            resolved.insert(num_id, levels.clone());
        }
    }
    resolved
}

fn parse_num_fmt(value: &str) -> ListKind {
    match value.to_ascii_lowercase().as_str() {
        "decimal" | "lowerletter" | "upperletter" | "lowerroman" | "upperroman" => {
            ListKind::Ordered
        }
        _ => ListKind::Bullet,
    }
}

fn parse_footnotes(xml: &str) -> HashMap<String, String> {
    let mut notes = HashMap::new();
    if xml.is_empty() {
        return notes;
    }

    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_text = String::new();
    let mut in_deleted = 0usize;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"footnote" => {
                    current_id = attr(&e, b"id");
                    current_text.clear();
                }
                b"del" => in_deleted += 1,
                _ => {}
            },
            Ok(Event::Text(t)) => {
                if current_id.is_some() && in_deleted == 0 {
                    let s = t.unescape().map(|c| c.into_owned()).unwrap_or_default();
                    current_text.push_str(&s);
                }
            }
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"footnote" => {
                    if let Some(id) = current_id.take() {
                        let text = current_text.trim();
                        if !text.is_empty() {
                            notes.insert(id, text.to_string());
                        }
                    }
                }
                b"del" => in_deleted = in_deleted.saturating_sub(1),
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    notes
}

/// rId → URL.
fn parse_rels(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if xml.is_empty() {
        return map;
    }
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e))
                if e.local_name().as_ref() == b"Relationship" =>
            {
                let id = attr(&e, b"Id");
                let target = attr(&e, b"Target");
                if let (Some(id), Some(target)) = (id, target) {
                    map.insert(id, target);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    map
}

fn render_document(
    xml: &str,
    style_map: &HashMap<String, StyleKind>,
    numbering: &HashMap<String, HashMap<usize, ListKind>>,
    footnotes: &HashMap<String, String>,
    rel_map: &HashMap<String, String>,
    md: &mut MarkdownBuilder,
) -> Result<()> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut paragraph: Option<ParagraphState> = None;
    let mut table: Option<TableState> = None;

    let mut in_run = false;
    let mut run_bold = false;
    let mut run_italic = false;
    let mut in_num_pr = false;
    let mut in_deleted = 0usize;

    let mut hyperlink_target: Option<String> = None;
    let mut used_footnotes: Vec<String> = Vec::new();
    let mut list_counters: HashMap<(String, usize), usize> = HashMap::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                b"tbl" => {
                    table = Some(TableState::default());
                }
                b"tr" => {
                    if let Some(table) = &mut table {
                        table.current_row = Some(Vec::new());
                    }
                }
                b"tc" => {
                    if let Some(table) = &mut table {
                        table.current_cell = Some(String::new());
                    }
                }
                b"p" => {
                    paragraph = Some(ParagraphState::new());
                }
                b"r" => {
                    in_run = true;
                    run_bold = false;
                    run_italic = false;
                }
                b"hyperlink" => {
                    if let Some(rid) = attr(&e, b"id") {
                        hyperlink_target = rel_map.get(&rid).cloned();
                    }
                }
                b"b" => run_bold = true,
                b"i" => run_italic = true,
                b"pStyle" => apply_pstyle(&e, &mut paragraph, style_map),
                b"numPr" => {
                    in_num_pr = true;
                    if let Some(p) = &mut paragraph {
                        p.list.get_or_insert_with(|| ListInfo::new(None));
                    }
                }
                b"ilvl" if in_num_pr => apply_list_level(&e, &mut paragraph),
                b"numId" if in_num_pr => apply_list_num_id(&e, &mut paragraph, numbering),
                b"footnoteReference" => {
                    push_footnote_reference(&e, &mut paragraph, &mut used_footnotes);
                }
                b"del" => in_deleted += 1,
                _ => {}
            },
            Ok(Event::Empty(e)) => match e.local_name().as_ref() {
                b"b" => run_bold = true,
                b"i" => run_italic = true,
                b"pStyle" => apply_pstyle(&e, &mut paragraph, style_map),
                b"numPr" => {
                    if let Some(p) = &mut paragraph {
                        p.list.get_or_insert_with(|| ListInfo::new(None));
                    }
                }
                b"ilvl" if in_num_pr => apply_list_level(&e, &mut paragraph),
                b"numId" if in_num_pr => apply_list_num_id(&e, &mut paragraph, numbering),
                b"footnoteReference" => {
                    push_footnote_reference(&e, &mut paragraph, &mut used_footnotes);
                }
                b"tab" => push_text(&mut paragraph, "\t", false, false, &hyperlink_target),
                b"br" => push_text(&mut paragraph, "\n", false, false, &hyperlink_target),
                _ => {}
            },
            Ok(Event::Text(t)) => {
                if in_run && in_deleted == 0 {
                    let s = t.unescape().map(|c| c.into_owned()).unwrap_or_default();
                    push_text(&mut paragraph, &s, run_bold, run_italic, &hyperlink_target);
                }
            }
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"r" => in_run = false,
                b"hyperlink" => hyperlink_target = None,
                b"numPr" => in_num_pr = false,
                b"del" => in_deleted = in_deleted.saturating_sub(1),
                b"p" => {
                    if let Some(p) = paragraph.take() {
                        if let Some(table) = &mut table {
                            append_cell_paragraph(table, &p.text);
                        } else {
                            render_paragraph(md, p, &mut list_counters);
                        }
                    }
                }
                b"tc" => {
                    if let Some(table) = &mut table {
                        if let (Some(row), Some(cell)) =
                            (&mut table.current_row, table.current_cell.take())
                        {
                            row.push(sanitize_table_cell(&cell));
                        }
                    }
                }
                b"tr" => {
                    if let Some(table) = &mut table {
                        if let Some(row) = table.current_row.take() {
                            if row.iter().any(|cell| !cell.trim().is_empty()) {
                                table.rows.push(row);
                            }
                        }
                    }
                }
                b"tbl" => {
                    if let Some(table) = table.take() {
                        render_table(md, table.rows);
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    render_footnotes(md, &used_footnotes, footnotes);
    Ok(())
}

fn apply_pstyle(
    e: &BytesStart,
    paragraph: &mut Option<ParagraphState>,
    style_map: &HashMap<String, StyleKind>,
) {
    let Some(p) = paragraph else {
        return;
    };
    let Some(val) = attr(e, b"val") else {
        return;
    };
    match style_map
        .get(&val)
        .copied()
        .or_else(|| parse_style_kind(&val))
    {
        Some(StyleKind::Heading(level)) => p.heading = Some(level),
        Some(StyleKind::List(kind)) => {
            let list = p.list.get_or_insert_with(|| ListInfo::new(Some(kind)));
            list.kind = Some(kind);
        }
        None => {}
    }
}

fn apply_list_level(e: &BytesStart, paragraph: &mut Option<ParagraphState>) {
    if let (Some(p), Some(level)) = (
        paragraph,
        attr(e, b"val").and_then(|value| value.parse().ok()),
    ) {
        let list = p.list.get_or_insert_with(|| ListInfo::new(None));
        list.level = level;
    }
}

fn apply_list_num_id(
    e: &BytesStart,
    paragraph: &mut Option<ParagraphState>,
    numbering: &HashMap<String, HashMap<usize, ListKind>>,
) {
    let Some(p) = paragraph else {
        return;
    };
    let Some(num_id) = attr(e, b"val") else {
        return;
    };
    let list = p.list.get_or_insert_with(|| ListInfo::new(None));
    list.num_id = Some(num_id.clone());
    if list.kind.is_none() {
        list.kind = numbering
            .get(&num_id)
            .and_then(|levels| levels.get(&list.level))
            .copied();
    }
}

fn push_footnote_reference(
    e: &BytesStart,
    paragraph: &mut Option<ParagraphState>,
    used_footnotes: &mut Vec<String>,
) {
    let Some(id) = attr(e, b"id") else {
        return;
    };
    if let Some(p) = paragraph {
        p.text.push_str(&format!("[^{id}]"));
        if !used_footnotes.iter().any(|seen| seen == &id) {
            used_footnotes.push(id);
        }
    }
}

fn push_text(
    paragraph: &mut Option<ParagraphState>,
    text: &str,
    bold: bool,
    italic: bool,
    hyperlink_target: &Option<String>,
) {
    if let Some(p) = paragraph {
        p.text
            .push_str(&wrap_run_text(text, bold, italic, hyperlink_target));
    }
}

fn wrap_run_text(
    text: &str,
    bold: bool,
    italic: bool,
    hyperlink_target: &Option<String>,
) -> String {
    if hyperlink_target.is_none() && !bold && !italic {
        return text.to_string();
    }

    // Whitespace-only runs (e.g. bold space, italic line break, linked tab) have
    // no slice-stable "middle" for markdown wrapping; for md output we keep the
    // raw characters and skip ** / * / []() — see docs/ENGINEERING_DECISIONS.md.
    if text.trim().is_empty() {
        return text.to_string();
    }

    let leading_len = text.len() - text.trim_start().len();
    let trailing_len = text.len() - text.trim_end().len();
    let trailing_start = text.len().saturating_sub(trailing_len);
    let leading = &text[..leading_len];
    let trailing = &text[trailing_start..];
    let middle = &text[leading_len..trailing_start];

    if middle.is_empty() {
        return text.to_string();
    }

    let wrapped = if let Some(target) = hyperlink_target {
        format!("[{middle}]({target})")
    } else if bold && italic {
        format!("***{middle}***")
    } else if bold {
        format!("**{middle}**")
    } else if italic {
        format!("*{middle}*")
    } else {
        middle.to_string()
    };
    format!("{leading}{wrapped}{trailing}")
}

fn render_paragraph(
    md: &mut MarkdownBuilder,
    paragraph: ParagraphState,
    list_counters: &mut HashMap<(String, usize), usize>,
) {
    if let Some(level) = paragraph.heading {
        md.heading(level, &paragraph.text);
        return;
    }

    if let Some(list) = paragraph.list {
        let trimmed = paragraph.text.trim();
        if trimmed.is_empty() {
            return;
        }
        let kind = list.kind.unwrap_or(ListKind::Bullet);
        let marker = match kind {
            ListKind::Bullet => kind.marker(0),
            ListKind::Ordered => {
                let key = (
                    list.num_id.unwrap_or_else(|| "__style_number".to_string()),
                    list.level,
                );
                let count = list_counters.entry(key).or_insert(0);
                *count += 1;
                kind.marker(*count)
            }
        };
        let indent = "  ".repeat(list.level);
        md.blank_line();
        md.raw(&format!("{indent}{marker} {trimmed}\n"));
        return;
    }

    md.paragraph(&paragraph.text);
}

fn append_cell_paragraph(table: &mut TableState, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    if let Some(cell) = &mut table.current_cell {
        if !cell.is_empty() {
            cell.push_str("<br>");
        }
        cell.push_str(trimmed);
    }
}

fn render_table(md: &mut MarkdownBuilder, mut rows: Vec<Vec<String>>) {
    if rows.is_empty() {
        return;
    }
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    if cols == 0 {
        return;
    }
    for row in &mut rows {
        while row.len() < cols {
            row.push(String::new());
        }
    }

    md.blank_line();
    md.raw(&format!("| {} |\n", rows[0].join(" | ")));
    md.raw(&format!("| {} |\n", vec!["---"; cols].join(" | ")));
    for row in rows.iter().skip(1) {
        md.raw(&format!("| {} |\n", row.join(" | ")));
    }
}

fn sanitize_table_cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\n', '\r', '\t'], " ")
}

fn render_footnotes(
    md: &mut MarkdownBuilder,
    used_footnotes: &[String],
    footnotes: &HashMap<String, String>,
) {
    if used_footnotes.is_empty() {
        return;
    }
    md.blank_line();
    for id in used_footnotes {
        if let Some(text) = footnotes.get(id) {
            md.raw(&format!("[^{id}]: {}\n", text.trim()));
        }
    }
}

fn attr(e: &BytesStart, local_name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.local_name().as_ref() == local_name)
        .and_then(|a| String::from_utf8(a.value.into_owned()).ok())
}

#[cfg(test)]
mod wrap_run_text_tests {
    use super::wrap_run_text;

    #[test]
    fn whitespace_only_skips_markdown_wrapping() {
        let url = Some("https://example.com".to_string());
        assert_eq!(wrap_run_text(" ", true, false, &None), " ");
        assert_eq!(wrap_run_text(" ", false, true, &None), " ");
        assert_eq!(wrap_run_text("\n", false, true, &None), "\n");
        assert_eq!(wrap_run_text("  \n\t", true, true, &None), "  \n\t");
        assert_eq!(wrap_run_text(" ", false, false, &url), " ");
    }

    #[test]
    fn visible_text_still_gets_markdown_wrapping() {
        assert_eq!(wrap_run_text("hi", true, false, &None), "**hi**");
        assert_eq!(
            wrap_run_text("x", false, false, &Some("https://a".to_string())),
            "[x](https://a)"
        );
    }
}
