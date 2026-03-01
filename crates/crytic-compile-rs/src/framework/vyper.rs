//! Vyper driver — compiles each `.vy` file via `vyper -f ast_dict`.

use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::framework::Framework;
use crate::{CompiledSource, CompilationUnit, CompileOptions};

pub fn compile(target: &Path, opts: &CompileOptions) -> eyre::Result<CompilationUnit> {
    let mut unit = CompilationUnit::default();
    unit.project_root = if target.is_file() {
        target.parent().unwrap_or(target).to_path_buf()
    } else {
        target.to_path_buf()
    };
    unit.framework = Framework::Vyper;

    let files: Vec<_> = if target.is_file() {
        vec![target.to_path_buf()]
    } else {
        walkdir::WalkDir::new(target)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("vy"))
            .map(|e| e.path().to_path_buf())
            .collect()
    };

    let vyper = which::which("vyper").ok();
    for file in files {
        let content = std::fs::read_to_string(&file).unwrap_or_default();
        let ast = if let Some(v) = &vyper {
            if !opts.ignore_compile {
                Command::new(v)
                    .args(["-f", "ast_dict"])
                    .arg(&file)
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            } else {
                None
            }
        } else {
            None
        };
        unit.sources.push(CompiledSource {
            path: file,
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
    Ok(unit)
}
