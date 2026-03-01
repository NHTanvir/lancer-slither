//! Foundry: read `foundry.toml`, run `forge build --build-info`, ingest
//! `out/build-info/*.json`.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use crate::artifacts::{ingest_build_info, walk_sources};
use crate::framework::Framework;
use crate::{CompilationUnit, CompileOptions};

pub fn compile(target: &Path, opts: &CompileOptions) -> eyre::Result<CompilationUnit> {
    let mut unit = CompilationUnit::default();
    unit.project_root = target.to_path_buf();
    unit.framework = Framework::Foundry;

    if !opts.ignore_compile {
        let forge = which::which("forge")
            .map_err(|_| eyre::eyre!("forge not on PATH; install Foundry"))?;
        let mut cmd = Command::new(forge);
        cmd.arg("build")
            .arg("--build-info")
            .arg("--no-cache")
            .current_dir(target);
        if let Some(extra) = &opts.solc_args {
            for arg in extra.split_whitespace() {
                cmd.arg(arg);
            }
        }
        let status = cmd.status()?;
        if !status.success() {
            tracing::warn!("forge build returned {status}; continuing with whatever artifacts exist");
        }
    }

    let build_info_dir = target.join("out").join("build-info");
    if build_info_dir.is_dir() {
        for entry in walkdir::WalkDir::new(&build_info_dir)
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

    // Even if forge build wasn't run, still walk sources so we can run with
    // `fallback_scrape`.
    if unit.sources.is_empty() {
        walk_sources(&mut unit, target, &[
            target.join("src").as_path(),
            target.join("contracts").as_path(),
            target.join("script").as_path(),
            target.join("test").as_path(),
            target.join("lib").as_path(),
        ])?;
    }

    mark_dependencies(&mut unit);
    Ok(unit)
}

fn mark_dependencies(unit: &mut CompilationUnit) {
    for s in &mut unit.sources {
        let p = s.path.to_string_lossy().to_lowercase();
        s.is_dependency = p.contains("/lib/")
            || p.contains("\\lib\\")
            || p.contains("/node_modules/")
            || p.contains("\\node_modules\\")
            || p.contains("/forge-std/");
    }
}

#[allow(dead_code)]
fn _unused(_: PathBuf) {}
