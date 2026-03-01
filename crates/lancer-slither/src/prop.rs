//! `lancer-slither-prop` — Echidna / Medusa / Halmos property scaffolding.

use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct Args {
    pub target: PathBuf,
    /// Which framework: `echidna`, `medusa`, `halmos`.
    #[arg(long, default_value = "echidna")]
    pub framework: String,
    /// Contract name (defaults to first).
    #[arg(long)]
    pub contract: Option<String>,
}

pub fn run(args: &Args) -> eyre::Result<i32> {
    let cfg = slither_core::SlitherConfig::default();
    let loaded = slither_core::project::load_project(&args.target, cfg)?;
    let slither = loaded.slither;
    let target = args.contract.clone().unwrap_or_else(|| {
        slither.all_contracts().next().map(|c| c.name.clone()).unwrap_or_default()
    });
    match args.framework.as_str() {
        "echidna" => emit_echidna(&target),
        "medusa" => emit_medusa(&target),
        "halmos" => emit_halmos(&target),
        _ => eyre::bail!("unknown framework: {}", args.framework),
    }
    Ok(0)
}

fn emit_echidna(target: &str) {
    println!("// crytic/echidna scaffold for {target}");
    println!("contract Echidna_{target} is {target} {{");
    println!("    function echidna_supply_nonneg() public view returns (bool) {{ return totalSupply() >= 0; }}");
    println!("    function echidna_balance_nonneg() public view returns (bool) {{ return balanceOf(address(this)) >= 0; }}");
    println!("}}");
}

fn emit_medusa(target: &str) {
    println!("// crytic/medusa scaffold for {target}");
    println!("contract Medusa_{target} is {target} {{");
    println!("    function property_balance_nonneg() public view returns (bool) {{ return balanceOf(address(this)) >= 0; }}");
    println!("}}");
}

fn emit_halmos(target: &str) {
    println!("// halmos scaffold for {target}");
    println!("contract Check_{target} is {target} {{");
    println!("    function check_transfer(uint256 amount) public {{");
    println!("        uint256 before = balanceOf(address(this));");
    println!("        transfer(msg.sender, amount);");
    println!("        assert(balanceOf(address(this)) <= before);");
    println!("    }}");
    println!("}}");
}
