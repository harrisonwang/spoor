use crate::output::MarkdownBuilder;
use crate::output::decode_text;
use crate::source::Source;
use anyhow::Result;
use scraper::node::Node;
use scraper::{ElementRef, Html, Selector};

pub fn extract(source: &Source<'_>, max_parse_bytes: usize) -> Result<String> {
    let html = decode_text(source.bytes());
    let doc = Html::parse_document(&html);

    let article_sel = Selector::parse("article").unwrap();
    let main_sel = Selector::parse("main").unwrap();
    let body_sel = Selector::parse("body").unwrap();

    let root = doc
        .select(&article_sel)
        .next()
        .or_else(|| doc.select(&main_sel).next())
        .or_else(|| doc.select(&body_sel).next());

    let mut md = MarkdownBuilder::with_max_bytes(max_parse_bytes);
    if let Some(root) = root {
        render_children(root, &mut md);
    }
    md.build()
}

fn render_children(element: ElementRef<'_>, md: &mut MarkdownBuilder) {
    for child in element.children() {
        let Some(child_el) = ElementRef::wrap(child) else {
            continue;
        };
        render_block(child_el, md);
    }
}

fn render_block(element: ElementRef<'_>, md: &mut MarkdownBuilder) {
    let name = element.value().name();
    if should_skip_block(name) {
        return;
    }

    match name {
        "h1" => md.heading(1, &inline_text(element)),
        "h2" => md.heading(2, &inline_text(element)),
        "h3" => md.heading(3, &inline_text(element)),
        "h4" => md.heading(4, &inline_text(element)),
        "h5" => md.heading(5, &inline_text(element)),
        "h6" => md.heading(6, &inline_text(element)),
        "p" => md.paragraph(&inline_text(element)),
        "ul" => render_list(element, md, false),
        "ol" => render_list(element, md, true),
        "table" => render_table(element, md),
        "blockquote" => render_blockquote(element, md),
        "pre" => render_preformatted(element, md),
        "article" | "main" | "section" | "div" | "body" => render_children(element, md),
        _ => {
            if has_block_children(element) {
                render_children(element, md);
            } else {
                md.paragraph(&inline_text(element));
            }
        }
    }
}

fn should_skip_block(name: &str) -> bool {
    matches!(
        name,
        "script" | "style" | "noscript" | "template" | "header" | "nav" | "footer" | "aside"
    )
}

fn has_block_children(element: ElementRef<'_>) -> bool {
    element.children().any(|child| {
        ElementRef::wrap(child)
            .map(|el| {
                matches!(
                    el.value().name(),
                    "h1" | "h2"
                        | "h3"
                        | "h4"
                        | "h5"
                        | "h6"
                        | "p"
                        | "ul"
                        | "ol"
                        | "table"
                        | "blockquote"
                        | "pre"
                        | "article"
                        | "main"
                        | "section"
                        | "div"
                )
            })
            .unwrap_or(false)
    })
}

fn render_blockquote(element: ElementRef<'_>, md: &mut MarkdownBuilder) {
    let text = inline_text(element);
    if text.is_empty() {
        return;
    }
    md.blank_line();
    for line in text.lines() {
        md.raw(&format!("> {}\n", line.trim()));
    }
}

fn render_preformatted(element: ElementRef<'_>, md: &mut MarkdownBuilder) {
    let text = element.text().collect::<String>();
    let text = text.trim_matches('\n');
    if text.trim().is_empty() {
        return;
    }
    md.blank_line();
    md.raw("```\n");
    md.raw(text);
    if !text.ends_with('\n') {
        md.raw("\n");
    }
    md.raw("```\n");
}

fn render_list(element: ElementRef<'_>, md: &mut MarkdownBuilder, ordered: bool) {
    let mut index = 1usize;
    for child in element.children() {
        let Some(li) = ElementRef::wrap(child) else {
            continue;
        };
        if li.value().name() != "li" {
            continue;
        }

        let text = inline_text(li);
        if text.is_empty() {
            continue;
        }
        md.blank_line();
        if ordered {
            md.raw(&format!("{index}. {text}\n"));
            index += 1;
        } else {
            md.raw(&format!("- {text}\n"));
        }
    }
}

fn render_table(element: ElementRef<'_>, md: &mut MarkdownBuilder) {
    let row_sel = Selector::parse("tr").unwrap();
    let cell_sel = Selector::parse("th, td").unwrap();
    let rows: Vec<Vec<String>> = element
        .select(&row_sel)
        .map(|row| row.select(&cell_sel).map(inline_text).collect::<Vec<_>>())
        .filter(|row| !row.is_empty())
        .collect();

    md.table(&rows);
}

fn inline_text(element: ElementRef<'_>) -> String {
    normalize_inline(&inline_children(element))
}

fn inline_children(element: ElementRef<'_>) -> String {
    let mut out = String::new();
    for child in element.children() {
        match child.value() {
            Node::Text(text) => out.push_str(text),
            Node::Element(el) => {
                let Some(child_el) = ElementRef::wrap(child) else {
                    continue;
                };
                match el.name() {
                    "script" | "style" | "noscript" | "template" => {}
                    "a" => {
                        let text = inline_text(child_el);
                        if let Some(href) = el.attr("href") {
                            if text.is_empty() {
                                out.push_str(href);
                            } else {
                                out.push_str(&format!("[{text}]({href})"));
                            }
                        } else {
                            out.push_str(&text);
                        }
                    }
                    "strong" | "b" => {
                        let text = inline_text(child_el);
                        if !text.is_empty() {
                            out.push_str(&format!("**{text}**"));
                        }
                    }
                    "em" | "i" => {
                        let text = inline_text(child_el);
                        if !text.is_empty() {
                            out.push_str(&format!("*{text}*"));
                        }
                    }
                    "code" => {
                        let text = inline_text(child_el);
                        if !text.is_empty() {
                            out.push_str(&format!("`{text}`"));
                        }
                    }
                    "img" => {
                        if let Some(alt) = el.attr("alt").filter(|alt| !alt.trim().is_empty()) {
                            out.push_str(&format!("[图片：{}]", alt.trim()));
                        }
                    }
                    "br" => out.push('\n'),
                    "ul" | "ol" | "table" => {}
                    _ => out.push_str(&inline_children(child_el)),
                }
            }
            _ => {}
        }
    }
    out
}

fn normalize_inline(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
