use std::path::{Path, PathBuf};
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

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("spoor-cli-{name}-{unique}"));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn multiple_inputs_get_source_sections() {
    let first = fixture_path("plain/01_ascii.txt");
    let second = fixture_path("plain/02_utf8.txt");

    let output = spoor_bin()
        .args([&first, &second])
        .output()
        .expect("run spoor");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains(&format!("# Source: {first}")));
    assert!(stdout.contains(&format!("# Source: {second}")));
    assert!(stdout.contains("Hello world"));
    assert!(stdout.contains("中文"));
}

#[test]
fn glob_inputs_expand_inside_cli() {
    let dir = TestDir::new("glob_inputs_expand_inside_cli");
    std::fs::write(dir.path().join("b.txt"), "bravo\n").unwrap();
    std::fs::write(dir.path().join("a.txt"), "alpha\n").unwrap();
    std::fs::write(dir.path().join("skip.md"), "skip\n").unwrap();

    let pattern = dir.path().join("*.txt").to_string_lossy().into_owned();
    let output = spoor_bin().arg(pattern).output().expect("run spoor");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("# Source:"));
    assert!(stdout.find("a.txt").unwrap() < stdout.find("b.txt").unwrap());
    assert!(stdout.contains("alpha"));
    assert!(stdout.contains("bravo"));
    assert!(!stdout.contains("skip"));
}

#[test]
fn format_override_is_honored() {
    let source = fixture_path("html/06_links.html");
    let output = spoor_bin()
        .args(["--format", "text", &source])
        .output()
        .expect("run spoor");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("<a href=\"https://example.com\""));
    assert!(!stdout.contains("[our site](https://example.com)"));
}

#[test]
fn extraction_errors_exit_nonzero() {
    let source = fixture_path("plain/01_ascii.txt");
    let output = spoor_bin()
        .args(["--format", "docx", &source])
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    // Forcing --format docx onto a text file is an unreadable container:
    // a lone failing input surfaces the structured envelope directly.
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr is pure JSON");
    assert_eq!(value["code"], "invalid_container");
}

#[test]
fn image_only_pdf_emits_machine_readable_error() {
    let source = fixture_path("pdf/04_image_only.pdf");
    let output = spoor_bin().arg(source).output().expect("run spoor");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr is pure JSON");
    assert_eq!(value["is_error"], true);
    assert_eq!(value["code"], "image_only_pdf");
    assert_eq!(value["reason"], "纯图片 PDF（无文本层）");
    assert_eq!(
        value["hint"],
        "该 PDF 没有文本层，需要 OCR，但 spoor 不执行 OCR。"
    );
    assert_eq!(value["recoverable"], true);
}

#[test]
fn cfb_container_emits_legacy_or_encrypted_office_error() {
    let dir = TestDir::new("cfb_container_emits_legacy_or_encrypted_office_error");
    let source = dir.path().join("locked.docx");
    // OLE/CFB magic: what a password-protected OOXML file (or a legacy
    // .doc/.xls/.ppt) starts with. Must be intercepted before the docx
    // extractor turns it into an opaque "invalid Zip archive" failure.
    let mut bytes = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    bytes.extend_from_slice(&[0u8; 512]);
    std::fs::write(&source, bytes).unwrap();

    let output = spoor_bin()
        .arg(source.to_string_lossy().as_ref())
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr is pure JSON");
    assert_eq!(value["code"], "legacy_or_encrypted_office");
    assert_eq!(value["recoverable"], false);
}

#[test]
fn unreadable_archive_emits_invalid_container_error() {
    let dir = TestDir::new("unreadable_archive_emits_invalid_container_error");
    let source = dir.path().join("empty.docx");
    std::fs::write(&source, b"").unwrap();

    let output = spoor_bin()
        .arg(source.to_string_lossy().as_ref())
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr is pure JSON");
    assert_eq!(value["code"], "invalid_container");
    assert_eq!(value["recoverable"], true);
    assert!(value["reason"].as_str().unwrap().contains("docx"));
}

#[test]
fn empty_extraction_emits_inband_placeholder() {
    let dir = TestDir::new("empty_extraction_emits_inband_placeholder");
    let source = dir.path().join("empty.txt");
    std::fs::write(&source, b"").unwrap();

    let output = spoor_bin()
        .arg(source.to_string_lossy().as_ref())
        .output()
        .expect("run spoor");

    // Empty content is a readable state, not a silent empty stdout: the
    // placeholder names the source and format so an agent can tell "file
    // has no text" from "spoor failed".
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("> [!NOTE]"));
    assert!(stdout.contains("未从"));
    assert!(stdout.contains("format=text"));
    assert!(stdout.contains("empty.txt"));
}

