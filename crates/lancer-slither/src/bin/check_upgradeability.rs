use clap::Parser;
use lancer_slither::upgradeability::{run, Args};

#[derive(Parser)]
#[command(name = "lancer-slither-check-upgradeability", about = "Storage layout differ for upgradeable proxies")]
struct Outer { #[command(flatten)] inner: Args }

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let Outer { inner } = Outer::parse();
    let code = run(&inner)?;
    std::process::exit(code);
}
