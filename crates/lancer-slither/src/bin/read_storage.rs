use clap::Parser;
use lancer_slither::read_storage::{run, Args};

#[derive(Parser)]
#[command(name = "lancer-slither-read-storage", about = "Read storage from a live contract via RPC")]
struct Outer { #[command(flatten)] inner: Args }

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let Outer { inner } = Outer::parse();
    let code = run(&inner)?;
    std::process::exit(code);
}