#[test]
fn unsupported_format_emits_machine_readable_error() {
    let dir = TestDir::new("unsupported_format_emits_machine_readable_error");
    let source = dir.path().join("mystery.bin");
    // Unknown extension, no magic bytes, and a NUL byte so it isn't sniffed as
    // text: detection must fail with a structured `unsupported_format` whose
    // `code` an agent can branch on without parsing the prose message.
    std::fs::write(&source, [0x00u8, 0x01, 0x02, 0xff, 0xfe, 0x00, 0x10]).unwrap();

    let output = spoor_bin()
        .arg(source.to_string_lossy().as_ref())
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr is pure JSON");
    assert_eq!(value["is_error"], true);
    assert_eq!(value["code"], "unsupported_format");
    assert_eq!(value["recoverable"], true);
}

#[test]
fn version_flag_reports_binary_name() {
    let output = spoor_bin().arg("--version").output().expect("run spoor");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.starts_with("spoor "));
}

#[test]
fn partial_failure_still_outputs_successes_in_md() {
    let good = fixture_path("plain/01_ascii.txt");
    let missing = fixture_path("plain/does_not_exist.txt");

    let output = spoor_bin()
        .args([&good, &missing])
        .output()
        .expect("run spoor");

    // One input failed but another succeeded: exit 0, success still printed.
    assert!(output.status.success(), "partial success should exit 0");
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("Hello world"));
    // The skipped input must be visible in stdout too (agents often only
    // read stdout), mirroring the JSON envelope's in-band warnings[].
    assert!(stdout.contains("> [!WARNING]"));
    assert!(stdout.contains("已跳过 1 个"));
    assert!(stdout.contains("does_not_exist.txt"));
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("warning: 已跳过"));
    assert!(stderr.contains("does_not_exist.txt"));
}

#[test]
fn partial_failure_records_warning_in_json_envelope() {
    let good = fixture_path("csv/01_basic.csv");
    let missing = fixture_path("csv/does_not_exist.csv");

    let output = spoor_bin()
        .args([&good, &missing])
        .output()
        .expect("run spoor");

    assert!(output.status.success(), "partial success should exit 0");
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json stdout");

    assert!(
        !value["tables"].as_array().expect("tables array").is_empty(),
        "the readable CSV should still produce a table"
    );
    let warnings = value["warnings"].as_array().expect("warnings array");
    assert_eq!(warnings.len(), 1, "the missing file should be one warning");
    assert!(
        warnings[0]
            .as_str()
            .expect("warning string")
            .contains("does_not_exist.csv")
    );
}

#[test]
fn all_inputs_failing_exits_nonzero() {
    let a = fixture_path("plain/missing_a.txt");
    let b = fixture_path("plain/missing_b.txt");

    let output = spoor_bin().args([&a, &b]).output().expect("run spoor");

    assert!(
        !output.status.success(),
        "total failure should exit nonzero"
    );
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("error:"));
    assert!(stderr.contains("全部 2 个输入均失败"));
    assert!(stderr.contains("missing_a.txt"));
    assert!(stderr.contains("missing_b.txt"));
}

#[test]
fn markdown_total_output_limit_is_global_and_visible() {
    let dir = TestDir::new("markdown_total_output_limit_is_global_and_visible");
    let first = dir.path().join("a.txt");
    let second = dir.path().join("b.txt");
    std::fs::write(&first, "alpha line\n".repeat(200)).unwrap();
    std::fs::write(&second, "bravo line\n".repeat(200)).unwrap();

    let output = spoor_bin()
        .args([
            "--max-output-bytes",
            "1024",
            &first.to_string_lossy(),
            &second.to_string_lossy(),
        ])
        .output()
        .expect("run spoor");

    assert!(output.status.success());
    assert!(output.stdout.len() <= 1024);
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("> [!WARNING]"));
    assert!(stdout.contains("内容不完整"));

    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("warning: spoor 输出"));
    assert!(stderr.contains("--max-output-bytes"));
}

#[test]
fn markdown_uses_default_total_output_limit() {
    let dir = TestDir::new("markdown_uses_default_total_output_limit");
    let source = dir.path().join("huge.txt");
    std::fs::write(&source, "large document line\n".repeat(20_000)).unwrap();

    let output = spoor_bin().arg(source).output().expect("run spoor");

    assert!(output.status.success());
    assert!(output.stdout.len() <= spoor_core::DEFAULT_MAX_OUTPUT_BYTES);
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("内容不完整"));
}

