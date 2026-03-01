//! Hardhat driver: run `npx hardhat compile`, ingest `artifacts/build-info/`.

use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::artifacts::{ingest_build_info, walk_sources};
use crate::framework::Framework;
use crate::{CompilationUnit, CompileOptions};

pub fn compile(target: &Path, opts: &CompileOptions) -> eyre::Result<CompilationUnit> {
    let mut unit = CompilationUnit::default();
    unit.project_root = target.to_path_buf();
    unit.framework = Framework::Hardhat;

    if !opts.ignore_compile {
        if let Ok(npx) = which::which("npx") {
            let mut cmd = Command::new(npx);
            cmd.args(["hardhat", "compile"]).current_dir(target);
            let _ = cmd.status();
        } else {
            tracing::warn!("npx not found; skipping hardhat compile and reading existing artifacts");
        }
    }

    let bi = target.join("artifacts").join("build-info");
    if bi.is_dir() {
        for entry in walkdir::WalkDir::new(&bi)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(raw) = std::fs::read_to_string(entry.path()) {
                    if let Ok(json) = serde_json::from_str::<Value>(&raw) {
                        ingest_build_info(&mut unit, &json, target)?;
                    }
                }
            }
        }
    }

    if unit.sources.is_empty() {
        walk_sources(&mut unit, target, &[
            target.join("contracts").as_path(),
            target.join("src").as_path(),
            target.join("node_modules").as_path(),
        ])?;
    }
    for s in &mut unit.sources {
        let p = s.path.to_string_lossy().to_lowercase();
        s.is_dependency = p.contains("node_modules");
    }
    Ok(unit)
}
