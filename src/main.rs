use anyhow::{Context, Result, anyhow};
use clap::{ArgAction, Parser};
use std::fmt;

#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::{Path, PathBuf};

use gist::extractors;
use gist::format::{self, Format, FormatArg};
use gist::source::Source;
use glob::{MatchOptions, glob_with};

const HELP_TEMPLATE: &str = "\
{about}

用法 (Usage):
  {usage}

参数 (Arguments):
{positionals}

选项 (Options):
{options}

示例 (Examples):
  gist report.pdf
  gist --format html https://example.com/article
  gist -m json notes.md
  gist *.pdf
  gist report.pdf | llm \"Summarize risks and action items\"
";

#[derive(Parser, Debug)]
#[command(
    name = "gist",
    version,
    about = "将文件或 URL 转换为 LLM-friendly Markdown",
    long_about = None,
    override_usage = "gist [OPTIONS] <input>...",
    help_template = HELP_TEMPLATE,
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    /// 文件路径、URL 或本地 glob，可传多个。
    #[arg(value_name = "input", required = true, num_args = 1..)]
    inputs: Vec<String>,

    /// 覆盖自动 format 检测；可选：html、markdown、pdf、docx、xlsx、pptx、csv、ipynb、epub、text。
    #[arg(long, value_enum, value_name = "format", hide_possible_values = true)]
    format: Option<FormatArg>,

    /// 输出 mode；md 输出 Markdown 正文，json 输出 JSON 包装；默认 md。可选：md、json。
    #[arg(
        long,
        short = 'm',
        value_enum,
        default_value_t = OutputMode::Md,
        value_name = "mode",
        hide_possible_values = true,
        hide_default_value = true
    )]
    mode: OutputMode,

    /// 显示帮助。
    #[arg(short = 'h', long = "help", action = ArgAction::Help)]
    help: Option<bool>,

    /// 显示版本。
    #[arg(short = 'V', long = "version", action = ArgAction::Version)]
    version: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum OutputMode {
    Md,
    Json,
}

impl fmt::Display for OutputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            OutputMode::Md => "md",
            OutputMode::Json => "json",
        })
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let inputs = expand_inputs(&cli.inputs)?;
    let mut documents = Vec::with_capacity(inputs.len());
    for input in inputs {
        documents.push(process_input(&input, cli.format)?);
    }

    match cli.mode {
        OutputMode::Md => write_markdown_output(&documents),
        OutputMode::Json => write_json_output(&documents),
    }
    Ok(())
}

#[derive(Debug)]
struct ProcessedInput {
    source: String,
    format: Format,
    markdown: String,
}

fn process_input(input: &str, format_override: Option<FormatArg>) -> Result<ProcessedInput> {
    let source =
        Source::resolve(input).with_context(|| format!("failed to resolve input: {input}"))?;

    let format = match format_override {
        Some(f) => f.into(),
        None => format::detect(&source)
            .with_context(|| format!("could not detect format for: {input}"))?,
    };

    let markdown = extractors::extract(&source, format)
        .with_context(|| format!("extraction failed ({})", format))?;

    Ok(ProcessedInput {
        source: input.to_string(),
        format,
        markdown,
    })
}

fn write_markdown_output(documents: &[ProcessedInput]) {
    if let [document] = documents {
        print_markdown_body(&document.markdown);
        return;
    }

    for (idx, document) in documents.iter().enumerate() {
        if idx > 0 {
            println!();
        }

        println!("# Source: {}\n", markdown_heading_text(&document.source));
        print_markdown_body(&document.markdown);
    }
}

fn print_markdown_body(markdown: &str) {
    print!("{}", markdown);
    if !markdown.ends_with('\n') {
        println!();
    }
}

fn markdown_heading_text(source: &str) -> String {
    source.replace(['\r', '\n'], " ")
}

fn write_json_output(documents: &[ProcessedInput]) {
    if let [document] = documents {
        let obj = serde_json::json!({
            "mode": "json",
            "schema_version": "gist-json-v0",
            "status": "placeholder",
            "content": document.markdown,
            "format": document.format.to_string(),
            "source": document.source,
        });
        println!("{}", obj);
        return;
    }

    let items = documents
        .iter()
        .map(|document| {
            serde_json::json!({
                "content": document.markdown,
                "format": document.format.to_string(),
                "source": document.source,
            })
        })
        .collect::<Vec<_>>();

    let obj = serde_json::json!({
        "mode": "json",
        "schema_version": "gist-json-v0",
        "status": "placeholder",
        "items": items,
    });
    println!("{}", obj);
}

fn expand_inputs(inputs: &[String]) -> Result<Vec<String>> {
    let mut expanded = Vec::new();

    for input in inputs {
        if is_url_input(input) || !has_glob_meta(input) {
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

fn is_url_input(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

#[cfg(test)]
struct TestDir {
    path: PathBuf,
}

#[cfg(test)]
impl TestDir {
    fn new(name: &str) -> Result<Self> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("gist-{name}-{unique}"));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_without_flags_still_parses() {
        let cli = Cli::try_parse_from(["gist", "report.pdf"]).unwrap();

        assert_eq!(cli.inputs, ["report.pdf"]);
        assert_eq!(cli.mode, OutputMode::Md);
        assert!(cli.format.is_none());
    }

    #[test]
    fn multiple_inputs_parse() {
        let cli = Cli::try_parse_from(["gist", "a.pdf", "b.pdf"]).unwrap();

        assert_eq!(cli.inputs, ["a.pdf", "b.pdf"]);
    }

    #[test]
    fn glob_inputs_expand_and_sort() -> Result<()> {
        let dir = TestDir::new("glob_inputs_expand_and_sort")?;
        fs::write(dir.path().join("b.pdf"), b"")?;
        fs::write(dir.path().join("a.pdf"), b"")?;
        fs::write(dir.path().join("notes.txt"), b"")?;

        let pattern = dir.path().join("*.pdf").to_string_lossy().into_owned();
        let expanded = expand_inputs(&[pattern])?;
        let names = expanded
            .iter()
            .map(|path| {
                Path::new(path)
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
    fn help_uses_bilingual_headings_and_english_placeholders() {
        let err = Cli::try_parse_from(["gist", "-h"]).unwrap_err();

        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
        let help = err.to_string();
        assert!(help.contains("将文件或 URL 转换为 LLM-friendly Markdown"));
        assert!(help.contains("用法 (Usage):"));
        assert!(help.contains("gist [OPTIONS] <input>..."));
        assert!(help.contains("参数 (Arguments):"));
        assert!(help.contains("选项 (Options):"));
        assert!(help.contains("--format <format>"));
        assert!(help.contains("--mode <mode>"));
        assert!(help.contains("gist *.pdf"));
        assert!(help.contains("示例 (Examples):"));
        assert!(help.contains("gist report.pdf | llm \"Summarize risks and action items\""));
        assert!(help.contains("显示帮助。"));
        assert!(!help.contains("<输入>"));
        assert!(!help.contains("<格式>"));
        assert!(!help.contains("<模式>"));
        assert!(!help.contains("用法:"));
        assert!(!help.contains("选项:"));
    }
}
