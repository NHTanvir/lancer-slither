//! `slither.config.json` — same schema as upstream Slither.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::finding::Impact;

/// Configuration honoured by `--config-file` and merged from CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SlitherConfig {
    /// Comma-separated list of detector names to run, or "all".
    pub detectors_to_run: String,
    /// Comma-separated list to exclude.
    pub detectors_to_exclude: String,
    /// Include-list overrides exclusion (added 0.10.3).
    pub include_detectors: String,
    pub exclude_dependencies: bool,
    pub exclude_optimization: bool,
    pub exclude_informational: bool,
    pub exclude_low: bool,
    pub exclude_medium: bool,
    pub exclude_high: bool,
    pub show_ignored_findings: bool,
    pub filter_paths: Option<String>,
    pub include_paths: Option<String>,
    pub fail_pedantic: bool,
    pub fail_low: bool,
    pub fail_medium: bool,
    pub fail_high: bool,
    pub no_fail_pedantic: bool,
    pub solc_disable_warnings: bool,
    pub disable_color: bool,
    pub solc: Option<String>,
    pub solc_args: Option<String>,
    pub solc_remaps: Option<String>,
    pub solc_working_dir: Option<PathBuf>,
    pub solc_solcs_select: Option<String>,
    pub solc_solcs_bin: Option<PathBuf>,
    pub compile_force_framework: Option<String>,
    pub compile_libraries: Option<String>,
    pub compile_remove_metadata: bool,
    pub compile_custom_build: Option<String>,
    pub ignore_compile: bool,
    pub skip_clean: bool,
    pub standard_json: Option<PathBuf>,
    pub etherscan_apikey: Option<String>,
    pub etherscan_export_directory: Option<PathBuf>,
    pub etherscan_only_source_code: bool,
    pub etherscan_only_bytecode: bool,
    pub etherscan_export_dir_format: Option<String>,
    pub etherscan_network: Option<String>,
    pub triage_mode: bool,
    pub triage_database: Option<PathBuf>,
    pub change_line_prefix: Option<String>,
    pub json: Option<PathBuf>,
    pub sarif: Option<PathBuf>,
    pub markdown: Option<PathBuf>,
    pub markdown_root: Option<String>,
    pub checklist: bool,
    pub zip: Option<PathBuf>,
    pub ai_rank: bool,
    pub mcp_server: bool,
    /// Run only these printers (comma-separated).
    pub print: Option<String>,
}

impl Default for SlitherConfig {
    fn default() -> Self {
        Self {
            detectors_to_run: "all".to_owned(),
            detectors_to_exclude: String::new(),
            include_detectors: String::new(),
            exclude_dependencies: false,
            exclude_optimization: false,
            exclude_informational: false,
            exclude_low: false,
            exclude_medium: false,
            exclude_high: false,
            show_ignored_findings: false,
            filter_paths: None,
            include_paths: None,
            fail_pedantic: false,
            fail_low: false,
            fail_medium: false,
            fail_high: false,
            no_fail_pedantic: false,
            solc_disable_warnings: false,
            disable_color: false,
            solc: None,
            solc_args: None,
            solc_remaps: None,
            solc_working_dir: None,
            solc_solcs_select: None,
            solc_solcs_bin: None,
            compile_force_framework: None,
            compile_libraries: None,
            compile_remove_metadata: false,
            compile_custom_build: None,
            ignore_compile: false,
            skip_clean: false,
            standard_json: None,
            etherscan_apikey: None,
            etherscan_export_directory: None,
            etherscan_only_source_code: false,
            etherscan_only_bytecode: false,
            etherscan_export_dir_format: None,
            etherscan_network: None,
            triage_mode: false,
            triage_database: None,
            change_line_prefix: None,
            json: None,
            sarif: None,
            markdown: None,
            markdown_root: None,
            checklist: false,
            zip: None,
            ai_rank: false,
            mcp_server: false,
            print: None,
        }
    }
}

impl SlitherConfig {
    pub fn load(path: &Path) -> eyre::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let cfg: Self = serde_json::from_str(&raw)?;
        Ok(cfg)
    }

    /// Auto-discover `slither.config.json` walking up from `cwd`.
    pub fn discover(cwd: &Path) -> Option<PathBuf> {
        let mut dir: Option<&Path> = Some(cwd);
        while let Some(d) = dir {
            let p = d.join("slither.config.json");
            if p.is_file() {
                return Some(p);
            }
            dir = d.parent();
        }
        None
    }

    pub fn is_detector_excluded_by_tier(&self, impact: Impact) -> bool {
        match impact {
            Impact::High => self.exclude_high,
            Impact::Medium => self.exclude_medium,
            Impact::Low => self.exclude_low,
            Impact::Informational => self.exclude_informational,
            Impact::Optimization => self.exclude_optimization,
        }
    }
}
