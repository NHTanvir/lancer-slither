//! Slither-native JSON emitter.

use std::path::Path;

use crate::finding::OutputResults;

pub fn write(path: &Path, results: &OutputResults) -> eyre::Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn to_string(results: &OutputResults) -> eyre::Result<String> {
    Ok(serde_json::to_string_pretty(results)?)
}
