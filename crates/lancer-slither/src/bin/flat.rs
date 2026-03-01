use clap::Parser;
use lancer_slither::flatten::{run, Args};

#[derive(Parser)]
#[command(name = "lancer-slither-flat", about = "Flatten a multi-file Solidity project into one .sol")]
struct Outer { #[command(flatten)] inner: Args }

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let Outer { inner } = Outer::parse();
    let code = run(&inner)?;
    std::process::exit(code);
}
