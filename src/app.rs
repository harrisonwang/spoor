use crate::cli::Cli;
use anyhow::{Context, Result, anyhow};
use glob::{MatchOptions, glob_with};
use pith::{
    ExtractOptions, JsonOutput, OutputMode, SourceInput, StructuredError, TableFilter,
    default_mode_for, extract_md, extract_table_entries, is_url, limit_markdown_output,
    render_documents, render_json_limited, resolve_input,
};

const MAX_FAILURE_DIAGNOSTICS: usize = 20;

pub(crate) fn run(cli: Cli) -> Result<String> {
    validate_max_output_bytes(cli.max_output_bytes)?;
    validate_max_parse_bytes(cli.max_parse_bytes)?;
    let inputs = expand_inputs(&cli.inputs)?;
    let format = cli.format.map(Into::into);

    // Resolve each input independently: one unreadable file or failed fetch
    // must not abort the whole batch. Failures are collected and surfaced as
    // warnings (stderr, plus the JSON envelope in json mode) so the remaining
    // inputs still produce output. We only fail hard (exit 1) when *nothing*
    // succeeds.
    let mut resolved = Vec::with_capacity(inputs.len());
    let mut failures = Vec::new();
    let mut source_bytes = 0usize;
    for input in inputs {
        let remaining = cli.max_parse_bytes.saturating_sub(source_bytes);
        let options = ExtractOptions {
            format,
            max_parse_bytes: remaining,
        };
        match resolve_input(SourceInput::from(input.clone()), &options) {
            Ok(r) => {
                source_bytes += r.source.len();
                resolved.push(r);
            }
            Err(e) => failures.push(InputFailure::from_error(input, &e)),
        }
    }

    let formats: Vec<_> = resolved.iter().map(|r| r.format).collect();
    let mode = cli.mode.unwrap_or_else(|| default_mode_for(&formats));

    match mode {
        OutputMode::Md => {
            warn_unused_narrowing(&cli);
            let mut documents = Vec::with_capacity(resolved.len());
            let mut extracted_bytes = 0usize;
            for r in &resolved {
                match extract_md(r) {
                    Ok(document) => {
                        if let Err(error) = retain_within_parse_budget(
                            &mut extracted_bytes,
                            document.markdown.len(),
                            cli.max_parse_bytes,
                            "retained extracted documents",
                        ) {
                            failures.push(InputFailure::from_error(r.label.clone(), &error));
                        } else {
                            documents.push(document);
                        }
                    }
                    Err(e) => failures.push(InputFailure::from_error(r.label.clone(), &e)),
                }
            }
            if documents.is_empty() {
                return Err(all_failed_error(&failures));
            }
            report_skipped(&failures);
            let markdown = render_documents(&documents, mode)?;
            let limited = limit_markdown_output(markdown, cli.max_output_bytes);
            report_output_truncation(limited.warning.as_deref());
            Ok(limited.content)
        }
        OutputMode::Json => {
            let filter = build_filter(&cli)?;
            let mut tables = Vec::new();
            let mut successes = 0usize;
            let mut extracted_bytes = 0usize;
            for r in &resolved {
                match extract_table_entries(r, &filter) {
                    Ok(extracted) => {
                        if let Err(error) = retain_within_parse_budget(
                            &mut extracted_bytes,
                            extracted.serialized_bytes,
                            cli.max_parse_bytes,
                            "retained extracted tables",
                        ) {
                            failures.push(InputFailure::from_error(r.label.clone(), &error));
                        } else {
                            successes += 1;
                            tables.extend(extracted.entries);
                        }
                    }
                    Err(e) => failures.push(InputFailure::from_error(r.label.clone(), &e)),
                }
            }
            if successes == 0 {
                return Err(all_failed_error(&failures));
            }
            report_skipped(&failures);
            let mut output = JsonOutput::new(tables);
            output.warnings = failures.iter().map(InputFailure::display).collect();
            let limited = render_json_limited(&output, cli.max_output_bytes);
            report_output_truncation(limited.warning.as_deref());
            Ok(limited.content)
        }
    }
}

fn validate_max_output_bytes(max_output_bytes: usize) -> Result<()> {
    if max_output_bytes < pith::MIN_MAX_OUTPUT_BYTES {
        return Err(anyhow!(
            "--max-output-bytes 不能小于 {}",
            pith::MIN_MAX_OUTPUT_BYTES
        ));
    }
    Ok(())
}

