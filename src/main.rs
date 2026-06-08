mod app;
mod cli;

use clap::Parser;

fn main() {
    if let Err(e) = run() {
        if let Some(error) = e.downcast_ref::<pith::StructuredError>() {
            eprintln!("{}", error.to_json());
        } else {
            eprintln!("error: {:#}", e);
        }
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let output = app::run(cli::Cli::parse())?;
    print!("{output}");
    Ok(())
}
