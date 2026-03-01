//! `lancer-slither-interface` — emit Solidity `interface` for a contract.

use std::path::PathBuf;

use clap::Args;
use slither_core::{FunctionKind, SlitherConfig, Visibility};

#[derive(Debug, Args)]
pub struct Args {
    pub target: PathBuf,
    #[arg(long)]
    pub contract: Option<String>,
}

pub fn run(args: &Args) -> eyre::Result<i32> {
    let cfg = SlitherConfig::default();
    let loaded = slither_core::project::load_project(&args.target, cfg)?;
    let slither = loaded.slither;
    for c in slither.all_contracts() {
        if let Some(ref name) = args.contract {
            if &c.name != name { continue; }
        }
        println!("interface I{} {{", c.name);
        for f in &c.functions {
            if !matches!(f.visibility, Visibility::Public | Visibility::External) { continue; }
            if matches!(f.kind, FunctionKind::Constructor | FunctionKind::Fallback | FunctionKind::Receive) { continue; }
            let params: Vec<String> = f.parameters.iter().map(|p| format!("{} {}", p.type_str, p.name)).collect();
            let rets: Vec<String> = f.returns.iter().map(|p| p.type_str.clone()).collect();
            let mut line = format!("    function {}({}) external", f.name, params.join(", "));
            if !rets.is_empty() {
                line.push_str(&format!(" returns ({})", rets.join(", ")));
            }
            line.push(';');
            println!("{line}");
        }
        println!("}}");
    }
    Ok(0)
}
