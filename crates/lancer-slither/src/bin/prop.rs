use clap::Parser;
use lancer_slither::prop::{run, Args};

#[derive(Parser)]
#[command(name = "lancer-slither-prop", about = "Echidna / Medusa / Halmos property scaffolds")]
struct Outer { #[command(flatten)] inner: Args }

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let Outer { inner } = Outer::parse();
    let code = run(&inner)?;
    std::process::exit(code);
}
