//! SARIF 2.1.0 emitter (GitHub code-scanning compatible).
//!
//! Written by hand (no `serde_sarif` dependency) — keeps build lean and gives
//! us full control over the result shape.

use std::path::Path;

use serde::Serialize;
use serde_json::{json, Value};

use crate::finding::{Finding, Impact, OutputResults};

#[derive(Serialize)]
struct SarifReport {
    version: String,
    #[serde(rename = "$schema")]
    schema: String,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: Value,
    results: Vec<Value>,
}

fn impact_to_level(i: Impact) -> &'static str {
    match i {
        Impact::High => "error",
        Impact::Medium => "warning",
        Impact::Low => "note",
        Impact::Informational | Impact::Optimization => "none",
    }
}

fn finding_to_result(f: &Finding) -> Value {
    let primary = f.elements.first();
    let physical_location = primary
        .map(|e| {
            json!({
                "artifactLocation": {
                    "uri": e.source_mapping.filename_relative.display().to_string(),
                },
                "region": {
                    "startLine": e.source_mapping.lines.first().copied().unwrap_or(1),
                    "endLine": e.source_mapping.lines.last().copied().unwrap_or(1),
                    "startColumn": e.source_mapping.starting_column,
                    "endColumn": e.source_mapping.ending_column,
                }
            })
        })
        .unwrap_or_else(|| Value::Null);
    json!({
        "ruleId": f.check,
        "level": impact_to_level(f.impact),
        "message": { "text": f.description },
        "locations": [ { "physicalLocation": physical_location } ],
        "partialFingerprints": { "primaryLocationLineHash": f.id },
    })
}

fn rules_payload(findings: &[Finding]) -> Vec<Value> {
    let mut seen = std::collections::BTreeSet::new();
    let mut rules = Vec::new();
    for f in findings {
        if seen.insert(f.check.clone()) {
            rules.push(json!({
                "id": f.check,
                "name": f.check,
                "shortDescription": { "text": f.first_markdown_element },
                "fullDescription": { "text": f.description },
                "defaultConfiguration": { "level": impact_to_level(f.impact) },
            }));
        }
    }
    rules
}

pub fn write(path: &Path, results: &OutputResults) -> eyre::Result<()> {
    let findings = &results.results.detectors;
    let run = SarifRun {
        tool: json!({
            "driver": {
                "name": "lancer-slither",
                "informationUri": "https://github.com/NHTanvir/lancer-slither",
                "version": env!("CARGO_PKG_VERSION"),
                "rules": rules_payload(findings),
            }
        }),
        results: findings.iter().map(finding_to_result).collect(),
    };
    let rep = SarifReport {
        version: "2.1.0".to_owned(),
        schema:
            "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json"
                .to_owned(),
        runs: vec![run],
    };
    let json = serde_json::to_string_pretty(&rep)?;
    std::fs::write(path, json)?;
    Ok(())
}
