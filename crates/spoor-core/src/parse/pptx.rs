use crate::limits;
use crate::output::MarkdownBuilder;
use crate::parse::ExtractedMarkdown;
use crate::parse::xml::attr;
use crate::result::{SpoorWarning, WarningCode};
use crate::source::Source;
use anyhow::{Result, anyhow};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::path::{Component, Path};

pub fn extract(source: &Source<'_>, max_parse_bytes: usize) -> Result<ExtractedMarkdown> {
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
    let mut warnings = Vec::new();
    let mut image_number: usize = 0;
    for (n, name) in &slides {
        md.heading(2, &format!("Slide {n}"));
        let xml = limits::read_zip_text(&mut zip, name, max_parse_bytes)?;
        let rels = slide_rel_targets(&mut zip, name, max_parse_bytes)?;
        let slide_no = *n as usize;
        let mut emitted = SlideImageEmission::default();
        render_slide(
            &xml,
            slide_no,
            &rels,
            &mut image_number,
            &mut emitted,
            &mut md,
        )?;
        warnings.extend(feature_warnings(
            slide_no,
            scan_slide_features(&xml)?,
            emitted,
        ));
        if let Some(notes_name) = notes_slide_for(&mut zip, name, max_parse_bytes)? {
            let notes_xml = limits::read_zip_text(&mut zip, &notes_name, max_parse_bytes)?;
            render_notes(&notes_xml, &mut md)?;
        }
    }
    Ok(ExtractedMarkdown::with_warnings(md.build()?, warnings))
}

/// Per-slide tally of `<a:blip>` references the renderer saw and what it could
/// resolve to a safe `ppt/media/*` part name. Feeds the warning so the agent
/// knows whether every visual was marked, only some were, or none.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SlideImageEmission {
    total_blips: usize,
    emitted_handles: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SlideFeatures {
    merged_table: bool,
    embedded_visuals: bool,
}

fn scan_slide_features(xml: &str) -> Result<SlideFeatures> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut features = SlideFeatures::default();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                features.merged_table |= [
                    b"gridSpan".as_slice(),
                    b"rowSpan".as_slice(),
                    b"hMerge".as_slice(),
                    b"vMerge".as_slice(),
                ]
                .iter()
                .any(|name| attr(&e, name).is_some());
                features.embedded_visuals |= matches!(
                    e.local_name().as_ref(),
                    b"pic" | b"blip" | b"chart" | b"oleObj"
                );
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(anyhow!("XML parse error: {error}")),
            _ => {}
        }
        buf.clear();
    }

    Ok(features)
}

fn feature_warnings(
    slide: usize,
    features: SlideFeatures,
    emission: SlideImageEmission,
) -> Vec<SpoorWarning> {
    let mut warnings = Vec::new();
    if features.merged_table {
        warnings.push(SpoorWarning::at_slide(
            WarningCode::MergedTableStructureNotPreserved,
            format!(
                "第 {slide} 张幻灯片包含合并单元格；当前 Markdown 表格不保留 rowspan/colspan，Agent 不应把空白或重复单元格解释为原始结构。"
            ),
            slide,
        ));
    }
    if features.embedded_visuals {
        // Mirror pdf.rs's three-branch wording: did every visual get a handle,
        // none, or only some? Agents key off the wording to decide whether the
        // slide is fully recoverable via `--extract` or still needs external
        // VLM rendering for the un-handled charts/OLE objects.
        let message = if emission.total_blips == 0 {
            format!(
                "第 {slide} 张幻灯片包含图表或嵌入对象（无栅格图片）；spoor 当前仅提取文本，Agent 应把该页视为不完整并按需调用外部视觉解析。"
            )
        } else if emission.emitted_handles == emission.total_blips {
            format!(
                "第 {slide} 张幻灯片含 {n} 张内嵌图片；已用 spoor://pptx/part/ 标注，Agent 可用 --extract 取出交给视觉模型。",
                n = emission.emitted_handles,
            )
        } else if emission.emitted_handles == 0 {
            format!(
                "第 {slide} 张幻灯片含 {n} 张内嵌图片，但 spoor 未能解析其引用；请视该页为不完整并按需调用外部视觉解析。",
                n = emission.total_blips,
            )
        } else {
            format!(
                "第 {slide} 张幻灯片含 {total} 张内嵌图片；其中 {ok} 张已用 spoor://pptx/part/ 标注可 --extract 取出，其余未解析。",
                total = emission.total_blips,
                ok = emission.emitted_handles,
            )
        };
        warnings.push(SpoorWarning::at_slide(
            WarningCode::EmbeddedVisualsOmitted,
            message,
            slide,
        ));
    }
    warnings
}

