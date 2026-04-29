use anyhow::{Context, Result};
use clap::Parser;
use std::fmt;

use gist::extractors;
use gist::format::{self, FormatArg};
use gist::source::Source;

/// Convert files and URLs to LLM-friendly markdown.
#[derive(Parser, Debug)]
#[command(name = "gist", version, about, long_about = None)]
struct Cli {
    /// Path to a file or a URL.
    input: String,

    /// Override format detection.
    #[arg(long, value_enum)]
    format: Option<FormatArg>,

    /// Output mode. `llm` is the default; `json` is a flat placeholder schema for now.
    #[arg(long, value_enum, default_value_t = OutputMode::Llm)]
    mode: OutputMode,

    /// Deprecated alias for `--mode json`.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum OutputMode {
    Llm,
    Json,
}

impl fmt::Display for OutputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            OutputMode::Llm => "llm",
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

    let source = Source::resolve(&cli.input)
        .with_context(|| format!("failed to resolve input: {}", cli.input))?;

    let format = match cli.format {
        Some(f) => f.into(),
        None => format::detect(&source)
            .with_context(|| format!("could not detect format for: {}", cli.input))?,
    };

    let markdown = extractors::extract(&source, format)
        .with_context(|| format!("extraction failed ({})", format))?;

    let mode = if cli.json { OutputMode::Json } else { cli.mode };
    match mode {
        OutputMode::Llm => {
            print!("{}", markdown);
            if !markdown.ends_with('\n') {
                println!();
            }
        }
        OutputMode::Json => {
            let obj = serde_json::json!({
                "mode": "json",
                "schema_version": "gist-json-v0",
                "status": "placeholder",
                "content": markdown,
                "format": format.to_string(),
                "source": cli.input,
            });
            println!("{}", obj);
        }
    }
    Ok(())
}
