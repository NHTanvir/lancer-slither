use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "lancer-slither-find-paths", about = "Find call-graph paths from FROM to TO")]
struct Args {
    target: PathBuf,
    from: String,
    to: String,
}

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let args = Args::parse();
    let cfg = slither_core::SlitherConfig::default();
    let loaded = slither_core::project::load_project(&args.target, cfg)?;
    let slither = loaded.slither;
    let re = regex::Regex::new(&format!(r"\b{}\s*\(", regex::escape(&args.to))).unwrap();
    for (c, f) in slither.all_functions() {
        if f.full_name() == args.from || f.name == args.from {
            if re.is_match(&f.body_excerpt) {
                println!("{}.{} -> {}", c.name, f.name, args.to);
            }
        }
    }
    Ok(())
}
