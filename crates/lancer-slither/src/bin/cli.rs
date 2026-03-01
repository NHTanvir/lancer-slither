//! `lancer-slither-cli` — same as `lancer-slither --cli` but always CLI.

use clap::Parser;
use lancer_slither::cli::{run, Cli};

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let cli = Cli::parse();
    let code = run(&cli)?;
    std::process::exit(code);
}
