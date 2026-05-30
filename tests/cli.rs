use std::path::{Path, PathBuf};
use std::process::Command;

fn pith_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pith"))
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
        let path = std::env::temp_dir().join(format!("pith-cli-{name}-{unique}"));
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

    let output = pith_bin()
        .args([&first, &second])
        .output()
        .expect("run pith");

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
    let output = pith_bin().arg(pattern).output().expect("run pith");

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
    let output = pith_bin()
        .args(["--format", "text", &source])
        .output()
        .expect("run pith");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("<a href=\"https://example.com\""));
    assert!(!stdout.contains("[our site](https://example.com)"));
}

#[test]
fn extraction_errors_exit_nonzero() {
    let source = fixture_path("plain/01_ascii.txt");
    let output = pith_bin()
        .args(["--format", "docx", &source])
        .output()
        .expect("run pith");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("error:"));
    // The failing input is named and the concise root cause (an invalid
    // zip/docx) is shown, not the full anyhow context chain.
    assert!(stderr.contains("01_ascii.txt"));
    assert!(stderr.to_lowercase().contains("zip"));
}

#[test]
fn version_flag_reports_binary_name() {
    let output = pith_bin().arg("--version").output().expect("run pith");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.starts_with("pith "));
}

#[test]
fn partial_failure_still_outputs_successes_in_md() {
    let good = fixture_path("plain/01_ascii.txt");
    let missing = fixture_path("plain/does_not_exist.txt");

    let output = pith_bin()
        .args([&good, &missing])
        .output()
        .expect("run pith");

    // One input failed but another succeeded: exit 0, success still printed.
    assert!(output.status.success(), "partial success should exit 0");
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("Hello world"));
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("warning: skipped"));
    assert!(stderr.contains("does_not_exist.txt"));
}

#[test]
fn partial_failure_records_warning_in_json_envelope() {
    let good = fixture_path("csv/01_basic.csv");
    let missing = fixture_path("csv/does_not_exist.csv");

    let output = pith_bin()
        .args([&good, &missing])
        .output()
        .expect("run pith");

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

    let output = pith_bin().args([&a, &b]).output().expect("run pith");

    assert!(
        !output.status.success(),
        "total failure should exit nonzero"
    );
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("error:"));
    assert!(stderr.contains("all 2 inputs failed"));
    assert!(stderr.contains("missing_a.txt"));
    assert!(stderr.contains("missing_b.txt"));
}

fn run_with_stdin(args: &[&str], input: &[u8]) -> std::process::Output {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = pith_bin()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn pith");
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(input)
        .expect("write to child stdin");
    child.wait_with_output().expect("wait for pith")
}

#[test]
fn stdin_dash_reads_text_as_markdown() {
    let output = run_with_stdin(&["-"], b"hello from stdin\n");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("hello from stdin"));
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
