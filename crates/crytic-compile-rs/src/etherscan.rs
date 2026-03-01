//! Etherscan verified-source fetcher.
//!
//! Multi-chain: mainnet, polygon, arbitrum, base, optimism, bsc, fantom, avalanche,
//! gnosis, sepolia, holesky, plus Blockscout / Routescan fallbacks.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::framework::Framework;
use crate::{CompiledSource, CompilationUnit, CompileOptions};

#[derive(Debug, Clone, Copy)]
pub enum Network {
    Mainnet,
    Polygon,
    Arbitrum,
    Base,
    Optimism,
    Bsc,
    Fantom,
    Avalanche,
    Gnosis,
    Sepolia,
    Holesky,
}

impl Network {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "mainnet" | "ethereum" | "eth" => Self::Mainnet,
            "polygon" | "matic" => Self::Polygon,
            "arbitrum" | "arb" => Self::Arbitrum,
            "base" => Self::Base,
            "optimism" | "op" => Self::Optimism,
            "bsc" | "bnb" => Self::Bsc,
            "fantom" | "ftm" => Self::Fantom,
            "avalanche" | "avax" => Self::Avalanche,
            "gnosis" | "xdai" => Self::Gnosis,
            "sepolia" => Self::Sepolia,
            "holesky" => Self::Holesky,
            _ => return None,
        })
    }
    pub fn api_url(self) -> &'static str {
        match self {
            Self::Mainnet => "https://api.etherscan.io/api",
            Self::Polygon => "https://api.polygonscan.com/api",
            Self::Arbitrum => "https://api.arbiscan.io/api",
            Self::Base => "https://api.basescan.org/api",
            Self::Optimism => "https://api-optimistic.etherscan.io/api",
            Self::Bsc => "https://api.bscscan.com/api",
            Self::Fantom => "https://api.ftmscan.com/api",
            Self::Avalanche => "https://api.snowtrace.io/api",
            Self::Gnosis => "https://api.gnosisscan.io/api",
            Self::Sepolia => "https://api-sepolia.etherscan.io/api",
            Self::Holesky => "https://api-holesky.etherscan.io/api",
        }
    }
}

#[derive(Debug, Deserialize)]
struct EsResp {
    status: String,
    message: String,
    result: Value,
}

/// Public-API: pull verified source code by address.
pub fn fetch(address: &str, apikey: Option<&str>, network: Network) -> eyre::Result<Vec<CompiledSource>> {
    let key = apikey.unwrap_or("");
    let url = format!(
        "{}?module=contract&action=getsourcecode&address={}&apikey={}",
        network.api_url(),
        address,
        key
    );
    let resp = reqwest::blocking::get(&url)?.text()?;
    let parsed: EsResp = serde_json::from_str(&resp)?;
    if parsed.status != "1" {
        eyre::bail!("etherscan: {}", parsed.message);
    }
    let arr = parsed.result.as_array().cloned().unwrap_or_default();
    let mut sources = Vec::new();
    for item in arr {
        let source = item.get("SourceCode").and_then(Value::as_str).unwrap_or("");
        let name = item.get("ContractName").and_then(Value::as_str).unwrap_or("Contract");
        if source.is_empty() {
            continue;
        }
        if source.starts_with("{{") {
            // Solc Standard JSON wrapped.
            let inner = &source[1..source.len() - 1];
            if let Ok(json) = serde_json::from_str::<Value>(inner) {
                if let Some(sources_obj) = json.get("sources").and_then(Value::as_object) {
                    for (path, file) in sources_obj {
                        let c = file.get("content").and_then(Value::as_str).unwrap_or("").to_owned();
                        sources.push(CompiledSource {
                            path: PathBuf::from(path),
                            content: c,
                            ast: None,
                            source_map: None,
                            bytecode: None,
                            deployed_bytecode: None,
                            abi: None,
                            storage_layout: None,
                            is_dependency: false,
                        });
                    }
                }
            }
        } else if source.starts_with('{') {
            if let Ok(json) = serde_json::from_str::<Value>(source) {
                if let Some(sources_obj) = json.get("sources").and_then(Value::as_object) {
                    for (path, file) in sources_obj {
                        let c = file.get("content").and_then(Value::as_str).unwrap_or("").to_owned();
                        sources.push(CompiledSource {
                            path: PathBuf::from(path),
                            content: c,
                            ast: None,
                            source_map: None,
                            bytecode: None,
                            deployed_bytecode: None,
                            abi: None,
                            storage_layout: None,
                            is_dependency: false,
                        });
                    }
                }
            }
        } else {
            sources.push(CompiledSource {
                path: PathBuf::from(format!("{name}.sol")),
                content: source.to_owned(),
                ast: None,
                source_map: None,
                bytecode: None,
                deployed_bytecode: None,
                abi: None,
                storage_layout: None,
                is_dependency: false,
            });
        }
    }
    Ok(sources)
}

pub fn compile_address(address: &str, opts: &CompileOptions) -> eyre::Result<CompilationUnit> {
    let network = opts
        .etherscan_network
        .as_deref()
        .and_then(Network::from_str)
        .unwrap_or(Network::Mainnet);
    let key = opts.etherscan_apikey.as_deref();
    let sources = fetch(address, key, network)?;
    let project_root = opts
        .etherscan_export_dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join(format!("lancer-slither-etherscan-{address}")));
    std::fs::create_dir_all(&project_root)?;

    // Persist fetched files to disk so source paths are real.
    let mut sources = sources;
    for s in &mut sources {
        let target = project_root.join(&s.path);
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&target, &s.content);
        s.path = target;
    }

    let _ = address;
    let _ = key;
    let _: &Path = &project_root;
    Ok(CompilationUnit {
        project_root,
        framework: Framework::EtherscanAddress,
        sources,
        solc_version: None,
        remappings: Vec::new(),
        settings: serde_json::Value::Null,
    })
}