fn slide_number(name: &str) -> Option<u32> {
    name.strip_prefix("ppt/slides/slide")?
        .strip_suffix(".xml")?
        .parse::<u32>()
        .ok()
}

fn render_slide(
    xml: &str,
    slide_number: usize,
    rels: &HashMap<String, String>,
    image_number: &mut usize,
    emission: &mut SlideImageEmission,
    md: &mut MarkdownBuilder,
) -> Result<()> {
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
                b"blip" => emit_blip_placeholder(
                    &e,
                    slide_number,
                    rels,
                    image_number,
                    emission,
                    if in_table_cell {
                        &mut current_cell
                    } else {
                        &mut paragraph
                    },
                ),
                _ => {}
            },
            Ok(Event::Empty(e)) if e.local_name().as_ref() == b"blip" => emit_blip_placeholder(
                &e,
                slide_number,
                rels,
                image_number,
                emission,
                if in_table_cell {
                    &mut current_cell
                } else {
                    &mut paragraph
                },
            ),
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

/// Emit a `![PPTX image N (slide S)](spoor://pptx/part/ppt/media/...)`
/// placeholder for an `<a:blip>` element, resolving its `r:embed` rId through
/// the slide's rels. A blip without a matching image target stays uncounted on
/// `emitted_handles` so the warning surfaces the gap. The caller picks the
/// active text sink (`paragraph` or `current_cell`) so this helper stays
/// independent of the renderer's in_table_cell branching.
fn emit_blip_placeholder(
    e: &quick_xml::events::BytesStart<'_>,
    slide_number: usize,
    rels: &HashMap<String, String>,
    image_number: &mut usize,
    emission: &mut SlideImageEmission,
    sink: &mut String,
) {
    emission.total_blips += 1;
    let Some(rid) = attr(e, b"embed") else {
        return;
    };
    let Some(target) = rels.get(&rid) else {
        return;
    };
    *image_number += 1;
    emission.emitted_handles += 1;
    let placeholder =
        format!("![PPTX image {image_number} (slide {slide_number})](spoor://pptx/part/{target})");
    sink.push_str(&placeholder);
}

/// Build a `rId → ppt/media/imageN.ext` map for `slide_name`'s rels file.
/// Targets are normalized through `normalize_zip_path` so `../media/foo.png`
/// (the form OOXML writes) becomes `ppt/media/foo.png`. Non-image rels (notes,
/// hyperlinks, …) are filtered by relationship type.
fn slide_rel_targets<R: std::io::Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    slide_name: &str,
    max_parse_bytes: usize,
) -> Result<HashMap<String, String>> {
    let Some(file_name) = Path::new(slide_name).file_name().and_then(|s| s.to_str()) else {
        return Ok(HashMap::new());
    };
    let rels_name = format!("ppt/slides/_rels/{file_name}.rels");
    let Some(rels_xml) = limits::read_zip_text_optional(zip, &rels_name, max_parse_bytes)? else {
        return Ok(HashMap::new());
    };
    let base = Path::new(slide_name)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    Ok(parse_image_rel_targets(&rels_xml, base))
}

