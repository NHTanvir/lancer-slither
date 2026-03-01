//! Dapp framework — `dapp build`.

use std::path::Path;
use std::process::Command;

use crate::artifacts::walk_sources;
use crate::framework::Framework;
use crate::{CompilationUnit, CompileOptions};

pub fn compile(target: &Path, opts: &CompileOptions) -> eyre::Result<CompilationUnit> {
    let mut unit = CompilationUnit::default();
    unit.project_root = target.to_path_buf();
    unit.framework = Framework::Dapp;
    if !opts.ignore_compile {
        if let Ok(dapp) = which::which("dapp") {
            let _ = Command::new(dapp).arg("build").current_dir(target).status();
        }
    }
    walk_sources(&mut unit, target, &[
        target.join("src").as_path(),
        target.join("lib").as_path(),
    ])?;
    Ok(unit)
}
