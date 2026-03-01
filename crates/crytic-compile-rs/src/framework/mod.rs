//! Framework detection + per-framework compile drivers.

pub mod auto;
pub mod brownie;
pub mod dapp;
pub mod foundry;
pub mod hardhat;
pub mod solc_direct;
pub mod truffle;
pub mod vyper;

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Project framework Slither recognizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Framework {
    #[default]
    Auto,
    Foundry,
    Hardhat,
    Truffle,
    Brownie,
    Dapp,
    Vyper,
    Solc,
    StandardJson,
    Embark,
    Waffle,
    Buidler,
    EtherscanAddress,
}

impl Framework {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "foundry" => Self::Foundry,
            "hardhat" => Self::Hardhat,
            "truffle" => Self::Truffle,
            "brownie" => Self::Brownie,
            "dapp" => Self::Dapp,
            "vyper" => Self::Vyper,
            "solc" => Self::Solc,
            "standard" | "standard-json" | "standard_json" => Self::StandardJson,
            "embark" => Self::Embark,
            "waffle" => Self::Waffle,
            "buidler" => Self::Buidler,
            "auto" => Self::Auto,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Foundry => "foundry",
            Self::Hardhat => "hardhat",
            Self::Truffle => "truffle",
            Self::Brownie => "brownie",
            Self::Dapp => "dapp",
            Self::Vyper => "vyper",
            Self::Solc => "solc",
            Self::StandardJson => "standard-json",
            Self::Embark => "embark",
            Self::Waffle => "waffle",
            Self::Buidler => "buidler",
            Self::EtherscanAddress => "etherscan",
        }
    }
}

/// Heuristic framework discovery.
pub fn detect(target: &Path) -> Option<Framework> {
    if target.is_file() {
        let ext = target.extension().and_then(|e| e.to_str()).unwrap_or("");
        return Some(match ext {
            "vy" => Framework::Vyper,
            "json" => {
                // Could be a standard-json input. Peek for "language".
                if peek_is_standard_json(target) {
                    Framework::StandardJson
                } else {
                    Framework::Solc
                }
            }
            _ => Framework::Solc,
        });
    }
    if target.join("foundry.toml").is_file() {
        return Some(Framework::Foundry);
    }
    for n in ["hardhat.config.js", "hardhat.config.ts", "hardhat.config.cjs"] {
        if target.join(n).is_file() {
            return Some(Framework::Hardhat);
        }
    }
    if target.join("truffle-config.js").is_file() || target.join("truffle.js").is_file() {
        return Some(Framework::Truffle);
    }
    if target.join("brownie-config.yaml").is_file() {
        return Some(Framework::Brownie);
    }
    if target.join("dapp.toml").is_file() || target.join("Makefile").is_file() {
        // Dapp's Makefile is the more reliable hint
        if target.join("dapp.toml").is_file() {
            return Some(Framework::Dapp);
        }
    }
    if any_file_with_ext(target, "vy") {
        return Some(Framework::Vyper);
    }
    None
}

fn peek_is_standard_json(path: &Path) -> bool {
    let Ok(s) = std::fs::read_to_string(path) else {
        return false;
    };
    s.contains("\"language\"") && (s.contains("\"Solidity\"") || s.contains("\"Yul\""))
}

fn any_file_with_ext(dir: &Path, ext: &str) -> bool {
    walkdir::WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
        .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some(ext))
}

pub fn is_etherscan_address(s: &str) -> bool {
    let t = s.trim();
    t.starts_with("0x") && t.len() == 42 && t[2..].chars().all(|c| c.is_ascii_hexdigit())
}
