//! Project discovery — walk a directory, detect the framework, dispatch.
//!
//! This is the thin in-`slither-core` wrapper. Actual framework probing
//! (`foundry.toml` vs `hardhat.config.{js,ts,cjs}` vs `truffle-config.js` vs
//! `brownie-config.yaml` vs `dapp.toml` vs `vapor.toml` vs `.vy` files) is
//! delegated to [`crytic_compile_rs::framework`].

use std::path::Path;

pub use crytic_compile_rs::framework::Framework;

/// Auto-detect framework. `None` means "fall back to single-file solc".
pub fn detect(target: &Path) -> Option<Framework> {
    crytic_compile_rs::framework::detect(target)
}

/// `true` if the string looks like an Etherscan address (`0x` + 40 hex).
pub fn is_etherscan_address(s: &str) -> bool {
    crytic_compile_rs::framework::is_etherscan_address(s)
}
