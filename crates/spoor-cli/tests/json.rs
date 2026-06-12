use serde_json::{Value, json};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn spoor_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spoor"))
}

fn fixture_path(path: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
        .to_string_lossy()
        .into_owned()
}

fn spoor_json(args: &[String]) -> Value {
    let output = spoor_bin().args(args).output().expect("run spoor json");

    assert!(
        output.status.success(),
        "spoor failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json stdout")
}

fn json_args(paths: &[&str]) -> Vec<String> {
    let mut args = vec!["--mode".to_string(), "json".to_string()];
    args.extend(paths.iter().map(|path| fixture_path(path)));
    args
}

fn assert_envelope(value: &Value) {
    assert_eq!(value["schema_version"], "spoor-table-json-v2");
    assert!(
        value["usage"].is_string(),
        "usage must be a string telling consumers how to narrow"
    );
    assert!(value["tables"].is_array());
    assert!(value["truncated"].is_boolean());
    assert!(value["warnings"].is_array());
}

#[test]
fn csv_json_is_self_describing_envelope() {
    let value = spoor_json(&json_args(&["csv/01_basic.csv"]));
    assert_envelope(&value);

    let tables = value["tables"].as_array().unwrap();
    assert_eq!(tables.len(), 1);
    let table = &tables[0];

    assert_eq!(table["source"], fixture_path("csv/01_basic.csv"));
    assert_eq!(table["format"], "csv");
    assert_eq!(table["sheet"], json!(null));
    assert_eq!(table["workbook_sheets"], json!(null));
    assert!(
        table["delimiter"].is_string(),
        "CSV entries must expose the detected delimiter"
    );
    assert_eq!(table["header_row"], json!(null));
    assert_eq!(table["range"], "A1:C4");
    assert_eq!(table["column_count"], 3);

    let headers = &table["headers"];
    assert_eq!(headers["Name"]["column_index"], 0);
    assert_eq!(headers["Score"]["column_index"], 1);
    assert_eq!(headers["Note"]["column_index"], 2);

    assert_eq!(table["preamble"], json!(null));

    let rows = &table["rows"];
    assert_eq!(rows[0]["Name"], "Alice");
    assert_eq!(rows[0]["Score"], "95");
    assert_eq!(rows[0]["Note"], "first");
    assert_eq!(rows[1]["Name"], "Bob");
    assert_eq!(rows[2]["Name"], "Carol");

    assert_eq!(table["row_range"]["first"], 2);
    assert_eq!(table["row_range"]["last"], 4);

    assert_eq!(table["truncated"], false);
    assert_eq!(table["warnings"], json!([]));
}

#[test]
fn multi_input_flattens_into_one_tables_array() {
    let value = spoor_json(&json_args(&["csv/01_basic.csv", "xlsx/01_basic.xlsx"]));
    assert_envelope(&value);

    let tables = value["tables"].as_array().unwrap();
    assert_eq!(tables.len(), 2);

    assert_eq!(tables[0]["format"], "csv");
    assert_eq!(tables[0]["headers"]["Name"]["column_index"], 0);

    assert_eq!(tables[1]["format"], "xlsx");
    assert_eq!(tables[1]["sheet"], "Data");
}

#[test]
fn xlsx_json_preserves_sheet_workbook_sheets_and_range() {
    let value = spoor_json(&json_args(&["xlsx/02_multi_sheets.xlsx"]));
    assert_envelope(&value);

    let tables = value["tables"].as_array().unwrap();
    assert!(!tables.is_empty());

    let first = &tables[0];
    assert_eq!(first["format"], "xlsx");
    assert_eq!(first["sheet"], "First");
    assert_eq!(first["range"], "A1:B2");
    assert_eq!(first["header_row"], 1);

    let workbook_sheets = first["workbook_sheets"].as_array().unwrap();
    assert!(
        workbook_sheets.contains(&json!("First")),
        "workbook_sheets must list every sheet in the workbook"
    );

    let headers = &first["headers"];
    assert_eq!(headers["a"]["column_index"], 0);
    assert_eq!(headers["b"]["column_index"], 1);

    let rows = &first["rows"];
    assert_eq!(rows[0]["a"], "1");
    assert_eq!(rows[0]["b"], "2");

    assert_eq!(first["row_range"]["first"], 2);
    assert_eq!(first["row_range"]["last"], 2);
}