fn parse_image_rel_targets(xml: &str, base: &Path) -> HashMap<String, String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut map = HashMap::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e))
                if e.local_name().as_ref() == b"Relationship" =>
            {
                let rel_type = attr(&e, b"Type").unwrap_or_default();
                if !rel_type.ends_with("/image") {
                    buf.clear();
                    continue;
                }
                let Some(id) = attr(&e, b"Id") else {
                    buf.clear();
                    continue;
                };
                let Some(target) = attr(&e, b"Target") else {
                    buf.clear();
                    continue;
                };
                let normalized = normalize_zip_path(base.join(target));
                // Emit a handle only for a path that will also pass the
                // extract-time OPC validator. This stops a crafted media
                // filename (markdown link syntax, spaces) from breaking out of /
                // injecting into the `](spoor://pptx/part/...)` placeholder link,
                // and guarantees every emitted handle round-trips through
                // `--extract`. A dropped rel leaves the blip unresolved, which
                // the per-slide "partial / unresolved" warning already surfaces.
                if crate::engine::safe_opc_media_subpath("ppt", &normalized) {
                    map.insert(id, normalized);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    map
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

#[cfg(test)]
mod feature_warning_tests {
    use super::{
        SlideFeatures, SlideImageEmission, feature_warnings, parse_image_rel_targets,
        scan_slide_features,
    };
    use crate::result::WarningLocation;
    use std::path::Path;

    #[test]
    fn detects_merged_cells_and_visuals() {
        let features = scan_slide_features(
            r#"<p:sld xmlns:p="urn:p" xmlns:a="urn:a"><a:tc gridSpan="2"/><p:pic/></p:sld>"#,
        )
        .unwrap();

        assert_eq!(
            features,
            SlideFeatures {
                merged_table: true,
                embedded_visuals: true,
            }
        );
        // No blips actually rendered: emission stays zero, surfacing the
        // "chart/OLE-only" wording rather than the false "spoor 未解析" one.
        let warnings = feature_warnings(3, features, SlideImageEmission::default());
        assert_eq!(warnings.len(), 2);
        assert_eq!(
            warnings[0].location,
            Some(WarningLocation::Slide { number: 3 })
        );
        assert!(
            warnings[1].message.contains("无栅格图片"),
            "expected chart/OLE wording, got {:?}",
            warnings[1].message
        );
    }

    #[test]
    fn fully_marked_visuals_get_extract_wording() {
        let features = SlideFeatures {
            merged_table: false,
            embedded_visuals: true,
        };
        let emission = SlideImageEmission {
            total_blips: 2,
            emitted_handles: 2,
        };
        let warnings = feature_warnings(1, features, emission);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("spoor://pptx/part/"));
        assert!(warnings[0].message.contains("--extract"));
    }

    #[test]
    fn partially_marked_visuals_surface_gap() {
        let features = SlideFeatures {
            merged_table: false,
            embedded_visuals: true,
        };
        let emission = SlideImageEmission {
            total_blips: 3,
            emitted_handles: 1,
        };
        let warnings = feature_warnings(2, features, emission);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("其中 1"));
        assert!(warnings[0].message.contains("其余未解析"));
    }

    #[test]
    fn parses_image_rels_and_normalizes_relative_paths() {
        // Real PPTX rels: image targets are written as `../media/imageN.png`
        // relative to `ppt/slides/`. `parse_image_rel_targets` must resolve
        // them to a canonical `ppt/media/imageN.png` ZIP entry, drop
        // non-image rels, and key on rId.
        let xml = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
            <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png"/>
            <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide1.xml"/>
            <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image2.jpeg"/>
        </Relationships>"#;
        let map = parse_image_rel_targets(xml, Path::new("ppt/slides"));
        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("rId1").map(String::as_str),
            Some("ppt/media/image1.png")
        );
        assert_eq!(
            map.get("rId3").map(String::as_str),
            Some("ppt/media/image2.jpeg")
        );
        assert!(!map.contains_key("rId2"), "non-image rel must be skipped");
    }

    #[test]
    fn unsafe_media_filenames_are_dropped_not_emitted_as_handles() {
        // A crafted media filename with markdown link syntax or spaces must not
        // become a placeholder target — otherwise it breaks out of / injects
        // into the `](spoor://pptx/part/...)` link in agent-facing output. Only
        // charset-safe `ppt/media/<name>` paths (which also pass --extract) emit.
        let xml = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
            <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/evil) [x](http://e).png"/>
            <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/a b.png"/>
            <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png"/>
        </Relationships>"#;
        let map = parse_image_rel_targets(xml, Path::new("ppt/slides"));
        assert!(
            !map.contains_key("rId1"),
            "markdown-injection filename must be dropped"
        );
        assert!(
            !map.contains_key("rId2"),
            "filename with a space must be dropped"
        );
        assert_eq!(
            map.get("rId3").map(String::as_str),
            Some("ppt/media/image1.png"),
            "the safe filename still emits a handle"
        );
    }
}