fn validate_max_parse_bytes(max_parse_bytes: usize) -> Result<()> {
    if max_parse_bytes < pith::MIN_MAX_PARSE_BYTES {
        return Err(anyhow!(
            "--max-parse-bytes 不能小于 {}",
            pith::MIN_MAX_PARSE_BYTES
        ));
    }
    Ok(())
}

fn retain_within_parse_budget(
    retained: &mut usize,
    additional: usize,
    max_parse_bytes: usize,
    stage: &str,
) -> Result<()> {
    let total = retained.checked_add(additional).unwrap_or(usize::MAX);
    if total > max_parse_bytes {
        return Err(StructuredError::parse_memory_limit(max_parse_bytes, stage).into());
    }
    *retained = total;
    Ok(())
}

fn report_output_truncation(warning: Option<&str>) {
    if let Some(warning) = warning {
        eprintln!("warning: {warning}");
    }
}

#[derive(Debug)]
struct InputFailure {
    source: String,
    message: String,
    structured: Option<StructuredError>,
}

impl InputFailure {
    fn from_error(source: String, error: &anyhow::Error) -> Self {
        let structured = error.downcast_ref::<StructuredError>().cloned();
        let message = structured
            .as_ref()
            .map(StructuredError::to_json)
            .unwrap_or_else(|| error.root_cause().to_string());

        Self {
            source,
            message,
            structured,
        }
    }

    fn display(&self) -> String {
        format!("{}: {}", self.source, self.message)
    }
}

/// Emit one stderr line per skipped input. In json mode the same messages
/// also land in the envelope's top-level `warnings[]`; stderr keeps a human
/// running the command in a terminal informed regardless of output mode.
fn report_skipped(failures: &[InputFailure]) {
    for failure in failures.iter().take(MAX_FAILURE_DIAGNOSTICS) {
        eprintln!("warning: 已跳过 {}", failure.display());
    }
    if failures.len() > MAX_FAILURE_DIAGNOSTICS {
        eprintln!(
            "warning: 另有 {} 条被跳过输入的警告未显示",
            failures.len() - MAX_FAILURE_DIAGNOSTICS
        );
    }
}

/// Error returned when every input failed (exit 1). A lone failure is
/// surfaced verbatim; multiple are aggregated into one message.
fn all_failed_error(failures: &[InputFailure]) -> anyhow::Error {
    match failures {
        [
            InputFailure {
                structured: Some(error),
                ..
            },
        ] => anyhow::Error::new(error.clone()),
        [only] => anyhow!(only.display()),
        _ => anyhow!(
            "全部 {} 个输入均失败：\n  - {}{}",
            failures.len(),
            failures
                .iter()
                .take(MAX_FAILURE_DIAGNOSTICS)
                .map(InputFailure::display)
                .collect::<Vec<_>>()
                .join("\n  - "),
            if failures.len() > MAX_FAILURE_DIAGNOSTICS {
                format!(
                    "\n  - …… 另有 {} 个失败未显示",
                    failures.len() - MAX_FAILURE_DIAGNOSTICS
                )
            } else {
                String::new()
            }
        ),
    }
}

fn build_filter(cli: &Cli) -> Result<TableFilter> {
    let row_range = match &cli.rows {
        Some(s) => Some(parse_row_range(s)?),
        None => None,
    };

    Ok(TableFilter {
        sheet: cli.sheet.clone(),
        row_range,
        columns: cli.columns.clone(),
        limit: cli.limit,
        offset: cli.offset,
    })
}

fn parse_row_range(s: &str) -> Result<(usize, usize)> {
    let (first, last) = s
        .split_once(':')
        .ok_or_else(|| anyhow!("--rows expects <first>:<last>, got {s:?}"))?;
    let first: usize = first
        .trim()
        .parse()
        .map_err(|_| anyhow!("--rows: invalid first row {first:?}"))?;
    let last: usize = last
        .trim()
        .parse()
        .map_err(|_| anyhow!("--rows: invalid last row {last:?}"))?;
    if first == 0 || last == 0 {
        return Err(anyhow!("--rows: row numbers must be >= 1, got {s:?}"));
    }
    if first > last {
        return Err(anyhow!("--rows: first ({first}) > last ({last})"));
    }
    Ok((first, last))
}

