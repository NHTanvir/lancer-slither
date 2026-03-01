//! `lancer-slither-check-erc` — ERC conformance checker.

use std::path::PathBuf;

use clap::Args;
use slither_core::SlitherConfig;

#[derive(Debug, Args)]
pub struct Args {
    pub target: PathBuf,
    /// One of `erc20`, `erc721`, `erc777`, `erc1155`, `erc4626`, `erc7683`.
    #[arg(long, default_value = "erc20")]
    pub standard: String,
    /// Contract name to check.
    #[arg(long)]
    pub contract: Option<String>,
}

pub fn run(args: &Args) -> eyre::Result<i32> {
    let cfg = SlitherConfig::default();
    let loaded = slither_core::project::load_project(&args.target, cfg)?;
    let slither = loaded.slither;
    let std_required = required_functions_for(&args.standard);
    let mut failures = 0;
    for c in slither.all_contracts() {
        if let Some(ref name) = args.contract {
            if &c.name != name { continue; }
        }
        let mut missing = Vec::new();
        for (fn_name, arity) in &std_required {
            let f = c.functions.iter().find(|f| &f.name == fn_name);
            match f {
                None => missing.push(*fn_name),
                Some(f) => {
                    if f.parameters.len() != *arity {
                        failures += 1;
                        println!("[{}] {}: wrong arity ({} vs {})", args.standard, fn_name, f.parameters.len(), arity);
                    }
                }
            }
        }
        if !missing.is_empty() {
            failures += missing.len();
            println!("[{}] missing in {}: {:?}", args.standard, c.name, missing);
        }
    }
    if failures == 0 {
        println!("ERC {} conformance: OK", args.standard);
    }
    Ok(if failures == 0 { 0 } else { 1 })
}

fn required_functions_for(standard: &str) -> Vec<(&'static str, usize)> {
    match standard {
        "erc20" => vec![
            ("totalSupply", 0), ("balanceOf", 1), ("transfer", 2),
            ("transferFrom", 3), ("approve", 2), ("allowance", 2),
        ],
        "erc721" => vec![
            ("balanceOf", 1), ("ownerOf", 1), ("safeTransferFrom", 3),
            ("transferFrom", 3), ("approve", 2), ("setApprovalForAll", 2),
            ("getApproved", 1), ("isApprovedForAll", 2),
        ],
        "erc777" => vec![
            ("name", 0), ("symbol", 0), ("totalSupply", 0), ("balanceOf", 1),
            ("granularity", 0), ("send", 3), ("burn", 2),
        ],
        "erc1155" => vec![
            ("safeTransferFrom", 5), ("safeBatchTransferFrom", 5),
            ("balanceOf", 2), ("balanceOfBatch", 2),
            ("setApprovalForAll", 2), ("isApprovedForAll", 2),
        ],
        "erc4626" => vec![
            ("asset", 0), ("totalAssets", 0), ("convertToShares", 1),
            ("convertToAssets", 1), ("maxDeposit", 1), ("previewDeposit", 1),
            ("deposit", 2), ("maxMint", 1), ("previewMint", 1), ("mint", 2),
            ("maxWithdraw", 1), ("previewWithdraw", 1), ("withdraw", 3),
            ("maxRedeem", 1), ("previewRedeem", 1), ("redeem", 3),
        ],
        "erc7683" => vec![
            ("open", 1), ("openFor", 3), ("resolve", 1), ("resolveFor", 2),
        ],
        _ => Vec::new(),
    }
}
