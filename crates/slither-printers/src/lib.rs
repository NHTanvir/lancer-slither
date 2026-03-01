//! `slither-printers` — visualisations and human-readable summaries.
//!
//! Each printer implements the [`Printer`] trait; the dispatch happens via
//! `--print <name>` in the CLI.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::module_name_repetitions)]

pub mod call_graph;
pub mod cheatcode;
pub mod cfg;
pub mod contract_summary;
pub mod data_dependency;
pub mod echidna;
pub mod entry_points;
pub mod evm;
pub mod function_summary;
pub mod halstead;
pub mod human_summary;
pub mod inheritance;
pub mod loc;
pub mod martin;
pub mod modifiers;
pub mod not_pausable;
pub mod require;
pub mod slithir;
pub mod slithir_ssa;
pub mod vars_and_auth;

use std::io::Write;

use slither_core::Slither;

/// One printer.
pub trait Printer: Send + Sync + 'static {
    fn argument(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()>;
}

pub fn all() -> Vec<Box<dyn Printer + Send + Sync>> {
    vec![
        Box::new(human_summary::HumanSummary),
        Box::new(contract_summary::ContractSummary),
        Box::new(function_summary::FunctionSummary),
        Box::new(vars_and_auth::VarsAndAuth),
        Box::new(inheritance::Inheritance),
        Box::new(inheritance::InheritanceGraph),
        Box::new(call_graph::CallGraph),
        Box::new(cfg::Cfg),
        Box::new(data_dependency::DataDependency),
        Box::new(loc::Loc),
        Box::new(modifiers::Modifiers),
        Box::new(require::Require),
        Box::new(slithir::SlithIr),
        Box::new(slithir_ssa::SlithIrSsa),
        Box::new(echidna::Echidna),
        Box::new(evm::Evm),
        Box::new(martin::Martin),
        Box::new(halstead::Halstead),
        Box::new(not_pausable::NotPausable),
        Box::new(entry_points::EntryPoints),
        Box::new(cheatcode::Cheatcode),
    ]
}

pub fn find(name: &str) -> Option<Box<dyn Printer + Send + Sync>> {
    all().into_iter().find(|p| p.argument() == name)
}
