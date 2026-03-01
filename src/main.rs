//! lancer-slither
//! Rust clone of slither. See plan.md (local, gitignored) for the spec.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "lancer-slither", version, about = "Rust clone of slither")]
struct Cli {}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let _cli = Cli::parse();
    println!("lancer-slither - scaffold ready. Implement per plan.md.");
    Ok(())
}
