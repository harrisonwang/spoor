mod app;
mod cli;
mod source;

use clap::Parser;
use std::io::Write;

fn main() {
    if let Err(e) = run() {
        if let Some(error) = e.downcast_ref::<spoor_core::SpoorError>() {
            eprintln!("{}", error.to_json());
        } else {
            eprintln!("error: {:#}", e);
        }
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let output = app::run(cli::Cli::parse())?;
    match output {
        app::CommandOutput::Text(text) => print!("{text}"),
        app::CommandOutput::Binary(bytes) => std::io::stdout().lock().write_all(&bytes)?,
    }
    Ok(())
}
