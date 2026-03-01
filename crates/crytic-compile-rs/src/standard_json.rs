//! solc standard-json input ingestion.

use std::path::Path;

use serde_json::Value;

use crate::artifacts::ingest_build_info;
use crate::framework::Framework;
use crate::{CompilationUnit, CompileOptions};

pub fn compile(target: &Path, _opts: &CompileOptions) -> eyre::Result<CompilationUnit> {
    let mut unit = CompilationUnit::default();
    unit.project_root = target
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    unit.framework = Framework::StandardJson;
    let raw = std::fs::read_to_string(target)?;
    let json: Value = serde_json::from_str(&raw)?;
    ingest_build_info(&mut unit, &json, &unit.project_root.clone())?;
    Ok(unit)
}