#[test]
fn max_output_bytes_rejects_too_small_budget() {
    let source = fixture_path("plain/01_ascii.txt");
    let output = spoor_bin()
        .args(["--max-output-bytes", "100", &source])
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("--max-output-bytes 不能小于 1024"));
}

#[test]
fn local_file_over_parse_budget_emits_structured_error() {
    let dir = TestDir::new("local_file_over_parse_budget_emits_structured_error");
    let source = dir.path().join("large.txt");
    std::fs::write(&source, vec![b'x'; 2048]).unwrap();

    let output = spoor_bin()
        .args(["--max-parse-bytes", "1024", &source.to_string_lossy()])
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr is pure JSON");
    assert_eq!(value["reason"], "超出解析预算");
    assert!(
        value["hint"]
            .as_str()
            .unwrap()
            .contains("--max-parse-bytes")
    );
    assert_eq!(value["recoverable"], true);
}

#[test]
fn multiple_inputs_share_parse_budget() {
    let dir = TestDir::new("multiple_inputs_share_parse_budget");
    let first = dir.path().join("a.txt");
    let second = dir.path().join("b.txt");
    std::fs::write(&first, vec![b'a'; 700]).unwrap();
    std::fs::write(&second, vec![b'b'; 700]).unwrap();

    let output = spoor_bin()
        .args([
            "--max-parse-bytes",
            "1024",
            &first.to_string_lossy(),
            &second.to_string_lossy(),
        ])
        .output()
        .expect("run spoor");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains(&"a".repeat(100)));
    assert!(!stdout.contains(&"b".repeat(100)));
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("超出解析预算"));
}

#[test]
fn extracted_text_expansion_respects_parse_budget() {
    let dir = TestDir::new("extracted_text_expansion_respects_parse_budget");
    let source = dir.path().join("wide.csv");
    std::fs::write(&source, format!("{}\n", vec!["x"; 400].join(","))).unwrap();

    let output = spoor_bin()
        .args([
            "--format",
            "csv",
            "--mode",
            "md",
            "--max-parse-bytes",
            "1024",
            &source.to_string_lossy(),
        ])
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr is pure JSON");
    assert_eq!(value["reason"], "超出解析预算");
    assert!(
        value["hint"]
            .as_str()
            .unwrap()
            .contains("CSV Markdown rendering")
    );
}

#[test]
fn max_parse_bytes_rejects_too_small_budget() {
    let source = fixture_path("plain/01_ascii.txt");
    let output = spoor_bin()
        .args(["--max-parse-bytes", "100", &source])
        .output()
        .expect("run spoor");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("--max-parse-bytes 不能小于 1024"));
}

fn run_with_stdin(args: &[&str], input: &[u8]) -> std::process::Output {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = spoor_bin()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn spoor");
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(input)
        .expect("write to child stdin");
    child.wait_with_output().expect("wait for spoor")
}

#[test]
fn stdin_dash_reads_text_as_markdown() {
    let output = run_with_stdin(&["-"], b"hello from stdin\n");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("hello from stdin"));
}

#[test]
fn stdin_over_parse_budget_emits_structured_error() {
    let output = run_with_stdin(
        &["--format", "text", "--max-parse-bytes", "1024", "-"],
        &vec![b'x'; 2048],
    );

    assert!(!output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr is pure JSON");
    assert_eq!(value["reason"], "超出解析预算");
}

#[test]
fn stdin_csv_with_format_flag_emits_json() {
    let output = run_with_stdin(&["--format", "csv", "-"], b"a,b\n1,2\n3,4\n");

    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json stdout");
    let table = &value["tables"][0];
    assert_eq!(table["format"], "csv");
    assert_eq!(table["source"], "-");
    let headers = table["headers"].as_object().expect("headers object");
    assert!(headers.contains_key("a") && headers.contains_key("b"));
}

#[test]
fn stdin_xlsx_detected_by_magic_bytes() {
    // No filename to sniff: detection must fall back to the ZIP/OOXML magic
    // bytes and still route the workbook to table JSON.
    let bytes = std::fs::read(fixture_path("xlsx/01_basic.xlsx")).expect("read fixture");
    let output = run_with_stdin(&["-"], &bytes);

    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(value["tables"][0]["format"], "xlsx");
}
