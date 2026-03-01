//! `lancer-slither-check-upgradeability` — diff storage layouts of V1 vs V2.

use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct Args {
    #[arg(long)]
    pub old: PathBuf,
    #[arg(long)]
    pub new: PathBuf,
    /// Contract name to compare (defaults to the first storage-layout in each).
    #[arg(long)]
    pub contract: Option<String>,
}

pub fn run(args: &Args) -> eyre::Result<i32> {
    let cfg = slither_core::SlitherConfig::default();
    let old = slither_core::project::load_project(&args.old, cfg.clone())?;
    let new = slither_core::project::load_project(&args.new, cfg)?;
    let old_layout = find_layout(&old.slither, args.contract.as_deref());
    let new_layout = find_layout(&new.slither, args.contract.as_deref());
    let (Some(old), Some(new)) = (old_layout, new_layout) else {
        eprintln!("storage layout missing in one of the projects (need Foundry/Hardhat artifacts).");
        return Ok(1);
    };
    let diff = diff_layouts(&old, &new);
    let bad = !diff.is_empty();
    for line in diff {
        println!("{}", line);
    }
    Ok(if bad { 1 } else { 0 })
}

fn find_layout(
    slither: &slither_core::Slither,
    target: Option<&str>,
) -> Option<serde_json::Value> {
    for su in &slither.source_units {
        if let Some(layout) = &su.storage_layout {
            if let Some(name) = target {
                if let Some(arr) = layout.get("storage").and_then(serde_json::Value::as_array) {
                    if arr.iter().any(|x| x.get("contract").and_then(serde_json::Value::as_str) == Some(name)) {
                        return Some(layout.clone());
                    }
                }
            }
            return Some(layout.clone());
        }
    }
    None
}

fn diff_layouts(old: &serde_json::Value, new: &serde_json::Value) -> Vec<String> {
    let mut diffs = Vec::new();
    let empty: Vec<serde_json::Value> = Vec::new();
    let o = old.get("storage").and_then(serde_json::Value::as_array).unwrap_or(&empty);
    let n = new.get("storage").and_then(serde_json::Value::as_array).unwrap_or(&empty);
    for i in 0..o.len().max(n.len()) {
        match (o.get(i), n.get(i)) {
            (Some(a), Some(b)) => {
                if a != b {
                    diffs.push(format!("slot {i}: layout changed:\n  old: {a}\n  new: {b}"));
                }
            }
            (Some(a), None) => diffs.push(format!("slot {i}: removed: {a}")),
            (None, Some(b)) => diffs.push(format!("slot {i}: added: {b}")),
            (None, None) => {}
        }
    }
    diffs
}
