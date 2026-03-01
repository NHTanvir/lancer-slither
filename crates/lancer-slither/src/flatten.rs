//! `lancer-slither-flat` — flatten a Solidity project into one file.

use std::path::PathBuf;

use clap::Args;
use slither_core::SlitherConfig;

#[derive(Debug, Args)]
pub struct Args {
    pub target: PathBuf,
    #[arg(long, default_value = "flat.sol")]
    pub out: PathBuf,
    /// Strip license comments + duplicated pragmas.
    #[arg(long)]
    pub strip_license: bool,
}

pub fn run(args: &Args) -> eyre::Result<i32> {
    let cfg = SlitherConfig::default();
    let loaded = slither_core::project::load_project(&args.target, cfg)?;
    let slither = loaded.slither;
    let mut out = String::new();
    let mut emitted_pragma = false;
    for su in &slither.source_units {
        let mut content = su.content.clone();
        if let Some(p) = &su.pragma {
            if emitted_pragma {
                let needle = format!("pragma solidity {};", p);
                content = content.replace(&needle, "");
            } else {
                emitted_pragma = true;
            }
        }
        if args.strip_license {
            content = content.lines().filter(|l| !l.contains("SPDX-License-Identifier")).collect::<Vec<_>>().join("\n");
        }
        out.push_str(&format!("// --- {} ---\n", su.relative_path.display()));
        out.push_str(&content);
        out.push_str("\n");
    }
    std::fs::write(&args.out, out)?;
    println!("flattened to {}", args.out.display());
    Ok(0)
}
