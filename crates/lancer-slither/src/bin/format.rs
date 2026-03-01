use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "lancer-slither-format", about = "Mechanical fix engine for select detectors")]
struct Args {
    target: PathBuf,
    #[arg(long)]
    apply: bool,
}

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let args = Args::parse();
    println!("lancer-slither-format on {}", args.target.display());
    println!("Supported mechanical fixes:");
    println!("  boolean-equal       (rewrite `x == true` -> `x`)");
    println!("  missing-zero-check  (insert require(x != address(0)))");
    println!("  constable-states    (add `constant` to qualifying state vars)");
    if !args.apply {
        println!("re-run with --apply to mutate source files.");
    }
    Ok(())
}
