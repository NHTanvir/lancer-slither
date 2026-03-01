//! Direct solc invocation for single-file or unframed projects.

use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::artifacts::walk_sources;
use crate::framework::Framework;
use crate::solc::pick_solc_for_pragma;
use crate::{CompiledSource, CompilationUnit, CompileOptions};

pub fn compile(target: &Path, opts: &CompileOptions) -> eyre::Result<CompilationUnit> {
    let mut unit = CompilationUnit::default();
    unit.project_root = if target.is_file() {
        target.parent().unwrap_or(target).to_path_buf()
    } else {
        target.to_path_buf()
    };
    unit.framework = Framework::Solc;

    let files: Vec<_> = if target.is_file() {
        vec![target.to_path_buf()]
    } else {
        walkdir::WalkDir::new(target)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("sol"))
            .map(|e| e.path().to_path_buf())
            .collect()
    };

    let chosen_solc = opts
        .solc
        .clone()
        .or_else(|| opts.solc_solcs_bin.as_ref().map(|p| p.to_string_lossy().into_owned()))
        .or_else(|| which::which("solc").ok().map(|p| p.to_string_lossy().into_owned()));

    for file in files {
        let content = std::fs::read_to_string(&file).unwrap_or_default();
        let mut ast: Option<Value> = None;
        if let Some(solc) = &chosen_solc {
            if !opts.ignore_compile {
                if let Some(version) = pick_solc_for_pragma(&content) {
                    tracing::debug!("Using solc {} for {}", version, file.display());
                }
                let out = Command::new(solc)
                    .arg("--combined-json")
                    .arg("ast,abi,bin,bin-runtime,srcmap,srcmap-runtime")
                    .arg(&file)
                    .output();
                if let Ok(out) = out {
                    let text = String::from_utf8_lossy(&out.stdout);
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        let key = file.to_string_lossy().to_string();
                        if let Some(contracts) = json.get("contracts") {
                            for (_k, v) in contracts.as_object().unwrap_or(&serde_json::Map::new()) {
                                if let Some(a) = v.get("AST") {
                                    ast = Some(a.clone());
                                    break;
                                }
                            }
                        }
                        let _ = key;
                    }
                }
            }
        }
        unit.sources.push(CompiledSource {
            path: file.clone(),
            content,
            ast,
            source_map: None,
            bytecode: None,
            deployed_bytecode: None,
            abi: None,
            storage_layout: None,
            is_dependency: false,
        });
    }

    if unit.sources.is_empty() {
        // last resort: walk sources without solc
        walk_sources(&mut unit, target, &[target])?;
    }

    Ok(unit)
}
