use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

mod format;
mod output;
mod source;
mod extractors;

use format::Format;
use source::Source;

/// Convert files and URLs to LLM-friendly markdown.
#[derive(Parser, Debug)]
#[command(name = "gist", version, about, long_about = None)]
struct Cli {
    /// Path to a file or a URL.
    input: String,

    /// Override format detection.
    #[arg(long, value_enum)]
    format: Option<Format>,

    /// Output as JSON ({"content": "...", "format": "...", "source": "..."}).
    #[arg(long)]
    json: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // 1. Resolve input → Source (file bytes or URL)
    let source = Source::resolve(&cli.input)
        .with_context(|| format!("failed to resolve input: {}", cli.input))?;

    // 2. Detect format (CLI override > magic bytes > extension > URL content-type)
    let format = match cli.format {
        Some(f) => f,
        None => format::detect(&source)
            .with_context(|| format!("could not detect format for: {}", cli.input))?,
    };

    // 3. Dispatch to extractor
    let markdown = extractors::extract(&source, format)
        .with_context(|| format!("extraction failed ({})", format))?;

    // 4. Emit
    if cli.json {
        let obj = serde_json::json!({
            "content": markdown,
            "format": format.to_string(),
            "source": cli.input,
        });
        println!("{}", obj);
    } else {
        print!("{}", markdown);
        if !markdown.ends_with('\n') {
            println!();
        }
    }
    Ok(())
}
