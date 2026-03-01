//! `lancer-slither-read-storage` — read storage slots via RPC.

use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct Args {
    /// On-chain address.
    pub address: String,
    #[arg(long, env = "RPC_URL", default_value = "https://eth.llamarpc.com")]
    pub rpc: String,
    /// Block number to read at (default: latest).
    #[arg(long)]
    pub block: Option<u64>,
    /// Slot expression: `<storage variable>[key]`.
    #[arg(long)]
    pub slot: Option<String>,
    /// Project for layout (Foundry preferred).
    #[arg(long)]
    pub project: Option<PathBuf>,
}

pub fn run(args: &Args) -> eyre::Result<i32> {
    let slot = args
        .slot
        .clone()
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000000000000000000000000000".to_owned());
    let block = args
        .block
        .map(|b| format!("0x{b:x}"))
        .unwrap_or_else(|| "latest".to_owned());
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_getStorageAt",
        "params": [args.address, slot, block],
    });
    let resp: serde_json::Value = reqwest::blocking::Client::new()
        .post(&args.rpc)
        .json(&body)
        .send()?
        .json()?;
    if let Some(val) = resp.get("result").and_then(serde_json::Value::as_str) {
        println!("{}", val);
    } else {
        eprintln!("rpc error: {resp}");
        return Ok(1);
    }
    Ok(0)
}
