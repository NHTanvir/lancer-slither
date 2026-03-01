use clap::Parser;
use lancer_slither::mutate::{run, Args};

#[derive(Parser)]
#[command(name = "lancer-slither-mutate", about = "Operator-mutation testing for Solidity")]
struct Outer { #[command(flatten)] inner: Args }

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let Outer { inner } = Outer::parse();
    let code = run(&inner)?;
    std::process::exit(code);
}
