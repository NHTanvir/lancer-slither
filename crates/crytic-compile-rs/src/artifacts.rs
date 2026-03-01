//! Helpers for ingesting solc artifact JSON shapes (Foundry build-info,
//! Hardhat build-info, Truffle/Brownie contract artifacts).

use std::path::Path;

use serde_json::Value;

use crate::{CompiledSource, CompilationUnit};

/// Ingest a Foundry / Hardhat `build-info/*.json` file into the unit.
///
/// Build-info shape (truncated):
/// ```json
/// { "id":"...", "solcVersion":"0.8.20", "input":{...},
///   "output":{ "sources": { "src/A.sol": {"id":0,"ast":{...}}, ... },
///              "contracts": { "src/A.sol": { "A": {"abi":..., "evm":{...}, "storageLayout":...} } } } }
/// ```
pub fn ingest_build_info(
    unit: &mut CompilationUnit,
    json: &Value,
    project_root: &Path,
) -> eyre::Result<()> {
    if let Some(ver) = json.get("solcVersion").and_then(Value::as_str) {
        unit.solc_version = Some(ver.to_owned());
    }
    let Some(output) = json.get("output") else {
        return Ok(());
    };
    let Some(sources) = output.get("sources").and_then(Value::as_object) else {
        return Ok(());
    };

    for (key, src_node) in sources {
        let rel = key.as_str();
        let path = project_root.join(rel);
        let content = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => {
                // Try absolute, sometimes solc emits absolute paths.
                std::fs::read_to_string(rel).unwrap_or_default()
            }
        };
        let ast = src_node.get("ast").cloned();
        let mut cs = CompiledSource {
            path: if path.exists() {
                path
            } else {
                std::path::PathBuf::from(rel)
            },
            content,
            ast,
            source_map: None,
            bytecode: None,
            deployed_bytecode: None,
            abi: None,
            storage_layout: None,
            is_dependency: false,
        };
        // Attach contract artifacts (abi/bytecode/srcmap/storageLayout).
        if let Some(contracts) = output.get("contracts").and_then(|c| c.get(rel)) {
            if let Some(obj) = contracts.as_object() {
                if let Some((_, first)) = obj.iter().next() {
                    cs.abi = first.get("abi").cloned();
                    cs.bytecode = first
                        .get("evm")
                        .and_then(|e| e.get("bytecode"))
                        .and_then(|b| b.get("object"))
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                    cs.deployed_bytecode = first
                        .get("evm")
                        .and_then(|e| e.get("deployedBytecode"))
                        .and_then(|b| b.get("object"))
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                    cs.source_map = first
                        .get("evm")
                        .and_then(|e| e.get("deployedBytecode"))
                        .and_then(|b| b.get("sourceMap"))
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                    cs.storage_layout = first.get("storageLayout").cloned();
                }
            }
        }
        unit.sources.push(cs);
    }
    Ok(())
}

pub fn ingest_truffle_artifact(
    unit: &mut CompilationUnit,
    json: &Value,
    project_root: &Path,
) -> eyre::Result<()> {
    let Some(sources) = json.get("sourcePath").and_then(Value::as_str) else {
        return Ok(());
    };
    let ast = json.get("ast").cloned();
    let path = project_root.join(sources);
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    unit.sources.push(CompiledSource {
        path,
        content,
        ast,
        source_map: json
            .get("deployedSourceMap")
            .and_then(Value::as_str)
            .map(str::to_owned),
        bytecode: json.get("bytecode").and_then(Value::as_str).map(str::to_owned),
        deployed_bytecode: json
            .get("deployedBytecode")
            .and_then(Value::as_str)
            .map(str::to_owned),
        abi: json.get("abi").cloned(),
        storage_layout: None,
        is_dependency: false,
    });
    Ok(())
}

/// Fallback for when no artifacts were produced — just glob `.sol` files
/// out of well-known source directories.
pub fn walk_sources(
    unit: &mut CompilationUnit,
    root: &Path,
    candidates: &[&Path],
) -> eyre::Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for c in candidates {
        if !c.is_dir() {
            continue;
        }
        for e in walkdir::WalkDir::new(c).into_iter().filter_map(Result::ok) {
            if e.path().extension().and_then(|x| x.to_str()) == Some("sol") {
                if seen.insert(e.path().to_path_buf()) {
                    let content = std::fs::read_to_string(e.path()).unwrap_or_default();
                    unit.sources.push(CompiledSource {
                        path: e.path().to_path_buf(),
                        content,
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
    }
    if unit.sources.is_empty() && root.is_dir() {
        for e in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if e.path().extension().and_then(|x| x.to_str()) == Some("sol") {
                if seen.insert(e.path().to_path_buf()) {
                    let content = std::fs::read_to_string(e.path()).unwrap_or_default();
                    unit.sources.push(CompiledSource {
                        path: e.path().to_path_buf(),
                        content,
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
    }
    Ok(())
}
