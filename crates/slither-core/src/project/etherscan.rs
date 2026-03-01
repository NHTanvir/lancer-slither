//! Convenience wrapper around `crytic-compile-rs`' Etherscan fetcher.
//!
//! The real fetcher (HTTP, network selection, cache layout) lives in
//! [`crytic_compile_rs::etherscan`]; this module exists so callers in
//! `slither-core` can stay framework-agnostic.

pub use crytic_compile_rs::etherscan::{fetch as fetch_address, Network};
