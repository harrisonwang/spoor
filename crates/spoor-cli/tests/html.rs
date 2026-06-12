//! HTML integration tests.

mod common;
use common::extract_fixture;
use insta::assert_snapshot;
use spoor_core::Format;

#[test]
fn semantic_article() {
    // <article> is the canonical readability target. We should:
    //   - keep h1, h2 → '#', '##'
    //   - keep <ul> → '- '
    //   - convert <a href> → markdown links
    //   - drop <header>, <nav>, <footer>
    //
    // extract-text's HTML mode is far weaker — it strips tags entirely
    // (no headings, no links, no list markers). We CHOOSE to emit
    // proper markdown, matching how docx is handled.
    let out = extract_fixture("html/01_article.html", Format::Html);
    assert_snapshot!(out);
}

#[test]
fn cluttered_page_main_content_isolated() {
    // Ad banners, nav, related-articles sidebars should be stripped.
    // The <main><article> content should survive.
    let out = extract_fixture("html/02_cluttered.html", Format::Html);
    assert_snapshot!(out);
}

#[test]
fn html_table_to_gfm() {
    // <table>/<thead>/<tbody>/<tr>/<th>/<td> → GFM table.
    let out = extract_fixture("html/03_table.html", Format::Html);
    assert_snapshot!(out);
}

#[test]
fn gbk_html_without_meta_charset() {
    // Many Chinese sites still serve GBK without declaring it.
    // Our chardetng-based decoder handles this.
    let out = extract_fixture("html/04_gbk_no_meta.html", Format::Html);
    assert_snapshot!(out);
}

#[test]
fn script_and_style_tags_stripped() {
    // <script> and <style> contents must NEVER appear in output.
    let out = extract_fixture("html/05_scripts_styles.html", Format::Html);
    assert!(!out.contains("alert"), "script content leaked: {}", out);
    assert!(!out.contains("color: red"), "style content leaked: {}", out);
    assert!(out.contains("Real content"));
}

#[test]
fn links_preserve_href() {
    let out = extract_fixture("html/06_links.html", Format::Html);
    assert_snapshot!(out);
}

#[test]
fn semantic_blocks_preserve_llm_relevant_content() {
    let out = extract_fixture("html/07_semantic_blocks.html", Format::Html);

    assert!(out.contains("> 文档结构比视觉样式更重要。"));
    assert!(out.contains("`spoor 报告.docx`"));
    assert!(out.contains("```\n风险 = \"需要复核\""));
    assert!(out.contains("[图片：季度收入趋势图]"));
}
