use clap::{ArgAction, Parser, ValueEnum};
use spoor_core::{Format, OutputMode, ProvenanceLevel};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum FormatArg {
    Html,
    Markdown,
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Csv,
    Ipynb,
    Epub,
    Text,
}

impl From<FormatArg> for Format {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Html => Format::Html,
            FormatArg::Markdown => Format::Markdown,
            FormatArg::Pdf => Format::Pdf,
            FormatArg::Docx => Format::Docx,
            FormatArg::Xlsx => Format::Xlsx,
            FormatArg::Pptx => Format::Pptx,
            FormatArg::Csv => Format::Csv,
            FormatArg::Ipynb => Format::Ipynb,
            FormatArg::Epub => Format::Epub,
            FormatArg::Text => Format::PlainText,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ModeArg {
    Md,
    Json,
}

impl From<ModeArg> for OutputMode {
    fn from(value: ModeArg) -> Self {
        match value {
            ModeArg::Md => OutputMode::Md,
            ModeArg::Json => OutputMode::Json,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ProvenanceArg {
    /// One mapping per source page (PDF page-level).
    Page,
}

impl From<ProvenanceArg> for ProvenanceLevel {
    fn from(value: ProvenanceArg) -> Self {
        match value {
            ProvenanceArg::Page => ProvenanceLevel::Page,
        }
    }
}

const HELP_TEMPLATE: &str = "\
{about}

Usage:
  {usage}

Arguments:
{positionals}

Options:
{options}

For tables (CSV/XLSX), the recommended pattern is:

  spoor file.xlsx                              # see structure + preview
  spoor file.xlsx --sheet L1 --rows 5:104      # read a slice
  spoor file.xlsx --columns 分类,技能          # filter columns

For PDFs, use --pages first when an agent only needs a slice:

  spoor report.pdf --pages 1:3              # read selected PDF pages

spoor bounds JSON previews by default (first 100 data rows per table) and
caps total CLI output at 256 KiB. Use --limit/--rows to narrow tables or
--max-output-bytes to raise the total output cap. Parsing uses a shared
64 MiB data-volume budget by default; raise it with --max-parse-bytes.

Examples:
  spoor report.pdf
  spoor report.pdf --pages 1:3
  spoor data.xlsx
  spoor data.csv | jq '.tables[]'
  cat data.csv | spoor --format csv -
  spoor https://example.com/article
  spoor \"*.pdf\"
  spoor document.docx --extract spoor-docx://word/media/image1.png > image.png
  spoor report.pdf | llm \"Summarize risks and action items\"
";

#[derive(Parser, Debug)]
#[command(
    name = "spoor",
    version,
    about = "离线、单二进制、CLI-first 的 LLM-friendly 文档预处理工具",
    long_about = None,
    override_usage = "spoor [OPTIONS] <input>...",
    help_template = HELP_TEMPLATE,
    disable_help_flag = true,
    disable_version_flag = true
)]
pub(crate) struct Cli {
    /// 文件路径、URL、本地 glob，或 - 表示标准输入；可传多个，URL 与 - 不参与 glob 展开。
    #[arg(value_name = "input", required = true, num_args = 1..)]
    pub(crate) inputs: Vec<String>,

    /// 覆盖自动 format 检测（默认按 magic-byte / 扩展名推断）。
    #[arg(long, value_enum, value_name = "format")]
    pub(crate) format: Option<FormatArg>,

    /// 覆盖默认输出模式；表格型（CSV/XLSX）默认 json，其他默认 md。
    #[arg(
        long,
        short = 'm',
        value_enum,
        value_name = "mode",
        hide_possible_values = true,
        hide_default_value = true
    )]
    pub(crate) mode: Option<ModeArg>,

    /// PDF 限定页码区间，例如 `1:3`（含两端）；用于按需读取大型 PDF。
    #[arg(long, value_name = "first:last")]
    pub(crate) pages: Option<String>,

    /// XLSX 限定 sheet；找不到时报错并列出可用 sheets。CSV 无此概念，自动忽略。
    #[arg(long, value_name = "name")]
    pub(crate) sheet: Option<String>,

