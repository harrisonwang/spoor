//! Plain-text / encoding tests.

mod common;
use common::extract_fixture;
use gist::format::Format;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn gist_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gist"))
}

fn fixture_path(path: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
        .to_string_lossy()
        .into_owned()
}

#[test]
fn ascii_passthrough() {
    let out = extract_fixture("plain/01_ascii.txt", Format::PlainText);
    assert_eq!(out, "Hello world\nLine two\n");
}

#[test]
fn utf8_passthrough() {
    let out = extract_fixture("plain/02_utf8.txt", Format::PlainText);
    assert!(out.contains("中文"));
    assert!(out.contains("日本語"));
    assert!(out.contains("한글"));
}

#[test]
fn gbk_decoded() {
    let out = extract_fixture("plain/03_gbk.txt", Format::PlainText);
    assert!(out.contains("中文"));
    assert!(out.contains("第二行"));
    // Ensure the bytes were actually decoded — the raw GBK bytes
    // for "中" are 0xD6 0xD0, which are NOT valid UTF-8 by themselves.
    assert!(!out.as_bytes().contains(&0xd6) || out.contains("中"));
}

#[test]
fn utf16_le_with_bom_decoded() {
    let out = extract_fixture("plain/04_utf16le_bom.txt", Format::PlainText);
    assert!(out.contains("UTF-16 LE with BOM"));
    assert!(out.contains("Line 2"));
}

#[test]
fn code_file_passthrough() {
    let out = extract_fixture("plain/05_code.py", Format::PlainText);
    assert!(out.contains("def hello"));
    assert!(out.contains("hello('world')"));
}

#[test]
fn default_mode_is_markdown_like_text() {
    let output = gist_bin()
        .arg(fixture_path("html/06_links.html"))
        .output()
        .expect("run gist");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("[our site](https://example.com)"),
        "默认 md 模式应保留 Markdown 链接, got:\n{stdout}"
    );
    assert!(
        !stdout.trim_start().starts_with('{'),
        "默认模式不应输出 JSON 对象"
    );
}

#[test]
fn json_mode_is_flat_placeholder_schema() {
    let source = fixture_path("plain/01_ascii.txt");
    let output = gist_bin()
        .args(["--mode", "json", &source])
        .output()
        .expect("run gist");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let value: Value = serde_json::from_str(&stdout).expect("json stdout");

    assert_eq!(value["mode"], "json");
    assert_eq!(value["schema_version"], "gist-json-v0");
    assert_eq!(value["status"], "placeholder");
    assert_eq!(value["format"], "text");
    assert_eq!(value["source"], source);
    assert_eq!(value["content"], "Hello world\nLine two\n");
}
