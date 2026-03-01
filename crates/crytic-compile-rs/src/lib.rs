//! `crytic-compile-rs` — Rust port of crytic-compile.
//!
//! Given a project path or address, detect the build framework, invoke the
//! right toolchain, ingest the resulting artifacts, and normalize everything
//! into a [`CompilationUnit`] that downstream code consumes.

#![warn(missing_debug_implementations)]
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod artifacts;
pub mod etherscan;
pub mod framework;
pub mod solc;
pub mod standard_json;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use framework::Framework;

/// One source file inside a compilation unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledSource {
    pub path: PathBuf,
    pub content: String,
    pub ast: Option<serde_json::Value>,
    pub source_map: Option<String>,
    pub bytecode: Option<String>,
    pub deployed_bytecode: Option<String>,
    pub abi: Option<serde_json::Value>,
    pub storage_layout: Option<serde_json::Value>,
    /// Heuristic: source path lives under `node_modules/` or `lib/forge-std/`.
    pub is_dependency: bool,
}

/// The aggregate output of [`compile`].
#[derive(Debug, Clone, Default)]
pub struct CompilationUnit {
    pub project_root: PathBuf,
    pub framework: Framework,
    pub sources: Vec<CompiledSource>,
    pub solc_version: Option<String>,
    pub remappings: Vec<String>,
    /// Pass-through of solc settings (optimizer, evm version, etc.).
    pub settings: serde_json::Value,
}

/// Options forwarded from CLI / config to the compile pipeline.
#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    pub force_framework: Option<Framework>,
    pub solc: Option<String>,
    pub solc_args: Option<String>,
    pub solc_remaps: Option<String>,
    pub solc_working_dir: Option<PathBuf>,
    pub solc_solcs_select: Option<String>,
    pub solc_solcs_bin: Option<PathBuf>,
    pub solc_disable_warnings: bool,
    pub ignore_compile: bool,
    pub skip_clean: bool,
    pub custom_build: Option<String>,
    pub libraries: Option<String>,
    pub remove_metadata: bool,
    pub standard_json: Option<PathBuf>,
    pub etherscan_apikey: Option<String>,
    pub etherscan_export_dir: Option<PathBuf>,
    pub etherscan_only_source: bool,
    pub etherscan_only_bytecode: bool,
    pub etherscan_network: Option<String>,
}

/// The main entry point. Detects the framework and runs the appropriate
/// pipeline.
pub fn compile(target: &Path, opts: &CompileOptions) -> eyre::Result<CompilationUnit> {
    let target_str = target.to_string_lossy();

    // Etherscan: looks like 0x40-hex.
    if framework::is_etherscan_address(&target_str) {
        return etherscan::compile_address(&target_str, opts);
    }

    // Standard JSON input override.
    if let Some(json) = &opts.standard_json {
        return standard_json::compile(json, opts);
    }

    let target = if target.is_dir() {
        target.to_path_buf()
    } else if target.is_file() {
        target.to_path_buf()
    } else {
        eyre::bail!("target not found: {}", target.display());
    };

    let framework = opts
        .force_framework
        .or_else(|| framework::detect(&target))
        .unwrap_or(Framework::Auto);

    match framework {
        Framework::Foundry => framework::foundry::compile(&target, opts),
        Framework::Hardhat => framework::hardhat::compile(&target, opts),
        Framework::Truffle => framework::truffle::compile(&target, opts),
        Framework::Brownie => framework::brownie::compile(&target, opts),
        Framework::Dapp => framework::dapp::compile(&target, opts),
        Framework::Vyper => framework::vyper::compile(&target, opts),
        Framework::Solc => framework::solc_direct::compile(&target, opts),
        Framework::StandardJson => {
            standard_json::compile(&target, opts)
        }
        Framework::Auto => framework::auto::compile(&target, opts),
        Framework::Embark
        | Framework::Waffle
        | Framework::Buidler
        | Framework::EtherscanAddress => framework::solc_direct::compile(&target, opts),
    }
}
