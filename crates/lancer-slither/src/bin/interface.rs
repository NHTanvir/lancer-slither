use clap::Parser;
use lancer_slither::interface_gen::{run, Args};

#[derive(Parser)]
#[command(name = "lancer-slither-interface", about = "Generate Solidity interface declaration for a contract")]
struct Outer { #[command(flatten)] inner: Args }

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let Outer { inner } = Outer::parse();
    let code = run(&inner)?;
    std::process::exit(code);
}
