//! Brownie driver — `brownie compile`, ingest `build/contracts/*.json`.

use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::artifacts::{ingest_truffle_artifact, walk_sources};
use crate::framework::Framework;
use crate::{CompilationUnit, CompileOptions};

pub fn compile(target: &Path, opts: &CompileOptions) -> eyre::Result<CompilationUnit> {
    let mut unit = CompilationUnit::default();
    unit.project_root = target.to_path_buf();
    unit.framework = Framework::Brownie;

    if !opts.ignore_compile {
        if let Ok(brownie) = which::which("brownie") {
            let _ = Command::new(brownie).arg("compile").current_dir(target).status();
        }
    }

    let dir = target.join("build").join("contracts");
    if dir.is_dir() {
        for e in walkdir::WalkDir::new(&dir).into_iter().filter_map(Result::ok) {
            if e.path().extension().and_then(|x| x.to_str()) == Some("json") {
                if let Ok(raw) = std::fs::read_to_string(e.path()) {
                    if let Ok(json) = serde_json::from_str::<Value>(&raw) {
                        ingest_truffle_artifact(&mut unit, &json, target)?;
                    }
                }
            }
        }
    }
    if unit.sources.is_empty() {
        walk_sources(&mut unit, target, &[
            target.join("contracts").as_path(),
            target.join("interfaces").as_path(),
        ])?;
    }
    Ok(unit)
}
