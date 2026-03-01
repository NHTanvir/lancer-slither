//! Project discovery & ingestion.
//!
//! Most of the heavy lifting lives in the [`crytic_compile_rs`] crate; this
//! module is a thin re-export plus a convenience [`load_project`] entry point
//! used by both the CLI and the GUI.

pub mod discover;
pub mod etherscan;
pub mod solc_select;

use std::path::{Path, PathBuf};

use crate::ast;
use crate::config::SlitherConfig;
use crate::model::SourceUnit;
use crate::slither::Slither;
use crate::suppression::SuppressionMap;
use crytic_compile_rs::{compile, framework::Framework, CompilationUnit};

/// Output of [`load_project`]: a [`Slither`] context + dependency info.
#[derive(Debug)]
pub struct LoadedProject {
    pub slither: Slither,
    pub framework: Framework,
}

/// Compile + lift a project target.
pub fn load_project(target: &Path, config: SlitherConfig) -> eyre::Result<LoadedProject> {
    let unit = compile(target, &compile_options_from(&config))?;
    let slither = lift_compilation_unit(target, unit, config)?;
    let framework = slither
        .source_units
        .first()
        .and_then(|_| Some(Framework::Auto))
        .unwrap_or(Framework::Auto);
    Ok(LoadedProject { slither, framework })
}

fn compile_options_from(cfg: &SlitherConfig) -> crytic_compile_rs::CompileOptions {
    crytic_compile_rs::CompileOptions {
        force_framework: cfg
            .compile_force_framework
            .as_deref()
            .and_then(Framework::from_str),
        solc: cfg.solc.clone(),
        solc_args: cfg.solc_args.clone(),
        solc_remaps: cfg.solc_remaps.clone(),
        solc_working_dir: cfg.solc_working_dir.clone(),
        solc_solcs_select: cfg.solc_solcs_select.clone(),
        solc_solcs_bin: cfg.solc_solcs_bin.clone(),
        solc_disable_warnings: cfg.solc_disable_warnings,
        ignore_compile: cfg.ignore_compile,
        skip_clean: cfg.skip_clean,
        custom_build: cfg.compile_custom_build.clone(),
        libraries: cfg.compile_libraries.clone(),
        remove_metadata: cfg.compile_remove_metadata,
        standard_json: cfg.standard_json.clone(),
        etherscan_apikey: cfg.etherscan_apikey.clone(),
        etherscan_export_dir: cfg.etherscan_export_directory.clone(),
        etherscan_only_source: cfg.etherscan_only_source_code,
        etherscan_only_bytecode: cfg.etherscan_only_bytecode,
        etherscan_network: cfg.etherscan_network.clone(),
    }
}

/// Convert a [`CompilationUnit`] into a fully-populated [`Slither`].
pub fn lift_compilation_unit(
    target: &Path,
    unit: CompilationUnit,
    config: SlitherConfig,
) -> eyre::Result<Slither> {
    let project_root = unit.project_root.clone();
    let mut source_units: Vec<SourceUnit> = Vec::with_capacity(unit.sources.len());
    let mut suppression = SuppressionMap::default();
    let mut deps: Vec<PathBuf> = Vec::new();

    for src in unit.sources.iter() {
        suppression.parse_file(&src.path, &src.content);
        if src.is_dependency {
            deps.push(src.path.clone());
        }
        let source_unit = if let Some(ast_json) = &src.ast {
            ast::lift_source_unit(ast_json, &src.path, Some(&project_root), &src.content)
        } else {
            ast::fallback_scrape(&src.path, Some(&project_root), &src.content)
        };
        source_units.push(source_unit);
    }
    let mut slither = Slither::new(project_root, source_units);
    slither.suppression = std::sync::Arc::new(suppression);
    slither.config = config;
    slither.dependency_paths = deps;
    slither.solc_version = unit.solc_version.clone();
    let _ = target;
    Ok(slither)
}
