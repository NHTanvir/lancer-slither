//! lancer-slither — main entry point.
//!
//! No arguments → eframe GUI. Any argument → CLI surface.

use clap::Parser;
use lancer_slither::cli::{run as run_cli, Cli};

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() == 1 {
        return lancer_slither::gui::launch();
    }
    let cli = Cli::parse();
    let code = run_cli(&cli)?;
    std::process::exit(code);
}
