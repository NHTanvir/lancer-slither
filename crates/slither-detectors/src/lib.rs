//! `slither-detectors` — every detector ships with full metadata
//! (CLI argument, impact, confidence, wiki URL/description/exploit/
//! recommendation) plus a `detect()` body.
//!
//! Detectors are registered into a global `all()` list at crate load.
//!
//! ### Detector tiers (per upstream 0.11.5)
//!
//! * **high** — exploit-grade issues (`high.rs`, 15 detectors).
//! * **medium** — significant issues (`medium.rs`, 42 detectors).
//! * **low** — hardening (`low.rs`, 17 detectors).
//! * **informational** — code quality (`informational.rs`, 22 detectors).
//! * **optimization** — gas / size (`optimization.rs`, 5 detectors).

#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc, clippy::missing_panics_doc)]

#[macro_use]
mod macros;
pub mod high;
pub mod informational;
pub mod low;
pub mod medium;
pub mod optimization;
pub mod util;

use slither_core::{Confidence, Finding, Impact, Slither};

/// The detector trait — implement for every check.
pub trait Detector: Send + Sync + 'static {
    fn argument(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn impact(&self) -> Impact;
    fn confidence(&self) -> Confidence;
    fn wiki_url(&self) -> &'static str {
        ""
    }
    fn wiki_title(&self) -> &'static str {
        ""
    }
    fn wiki_description(&self) -> &'static str {
        ""
    }
    fn wiki_exploit_scenario(&self) -> &'static str {
        ""
    }
    fn wiki_recommendation(&self) -> &'static str {
        ""
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding>;
}

/// Return the complete detector roster.
pub fn all() -> Vec<Box<dyn Detector + Send + Sync>> {
    let mut v: Vec<Box<dyn Detector + Send + Sync>> = Vec::new();
    high::register(&mut v);
    medium::register(&mut v);
    low::register(&mut v);
    informational::register(&mut v);
    optimization::register(&mut v);
    v
}

/// Look up a detector by CLI argument name.
pub fn find_by_argument(name: &str) -> Option<Box<dyn Detector + Send + Sync>> {
    all().into_iter().find(|d| d.argument() == name)
}

/// Apply config filters (`--detect`, `--exclude`, tier exclusions,
/// `--include-detectors`) and return the effective detector subset.
pub fn select(slither: &Slither) -> Vec<Box<dyn Detector + Send + Sync>> {
    let cfg = &slither.config;
    let include: ahash::AHashSet<&str> = cfg
        .include_detectors
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .collect();
    let exclude: ahash::AHashSet<&str> = cfg
        .detectors_to_exclude
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .collect();
    let run_all = cfg.detectors_to_run == "all" || cfg.detectors_to_run.is_empty();
    let run_set: ahash::AHashSet<&str> = if run_all {
        ahash::AHashSet::new()
    } else {
        cfg.detectors_to_run
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .collect()
    };
    all()
        .into_iter()
        .filter(|d| {
            let arg = d.argument();
            if include.contains(arg) {
                return true;
            }
            if exclude.contains(arg) {
                return false;
            }
            if cfg.is_detector_excluded_by_tier(d.impact()) {
                return false;
            }
            if !run_all && !run_set.contains(arg) {
                return false;
            }
            true
        })
        .collect()
}

/// Run all selected detectors. Findings are aggregated, suppression filtered,
/// then sorted by (impact desc, confidence desc, file, line).
pub fn run_all(slither: &Slither) -> Vec<Finding> {
    use rayon::prelude::*;
    let detectors = select(slither);
    let mut findings: Vec<Finding> = detectors
        .par_iter()
        .flat_map(|d| {
            d.detect(slither)
                .into_iter()
                .filter(|f| {
                    if slither.config.show_ignored_findings {
                        return true;
                    }
                    !is_suppressed(slither, f)
                })
                .filter(|f| !slither.triage_db.is_false_positive(&f.id))
                .collect::<Vec<_>>()
        })
        .collect();
    findings.sort_by(|a, b| {
        b.impact
            .cmp(&a.impact)
            .then(b.confidence.cmp(&a.confidence))
            .then(
                a.elements
                    .first()
                    .map(|e| e.source_mapping.filename_short.clone())
                    .unwrap_or_default()
                    .cmp(
                        &b.elements
                            .first()
                            .map(|e| e.source_mapping.filename_short.clone())
                            .unwrap_or_default(),
                    ),
            )
            .then(
                a.elements
                    .first()
                    .and_then(|e| e.source_mapping.lines.first().copied())
                    .unwrap_or(0)
                    .cmp(
                        &b.elements
                            .first()
                            .and_then(|e| e.source_mapping.lines.first().copied())
                            .unwrap_or(0),
                    ),
            )
    });
    findings
}

fn is_suppressed(slither: &Slither, f: &Finding) -> bool {
    f.elements.iter().any(|e| {
        let path = &e.source_mapping.filename_absolute;
        e.source_mapping
            .lines
            .iter()
            .any(|l| slither.suppression.is_suppressed(path, *l, &f.check))
    })
}
