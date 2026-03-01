//! `Slither` — the central analysis context.
//!
//! Built once from a successful `crytic-compile-rs` run, then passed
//! immutably to every detector and printer.

use std::path::PathBuf;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::config::SlitherConfig;
use crate::model::{Contract, Function, SourceUnit};
use crate::suppression::SuppressionMap;

/// The shared analysis context.
#[derive(Debug, Clone, Default)]
pub struct Slither {
    /// Project root directory.
    pub project_root: PathBuf,
    /// All parsed source files.
    pub source_units: Vec<SourceUnit>,
    /// Quick lookup: contract name → indices into `source_units`.
    pub contracts_by_name: IndexMap<String, Vec<(usize, usize)>>,
    /// Quick lookup: function full name (Contract.name) → coordinates.
    pub functions_by_full_name: IndexMap<String, Vec<FunctionRef>>,
    /// NatSpec `slither-disable-*` suppression map (per file).
    pub suppression: Arc<SuppressionMap>,
    /// Effective configuration.
    pub config: SlitherConfig,
    /// Triage decisions loaded from `slither-triage.json`.
    pub triage_db: Arc<crate::triage::TriageDb>,
    /// solc version that was used.
    pub solc_version: Option<String>,
    /// Paths flagged as `dependencies` by crytic-compile.
    pub dependency_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub struct FunctionRef {
    pub source_unit: usize,
    pub contract: usize,
    pub function: usize,
}

impl Slither {
    pub fn new(project_root: PathBuf, source_units: Vec<SourceUnit>) -> Self {
        let mut me = Self {
            project_root,
            source_units,
            contracts_by_name: IndexMap::new(),
            functions_by_full_name: IndexMap::new(),
            suppression: Arc::new(SuppressionMap::default()),
            config: SlitherConfig::default(),
            triage_db: Arc::new(crate::triage::TriageDb::default()),
            solc_version: None,
            dependency_paths: Vec::new(),
        };
        me.rebuild_indices();
        me
    }

    pub fn rebuild_indices(&mut self) {
        self.contracts_by_name.clear();
        self.functions_by_full_name.clear();
        for (i, su) in self.source_units.iter().enumerate() {
            for (j, c) in su.contracts.iter().enumerate() {
                self.contracts_by_name
                    .entry(c.name.clone())
                    .or_default()
                    .push((i, j));
                for (k, f) in c.functions.iter().enumerate() {
                    self.functions_by_full_name
                        .entry(f.full_name())
                        .or_default()
                        .push(FunctionRef {
                            source_unit: i,
                            contract: j,
                            function: k,
                        });
                }
            }
        }
    }

    pub fn all_contracts(&self) -> impl Iterator<Item = &Contract> {
        self.source_units.iter().flat_map(|su| su.contracts.iter())
    }

    pub fn all_functions(&self) -> impl Iterator<Item = (&Contract, &Function)> {
        self.source_units
            .iter()
            .flat_map(|su| su.contracts.iter())
            .flat_map(|c| c.functions.iter().map(move |f| (c, f)))
    }

    /// Iterate functions, filtering out functions whose source file is
    /// flagged as a dependency (when `--exclude-dependencies` is set).
    pub fn user_functions(&self) -> impl Iterator<Item = (&Contract, &Function)> {
        let deps = self.dependency_paths.clone();
        let exclude = self.config.exclude_dependencies;
        self.all_functions().filter(move |(_, f)| {
            if !exclude {
                return true;
            }
            !deps.iter().any(|d| f.source.filename_absolute.starts_with(d))
        })
    }

    pub fn user_source_units(&self) -> impl Iterator<Item = &SourceUnit> {
        let deps = self.dependency_paths.clone();
        let exclude = self.config.exclude_dependencies;
        self.source_units.iter().filter(move |su| {
            if !exclude {
                return true;
            }
            !deps.iter().any(|d| su.path.starts_with(d))
        })
    }
}