#[test]
fn large_csv_json_uses_preview_and_truncation_warning() {
    let value = spoor_json(&json_args(&["csv/10_large.csv"]));
    let table = &value["tables"][0];

    assert_eq!(table["headers"]["id"]["column_index"], 0);
    assert_eq!(table["truncated"], true);
    assert_eq!(
        table["warnings"][0],
        "preview limited to first 100 data rows out of 2000"
    );
    assert_eq!(table["rows"].as_array().unwrap().len(), 100);
    assert_eq!(table["rows"][0]["id"], "0");
    assert_eq!(table["row_range"]["first"], 2);
    assert_eq!(table["row_range"]["last"], 101);
}

#[test]
fn xlsx_json_detects_title_preamble_and_real_header() {
    let path = titled_table_fixture();
    let value = spoor_json(&json_args_from_paths(&[path
        .to_string_lossy()
        .into_owned()]));
    let table = &value["tables"][0];

    assert_eq!(table["sheet"], "L1");
    assert_eq!(table["title"], "L1 · 助理工程师 · 学徒");
    assert_eq!(table["header_row"], 4);

    let workbook_sheets = table["workbook_sheets"].as_array().unwrap();
    assert_eq!(workbook_sheets[0], "L1");

    let headers = &table["headers"];
    assert_eq!(headers["分类"]["column_index"], 0);
    assert_eq!(headers["技能"]["column_index"], 1);
    assert_eq!(headers["说明"]["column_index"], 2);
    assert_eq!(headers["需学内容"]["column_index"], 3);
    assert_eq!(headers["课程"]["column_index"], 4);

    let preamble = &table["preamble"];
    assert_eq!(preamble["row"], 2);
    assert_eq!(preamble["content"]["分类"], "核心定位");
    assert_eq!(preamble["content"]["技能"], "在指导下完成明确任务");

    let rows = &table["rows"];
    assert_eq!(rows[0]["分类"], "后端开发基础");
    assert_eq!(rows[0]["技能"], "Java 语法基础");
    assert_eq!(rows[0]["说明"], "能读懂基础 Java 代码");
    assert_eq!(rows[0]["课程"], "Java 代码阅读理解");

    assert_eq!(table["row_range"]["first"], 5);
    assert_eq!(table["row_range"]["last"], 6);

    let _ = std::fs::remove_file(path);
}

#[test]
fn xlsx_default_mode_is_json_without_explicit_flag() {
    let output = spoor_bin()
        .arg(fixture_path("xlsx/01_basic.xlsx"))
        .output()
        .expect("run spoor");

    assert!(
        output.status.success(),
        "spoor failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout)
        .expect("XLSX without -m flag should default to JSON output");
    assert_envelope(&value);
    assert_eq!(value["tables"][0]["format"], "xlsx");
}

#[test]
fn csv_default_mode_is_json_without_explicit_flag() {
    let output = spoor_bin()
        .arg(fixture_path("csv/01_basic.csv"))
        .output()
        .expect("run spoor");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout)
        .expect("CSV without -m flag should default to JSON output");
    assert_envelope(&value);
    assert_eq!(value["tables"][0]["format"], "csv");
}

#[test]
fn prose_format_with_explicit_json_errors() {
    let output = spoor_bin()
        .args(["-m", "json", &fixture_path("plain/01_ascii.txt")])
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("csv 和 xlsx"),
        "stderr should mention csv/xlsx-only json support; got: {stderr}"
    );
}

#[test]
fn xlsx_with_explicit_md_falls_back_to_markdown() {
    let output = spoor_bin()
        .args(["-m", "md", &fixture_path("xlsx/01_basic.xlsx")])
        .output()
        .expect("run spoor");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("## Sheet:"),
        "XLSX with -m md should output Markdown headings; got: {stdout}"
    );
}

#[test]
fn mixed_table_and_prose_defaults_to_markdown() {
    let output = spoor_bin()
        .args([
            &fixture_path("xlsx/01_basic.xlsx"),
            &fixture_path("plain/01_ascii.txt"),
        ])
        .output()
        .expect("run spoor");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Source:"));
    assert!(stdout.contains("## Sheet:"));
}

#[test]
fn usage_string_describes_real_flags() {
    let value = spoor_json(&json_args(&["csv/01_basic.csv"]));
    let usage = value["usage"].as_str().unwrap();
    for flag in ["--sheet", "--rows", "--columns", "--limit", "--offset"] {
        assert!(usage.contains(flag), "usage must mention {flag}: {usage}");
    }
    assert!(!usage.to_lowercase().contains("planned"));
    // The emitted hint is exactly the published constant, so consumers and
    // docs have a single source of truth to track.
    assert_eq!(usage, spoor_core::TABLE_USAGE);
}