    /// 限定数据行的 Excel 行号区间，例如 `5:104`（含两端）。与 --limit/--offset 互斥。
    #[arg(long, value_name = "first:last", conflicts_with_all = ["limit", "offset"])]
    pub(crate) rows: Option<String>,

    /// 按列名筛选，逗号分隔；找不到时报错并列出可用列。
    #[arg(long, value_name = "columns", value_delimiter = ',')]
    pub(crate) columns: Vec<String>,

    /// 每个 table 最多返回多少数据行；默认 100。
    #[arg(long, value_name = "n")]
    pub(crate) limit: Option<usize>,

    /// 跳过前 N 条数据行再应用 --limit；默认 0。
    #[arg(long, value_name = "n")]
    pub(crate) offset: Option<usize>,

    /// 整次命令 stdout 的最大字节数；默认 262144（256 KiB）。
    #[arg(
        long,
        value_name = "n",
        default_value_t = spoor_core::DEFAULT_MAX_OUTPUT_BYTES
    )]
    pub(crate) max_output_bytes: usize,

    /// 解析输入、中间文本和容器解压内容的共享字节预算；默认 67108864（64 MiB）。
    #[arg(
        long,
        value_name = "n",
        default_value_t = spoor_core::DEFAULT_MAX_PARSE_BYTES
    )]
    pub(crate) max_parse_bytes: usize,

    /// 解析工作量上限（如 PDF 操作数），用于约束字节预算管不到的 CPU；默认不限。
    /// 不可信输入还应配合宿主级超时与进程/容器隔离才能真正掐断。
    #[arg(long, value_name = "n")]
    pub(crate) max_work_units: Option<usize>,

    /// 返回"输出 → 原文"来源定位；当前支持 page（PDF 页级）。默认不产出。
    /// 输出为 JSON（含 markdown 与 provenance），仅支持单个文档型输入。
    #[arg(long, value_enum, value_name = "level", conflicts_with = "mode")]
    pub(crate) provenance: Option<ProvenanceArg>,

    /// 将 spoor 输出的单个内嵌媒体资源原样输出到 stdout；当前支持 spoor-docx://。
    #[arg(
        long,
        value_name = "uri",
        conflicts_with_all = [
            "format",
            "mode",
            "pages",
            "sheet",
            "rows",
            "columns",
            "limit",
            "offset",
            "max_output_bytes",
            "provenance"
        ]
    )]
    pub(crate) extract: Option<String>,

    /// 显示帮助。
    #[arg(short = 'h', long = "help", action = ArgAction::Help)]
    help: Option<bool>,

    /// 显示版本。
    #[arg(short = 'V', long = "version", action = ArgAction::Version)]
    version: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_without_flags_still_parses() {
        let cli = Cli::try_parse_from(["spoor", "report.pdf"]).unwrap();

        assert_eq!(cli.inputs, ["report.pdf"]);
        assert!(cli.mode.is_none());
        assert!(cli.format.is_none());
        assert!(cli.sheet.is_none());
        assert!(cli.pages.is_none());
        assert!(cli.rows.is_none());
        assert!(cli.columns.is_empty());
        assert!(cli.limit.is_none());
        assert!(cli.offset.is_none());
        assert_eq!(cli.max_output_bytes, spoor_core::DEFAULT_MAX_OUTPUT_BYTES);
        assert_eq!(cli.max_parse_bytes, spoor_core::DEFAULT_MAX_PARSE_BYTES);
        assert!(cli.extract.is_none());
    }

    #[test]
    fn multiple_inputs_parse() {
        let cli = Cli::try_parse_from(["spoor", "a.pdf", "b.pdf"]).unwrap();

        assert_eq!(cli.inputs, ["a.pdf", "b.pdf"]);
    }

    #[test]
    fn explicit_mode_parses() {
        let cli = Cli::try_parse_from(["spoor", "-m", "json", "data.csv"]).unwrap();
        assert_eq!(cli.mode, Some(ModeArg::Json));

        let cli = Cli::try_parse_from(["spoor", "-m", "md", "data.xlsx"]).unwrap();
        assert_eq!(cli.mode, Some(ModeArg::Md));
    }

    #[test]
    fn narrowing_flags_parse() {
        let cli = Cli::try_parse_from([
            "spoor",
            "data.xlsx",
            "--sheet",
            "L1",
            "--pages",
            "1:3",
            "--rows",
            "5:104",
            "--columns",
            "分类,技能",
        ])
        .unwrap();

        assert_eq!(cli.sheet, Some("L1".to_string()));
        assert_eq!(cli.pages, Some("1:3".to_string()));
        assert_eq!(cli.rows, Some("5:104".to_string()));
        assert_eq!(cli.columns, vec!["分类".to_string(), "技能".to_string()]);
    }

    #[test]
    fn rows_conflicts_with_limit() {
        let err = Cli::try_parse_from(["spoor", "data.xlsx", "--rows", "5:104", "--limit", "10"])
            .unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn rows_conflicts_with_offset() {
        let err = Cli::try_parse_from(["spoor", "data.xlsx", "--rows", "5:104", "--offset", "2"])
            .unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn extract_parses_and_conflicts_with_text_output_options() {
        let cli = Cli::try_parse_from([
            "spoor",
            "document.docx",
            "--extract",
            "spoor-docx://word/media/image1.png",
        ])
        .unwrap();
        assert_eq!(
            cli.extract.as_deref(),
            Some("spoor-docx://word/media/image1.png")
        );

        let err = Cli::try_parse_from([
            "spoor",
            "document.docx",
            "--extract",
            "spoor-docx://word/media/image1.png",
            "--mode",
            "md",
        ])
        .unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn help_uses_bilingual_headings_and_english_placeholders() {
        let err = Cli::try_parse_from(["spoor", "-h"]).unwrap_err();

        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
        let help = err.to_string();
        assert!(help.contains("离线、单二进制、CLI-first 的 LLM-friendly 文档预处理工具"));
        assert!(help.contains("Usage:"));
        assert!(help.contains("spoor [OPTIONS] <input>..."));
        assert!(help.contains("Arguments:"));
        assert!(help.contains("Options:"));
        assert!(help.contains("--format <format>"));
        assert!(help.contains("--mode <mode>"));
        assert!(help.contains("--sheet <name>"));
        assert!(help.contains("--pages <first:last>"));
        assert!(help.contains("--rows <first:last>"));
        assert!(help.contains("--columns <columns>"));
        assert!(help.contains("--limit <n>"));
        assert!(help.contains("--offset <n>"));
        assert!(help.contains("--max-output-bytes <n>"));
        assert!(help.contains("--max-parse-bytes <n>"));
        assert!(help.contains("--extract <uri>"));
        assert!(help.contains("spoor \"*.pdf\""));
        assert!(help.contains("Examples:"));
        assert!(help.contains("spoor report.pdf --pages 1:3"));
        assert!(help.contains("spoor report.pdf | llm \"Summarize risks and action items\""));
        assert!(help.contains("--sheet L1 --rows 5:104"));
        assert!(help.contains("caps total CLI output at 256 KiB"));
        assert!(help.contains("显示帮助。"));
        assert!(!help.contains("<输入>"));
        assert!(!help.contains("<格式>"));
        assert!(!help.contains("<模式>"));
        assert!(!help.contains("用法:"));
        assert!(!help.contains("选项:"));
    }

    /// The JSON envelope's `usage` hint must stay in sync with the real CLI
    /// flags. We derive the narrowing flags from the clap definition (every
    /// long flag except the non-narrowing ones) and require each to appear in
    /// `TABLE_USAGE`, so renaming or adding a narrowing flag without updating
    /// the hint fails CI instead of silently lying to consumers.
    #[test]
    fn table_usage_lists_every_narrowing_flag() {
        use clap::CommandFactory;

        let not_narrowing = [
            "format",
            "mode",
            "pages",
            "max-output-bytes",
            "max-parse-bytes",
            "max-work-units",
            "provenance",
            "extract",
            "help",
            "version",
        ];

        for arg in Cli::command().get_arguments() {
            let Some(long) = arg.get_long() else {
                continue;
            };
            if not_narrowing.contains(&long) {
                continue;
            }
            assert!(
                spoor_core::TABLE_USAGE.contains(&format!("--{long}")),
                "narrowing flag --{long} is missing from TABLE_USAGE; the JSON \
                 `usage` hint would no longer match the real CLI"
            );
        }
    }
}
