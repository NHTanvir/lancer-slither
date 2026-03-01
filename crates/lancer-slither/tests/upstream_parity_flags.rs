//! Pinned set of upstream-slither CLI flags. CI checks every flag below
//! is present in `lancer_slither::cli::Cli`. The list is verified against
//! crytic/slither 0.11.5 `--help` output.
//!
//! Run: `cargo test -p lancer-slither --test upstream_parity_flags`.

use clap::CommandFactory;
use lancer_slither::cli::Cli;

const EXPECTED: &[&str] = &[
    "detect", "exclude", "include-detectors",
    "exclude-dependencies", "exclude-optimization", "exclude-informational",
    "exclude-low", "exclude-medium", "exclude-high",
    "show-ignored-findings", "filter-paths", "include-paths",
    "print", "list-detectors", "list-printers",
    "json", "sarif", "markdown", "markdown-root", "checklist", "zip",
    "fail-pedantic", "no-fail-pedantic", "fail-low", "fail-medium", "fail-high",
    "compile-force-framework", "compile-libraries", "compile-remove-metadata", "compile-custom-build",
    "solc", "solc-args", "solc-disable-warnings", "solc-working-dir", "solc-remaps",
    "solc-solcs-select", "solc-solcs-bin",
    "ignore-compile", "skip-clean", "standard-json",
    "etherscan-apikey", "etherscan-export-directory", "etherscan-only-source-code",
    "etherscan-only-bytecode", "etherscan-export-dir-format", "etherscan-network",
    "triage-mode", "triage-database", "change-line-prefix",
    "config-file", "disable-color", "debug",
];

#[test]
fn every_upstream_flag_is_present() {
    let cmd = Cli::command();
    let names: std::collections::BTreeSet<String> = cmd
        .get_arguments()
        .filter_map(|a| a.get_long().map(str::to_owned))
        .collect();
    let mut missing = Vec::new();
    for flag in EXPECTED {
        if !names.contains(*flag) {
            missing.push(*flag);
        }
    }
    assert!(missing.is_empty(), "missing CLI flags vs upstream: {missing:?}");
}
