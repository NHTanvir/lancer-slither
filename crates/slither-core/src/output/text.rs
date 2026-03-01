//! Human-readable terminal output (mirrors upstream `slither`'s coloured
//! output but defaults to plain ANSI when `--disable-color` is set).

use std::fmt::Write;

use crate::finding::{Finding, Impact, OutputResults};

const C_RED: &str = "\x1b[31m";
const C_YEL: &str = "\x1b[33m";
const C_BLU: &str = "\x1b[34m";
const C_GRY: &str = "\x1b[90m";
const C_RST: &str = "\x1b[0m";
const C_BOLD: &str = "\x1b[1m";

pub fn render(results: &OutputResults, color: bool) -> String {
    let findings = &results.results.detectors;
    let mut out = String::new();
    if findings.is_empty() {
        let _ = writeln!(out, "No findings.");
        return out;
    }
    for f in findings {
        let header = format!(
            "{}{}{} [{}/{}] {}",
            if color { color_for_impact(f.impact) } else { "" },
            if color { C_BOLD } else { "" },
            f.check,
            f.impact,
            f.confidence,
            if color { C_RST } else { "" },
        );
        let _ = writeln!(out, "{header}");
        let _ = writeln!(out, "  {}", f.description);
        for e in &f.elements {
            let loc = e.source_mapping.pretty();
            let _ = writeln!(
                out,
                "  {}{}{}",
                if color { C_GRY } else { "" },
                loc,
                if color { C_RST } else { "" }
            );
        }
        let _ = writeln!(out);
    }
    let _ = writeln!(out, "Total findings: {}", findings.len());
    out
}

fn color_for_impact(i: Impact) -> &'static str {
    match i {
        Impact::High => C_RED,
        Impact::Medium => C_YEL,
        Impact::Low => C_BLU,
        Impact::Informational | Impact::Optimization => C_GRY,
    }
}