fn warn_unused_narrowing(cli: &Cli) {
    let used = cli.sheet.is_some()
        || cli.rows.is_some()
        || !cli.columns.is_empty()
        || cli.limit.is_some()
        || cli.offset.is_some();
    if used {
        eprintln!("warning: Markdown 模式下会忽略 --sheet/--rows/--columns/--limit/--offset");
    }
}

fn expand_inputs(inputs: &[String]) -> Result<Vec<String>> {
    let mut expanded = Vec::new();

    for input in inputs {
        if is_url(input) || !has_glob_meta(input) {
            expanded.push(input.clone());
            continue;
        }

        expanded
            .extend(expand_glob(input).with_context(|| format!("failed to expand glob: {input}"))?);
    }

    Ok(expanded)
}

fn expand_glob(pattern: &str) -> Result<Vec<String>> {
    let options = MatchOptions {
        case_sensitive: !cfg!(windows),
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };

    let mut matches = glob_with(pattern, options)
        .with_context(|| format!("invalid glob pattern: {pattern}"))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to read glob matches: {pattern}"))?
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    matches.sort();
    matches.dedup();

    if matches.is_empty() {
        return Err(anyhow!("glob matched no files: {pattern}"));
    }

    Ok(matches)
}

fn has_glob_meta(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

#[cfg(test)]
struct TestDir {
    path: std::path::PathBuf,
}

#[cfg(test)]
impl TestDir {
    fn new(name: &str) -> Result<Self> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("pith-{name}-{unique}"));
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

#[cfg(test)]
impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_inputs_expand_and_sort() -> Result<()> {
        let dir = TestDir::new("glob_inputs_expand_and_sort")?;
        std::fs::write(dir.path().join("b.pdf"), b"")?;
        std::fs::write(dir.path().join("a.pdf"), b"")?;
        std::fs::write(dir.path().join("notes.txt"), b"")?;

        let pattern = dir.path().join("*.pdf").to_string_lossy().into_owned();
        let expanded = expand_inputs(&[pattern])?;
        let names = expanded
            .iter()
            .map(|path| {
                std::path::Path::new(path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();

        assert_eq!(names, ["a.pdf", "b.pdf"]);
        Ok(())
    }

    #[test]
    fn glob_without_matches_errors() -> Result<()> {
        let dir = TestDir::new("glob_without_matches_errors")?;
        let pattern = dir.path().join("*.pdf").to_string_lossy().into_owned();
        let err = expand_inputs(&[pattern]).unwrap_err();

        assert!(err.to_string().contains("failed to expand glob"));
        assert!(format!("{err:#}").contains("glob matched no files"));
        Ok(())
    }

    #[test]
    fn parse_row_range_accepts_valid_input() {
        assert_eq!(parse_row_range("5:104").unwrap(), (5, 104));
        assert_eq!(parse_row_range("1:1").unwrap(), (1, 1));
        assert_eq!(parse_row_range(" 5 : 104 ").unwrap(), (5, 104));
    }

    #[test]
    fn parse_row_range_rejects_invalid_input() {
        assert!(parse_row_range("5").is_err());
        assert!(parse_row_range("a:b").is_err());
        assert!(parse_row_range("104:5").is_err());
        assert!(parse_row_range("0:10").is_err());
    }

    #[test]
    fn all_failed_error_caps_diagnostics() {
        let failures = (0..25)
            .map(|index| InputFailure {
                source: format!("input-{index}"),
                message: "failed".to_string(),
                structured: None,
            })
            .collect::<Vec<_>>();

        let message = all_failed_error(&failures).to_string();
        assert!(message.contains("全部 25 个输入均失败"));
        assert!(message.contains("input-19"));
        assert!(!message.contains("input-20"));
        assert!(message.contains("另有 5 个失败未显示"));
    }

    #[test]
    fn retained_results_share_parse_budget() {
        let mut retained = 600;
        let error = retain_within_parse_budget(&mut retained, 500, 1024, "test").unwrap_err();

        assert_eq!(retained, 600);
        assert!(error.downcast_ref::<StructuredError>().is_some());
    }
}
