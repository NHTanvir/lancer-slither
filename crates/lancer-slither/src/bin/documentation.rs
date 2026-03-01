use std::path::PathBuf;

use clap::Parser;
use slither_core::{FunctionKind, SlitherConfig};

#[derive(Parser)]
#[command(name = "lancer-slither-documentation", about = "Generate NatSpec scaffolds for undocumented functions")]
struct Args {
    target: PathBuf,
    #[arg(long)]
    apply: bool,
    #[arg(long)]
    contract: Option<String>,
}

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let args = Args::parse();
    let cfg = SlitherConfig::default();
    let loaded = slither_core::project::load_project(&args.target, cfg)?;
    let slither = loaded.slither;
    for (c, f) in slither.all_functions() {
        if let Some(ref n) = args.contract {
            if &c.name != n { continue; }
        }
        if matches!(f.kind, FunctionKind::Constructor | FunctionKind::Fallback | FunctionKind::Receive) {
            continue;
        }
        println!("/// @notice TODO: describe {}.{}", c.name, f.name);
        for p in &f.parameters {
            println!("/// @param {} TODO", p.name);
        }
        for r in &f.returns {
            println!("/// @return {} TODO", if r.name.is_empty() { "ret" } else { &r.name });
        }
    }
    let _ = args.apply;
    Ok(())
}