#[test]
fn sheet_filter_keeps_only_matching_sheet() {
    let value = spoor_json(&[
        "--mode".to_string(),
        "json".to_string(),
        "--sheet".to_string(),
        "Second".to_string(),
        fixture_path("xlsx/02_multi_sheets.xlsx"),
    ]);

    let tables = value["tables"].as_array().unwrap();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0]["sheet"], "Second");
    // workbook_sheets still lists every sheet so consumers know what else exists
    let workbook_sheets = tables[0]["workbook_sheets"].as_array().unwrap();
    assert!(workbook_sheets.contains(&json!("First")));
    assert!(workbook_sheets.contains(&json!("Empty")));
}

#[test]
fn sheet_filter_missing_errors_with_available_list() {
    let output = spoor_bin()
        .args([
            "-m",
            "json",
            "--sheet",
            "NoSuch",
            &fixture_path("xlsx/02_multi_sheets.xlsx"),
        ])
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    let value: Value = serde_json::from_slice(&output.stderr).expect("structured error");
    let reason = value["reason"].as_str().expect("error reason");
    assert!(
        reason.contains("\"NoSuch\""),
        "stderr should quote the missing sheet name: {reason}"
    );
    assert!(
        reason.contains("available sheets") && reason.contains("First"),
        "stderr should list available sheets: {reason}"
    );
}

#[test]
fn sheet_filter_is_noop_for_csv() {
    let value = spoor_json(&[
        "--mode".to_string(),
        "json".to_string(),
        "--sheet".to_string(),
        "Anything".to_string(),
        fixture_path("csv/01_basic.csv"),
    ]);

    let tables = value["tables"].as_array().unwrap();
    assert_eq!(tables.len(), 1, "CSV ignores --sheet rather than erroring");
    assert_eq!(tables[0]["format"], "csv");
}

