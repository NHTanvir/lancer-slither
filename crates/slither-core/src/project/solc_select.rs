//! Convenience wrapper around `crytic-compile-rs`' solc selector.

pub use crytic_compile_rs::solc::{
    download_solc, find_solc_binary, pick_solc_for_pragma, SolcBinary,
};
