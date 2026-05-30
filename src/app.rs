use crate::cli::Cli;
use anyhow::{Context, Result, anyhow};
use glob::{MatchOptions, glob_with};
use pith::{
    ExtractOptions, JsonOutput, OutputMode, SourceInput, TableFilter, default_mode_for, extract_md,
    extract_table_entries, is_url, render_documents, render_json, resolve_input,
};

pub(crate) fn run(cli: Cli) -> Result<String> {
    let inputs = expand_inputs(&cli.inputs)?;
    let options = ExtractOptions {
        format: cli.format.map(Into::into),
    };

    // Resolve each input independently: one unreadable file or failed fetch
    // must not abort the whole batch. Failures are collected and surfaced as
    // warnings (stderr, plus the JSON envelope in json mode) so the remaining
    // inputs still produce output. We only fail hard (exit 1) when *nothing*
    // succeeds.
    let mut resolved = Vec::with_capacity(inputs.len());
    let mut failures: Vec<String> = Vec::new();
    for input in inputs {
        match resolve_input(SourceInput::from(input.clone()), &options) {
            Ok(r) => resolved.push(r),
            Err(e) => failures.push(format!("{input}: {}", e.root_cause())),
        }
    }

    let formats: Vec<_> = resolved.iter().map(|r| r.format).collect();
    let mode = cli.mode.unwrap_or_else(|| default_mode_for(&formats));

    match mode {
        OutputMode::Md => {
            warn_unused_narrowing(&cli);
            let mut documents = Vec::with_capacity(resolved.len());
            for r in &resolved {
                match extract_md(r) {
                    Ok(document) => documents.push(document),
                    Err(e) => failures.push(format!("{}: {}", r.label, e.root_cause())),
                }
            }
            if documents.is_empty() {
                return Err(all_failed_error(&failures));
            }
            report_skipped(&failures);
            render_documents(&documents, mode)
        }
        OutputMode::Json => {
            let filter = build_filter(&cli)?;
            let mut tables = Vec::new();
            let mut successes = 0usize;
            for r in &resolved {
                match extract_table_entries(r, &filter) {
                    Ok(entries) => {
                        successes += 1;
                        tables.extend(entries);
                    }
                    Err(e) => failures.push(format!("{}: {}", r.label, e.root_cause())),
                }
            }
            if successes == 0 {
                return Err(all_failed_error(&failures));
            }
            report_skipped(&failures);
            let mut output = JsonOutput::new(tables);
            output.warnings = failures;
            Ok(render_json(&output))
        }
    }
}

/// Emit one stderr line per skipped input. In json mode the same messages
/// also land in the envelope's top-level `warnings[]`; stderr keeps a human
/// running the command in a terminal informed regardless of output mode.
fn report_skipped(failures: &[String]) {
    for failure in failures {
        eprintln!("warning: skipped {failure}");
    }
}

/// Error returned when every input failed (exit 1). A lone failure is
/// surfaced verbatim; multiple are aggregated into one message.
fn all_failed_error(failures: &[String]) -> anyhow::Error {
    match failures {
        [only] => anyhow!("{only}"),
        _ => anyhow!(
            "all {} inputs failed:\n  - {}",
            failures.len(),
            failures.join("\n  - ")
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
        eprintln!(
            "warning: --sheet/--rows/--columns/--limit/--offset are ignored in markdown mode"
        );
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
}