#[test]
fn rows_filter_returns_excel_row_range() {
    let path = titled_table_fixture();
    // Fixture has header at Excel row 4, data at rows 5-6.
    let value = spoor_json(&[
        "-m".to_string(),
        "json".to_string(),
        "--rows".to_string(),
        "5:5".to_string(),
        path.to_string_lossy().into_owned(),
    ]);

    let table = &value["tables"][0];
    let rows = table["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["分类"], "后端开发基础");
    assert_eq!(table["row_range"]["first"], 5);
    assert_eq!(table["row_range"]["last"], 5);
    assert_eq!(table["truncated"], true);
    let warning = table["warnings"][0].as_str().unwrap();
    assert!(
        warning.contains("--rows 5:5"),
        "warning should reference the requested range: {warning}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn rows_filter_excluding_data_returns_empty_rows() {
    let path = titled_table_fixture();
    // Excel rows 1..3 are title/preamble/blank; no data rows in this range.
    let value = spoor_json(&[
        "-m".to_string(),
        "json".to_string(),
        "--rows".to_string(),
        "1:3".to_string(),
        path.to_string_lossy().into_owned(),
    ]);

    let table = &value["tables"][0];
    assert_eq!(table["rows"].as_array().unwrap().len(), 0);
    assert!(table["title"].is_string(), "title should still be emitted");
    assert_eq!(table["truncated"], true);

    let _ = std::fs::remove_file(path);
}

#[test]
fn columns_filter_keeps_only_selected_keys() {
    let path = titled_table_fixture();
    let value = spoor_json(&[
        "-m".to_string(),
        "json".to_string(),
        "--columns".to_string(),
        "分类,技能".to_string(),
        path.to_string_lossy().into_owned(),
    ]);

    let table = &value["tables"][0];
    let headers = table["headers"].as_object().unwrap();
    assert_eq!(headers.len(), 2);
    assert!(headers.contains_key("分类"));
    assert!(headers.contains_key("技能"));
    // Original column_index preserved
    assert_eq!(headers["分类"]["column_index"], 0);
    assert_eq!(headers["技能"]["column_index"], 1);
    // column_count still reflects the full source sheet
    assert_eq!(table["column_count"], 5);

    let row = &table["rows"][0];
    let row_keys: Vec<&str> = row
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(row_keys.len(), 2);
    assert!(row_keys.contains(&"分类"));
    assert!(row_keys.contains(&"技能"));

    let _ = std::fs::remove_file(path);
}

#[test]
fn columns_filter_missing_errors_with_available_list() {
    let path = titled_table_fixture();
    let output = spoor_bin()
        .args([
            "-m",
            "json",
            "--columns",
            "no_such_column",
            &path.to_string_lossy(),
        ])
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no_such_column"),
        "stderr should quote the missing column: {stderr}"
    );
    assert!(
        stderr.contains("available columns") && stderr.contains("分类"),
        "stderr should list available columns: {stderr}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn limit_caps_data_rows() {
    let value = spoor_json(&[
        "-m".to_string(),
        "json".to_string(),
        "--limit".to_string(),
        "10".to_string(),
        fixture_path("csv/10_large.csv"),
    ]);

    let table = &value["tables"][0];
    assert_eq!(table["rows"].as_array().unwrap().len(), 10);
    assert_eq!(table["truncated"], true);
}

#[test]
fn total_output_limit_keeps_json_valid_and_marks_truncation() {
    let output = spoor_bin()
        .args([
            "-m",
            "json",
            "--limit",
            "2000",
            "--max-output-bytes",
            "2048",
            &fixture_path("csv/10_large.csv"),
        ])
        .output()
        .expect("run spoor");

    assert!(output.status.success());
    assert!(output.stdout.len() <= 2048);

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid truncated JSON");
    assert_eq!(value["truncated"], true);
    assert!(
        value["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning.as_str().unwrap().contains("--max-output-bytes"))
    );

    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("warning: spoor 输出"));
}

#[test]
fn offset_skips_data_rows() {
    let value = spoor_json(&[
        "-m".to_string(),
        "json".to_string(),
        "--offset".to_string(),
        "5".to_string(),
        "--limit".to_string(),
        "3".to_string(),
        fixture_path("csv/10_large.csv"),
    ]);

    let table = &value["tables"][0];
    let rows = table["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    // The 10_large fixture has `id` column starting from 0; offset 5 should
    // give us ids 5, 6, 7.
    assert_eq!(rows[0]["id"], "5");
    assert_eq!(rows[2]["id"], "7");
}

fn json_args_from_paths(paths: &[String]) -> Vec<String> {
    let mut args = vec!["--mode".to_string(), "json".to_string()];
    args.extend(paths.iter().cloned());
    args
}

fn titled_table_fixture() -> std::path::PathBuf {
    // Process-unique, monotonic name so concurrently running tests never share
    // a path (a wall-clock timestamp can collide under coarse clock resolution,
    // letting one test delete the file another is still reading).
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "spoor-titled-table-{}-{unique}.xlsx",
        std::process::id()
    ));
    write_titled_table_xlsx(&path);
    path
}

fn write_titled_table_xlsx(path: &Path) {
    let file = File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    write_zip_entry(
        &mut zip,
        options,
        "[Content_Types].xml",
        r#"<?xml version="1.0"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#,
    );
    write_zip_entry(
        &mut zip,
        options,
        "_rels/.rels",
        r#"<?xml version="1.0"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
    );
    write_zip_entry(
        &mut zip,
        options,
        "xl/workbook.xml",
        r#"<?xml version="1.0"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="L1" sheetId="1" r:id="rIdS1"/></sheets>
</workbook>"#,
    );
    write_zip_entry(
        &mut zip,
        options,
        "xl/_rels/workbook.xml.rels",
        r#"<?xml version="1.0"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rIdS1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#,
    );
    write_zip_entry(
        &mut zip,
        options,
        "xl/worksheets/sheet1.xml",
        r#"<?xml version="1.0"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<sheetData>
<row r="1"><c r="A1" t="inlineStr"><is><t>L1 · 助理工程师 · 学徒</t></is></c></row>
<row r="2"><c r="A2" t="inlineStr"><is><t>核心定位</t></is></c><c r="B2" t="inlineStr"><is><t>在指导下完成明确任务</t></is></c></row>
<row r="3"/>
<row r="4"><c r="A4" t="inlineStr"><is><t>分类</t></is></c><c r="B4" t="inlineStr"><is><t>技能</t></is></c><c r="C4" t="inlineStr"><is><t>说明</t></is></c><c r="D4" t="inlineStr"><is><t>需学内容</t></is></c><c r="E4" t="inlineStr"><is><t>课程</t></is></c></row>
<row r="5"><c r="A5" t="inlineStr"><is><t>后端开发基础</t></is></c><c r="B5" t="inlineStr"><is><t>Java 语法基础</t></is></c><c r="C5" t="inlineStr"><is><t>能读懂基础 Java 代码</t></is></c><c r="E5" t="inlineStr"><is><t>Java 代码阅读理解</t></is></c></row>
<row r="6"><c r="B6" t="inlineStr"><is><t>Spring Boot 项目能跑起来</t></is></c><c r="C6" t="inlineStr"><is><t>能启动项目并定位 Controller</t></is></c><c r="E6" t="inlineStr"><is><t>Spring Boot 项目结构与启动</t></is></c></row>
</sheetData>
</worksheet>"#,
    );

    zip.finish().unwrap();
}

fn write_zip_entry(
    zip: &mut zip::ZipWriter<File>,
    options: zip::write::SimpleFileOptions,
    name: &str,
    contents: &str,
) {
    zip.start_file(name, options).unwrap();
    zip.write_all(contents.as_bytes()).unwrap();
}
