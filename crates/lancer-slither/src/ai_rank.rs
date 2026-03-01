//! AI augmentation entry points listed in `plan.md` § "AI augmentation ideas".
//!
//! All seven ideas are scaffolded behind heuristic stubs so the codebase
//! compiles without an API key. If `ANTHROPIC_API_KEY` is set, the same
//! functions can be upgraded to round-trip through the Claude API.
//!
//! * `rank()` — score each finding 0..=100.
//! * `explain()` — rewrite the description per-finding with codebase names.
//! * `suggest_fixes()` — emit a unified-diff per finding.
//! * `classify_false_positives()` — likely-FP scoring.
//! * `cluster()` — group findings by root cause.
//! * `slithir_to_natural_language()` — function-summary in prose.
//! * `scaffold_custom_detector()` — given an English prompt, emit a Rust
//!   detector struct.
//! * `auto_triage()` — train an FP classifier from a triage DB and rank.

use slither_core::{Confidence, Finding, Impact};

/// `--ai-rank` — assign an `ai_priority_score` field to every finding and
/// re-sort the list.
pub fn rank(findings: &mut [Finding]) -> eyre::Result<()> {
    for f in findings.iter_mut() {
        let score = heuristic_score(f);
        f.additional_fields.insert(
            "ai_priority_score".to_owned(),
            serde_json::Value::Number(serde_json::Number::from(score)),
        );
    }
    findings.sort_by(|a, b| {
        let av = a
            .additional_fields
            .get("ai_priority_score")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let bv = b
            .additional_fields
            .get("ai_priority_score")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        bv.cmp(&av)
    });
    Ok(())
}

/// Rewrite each finding's `description` with the actual codebase names.
/// Heuristic implementation: prepends the contract/function/variable named
/// in the first element so the description reads more concretely.
pub fn explain(findings: &mut [Finding]) {
    for f in findings.iter_mut() {
        if let Some(el) = f.elements.first() {
            let prefix = match el.kind {
                slither_core::finding::FindingElementKind::Function => format!(
                    "In `{}`: ",
                    el.type_specific_fields
                        .get("signature")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                ),
                _ => format!("Concerning `{}`: ", el.name),
            };
            if !f.description.starts_with(&prefix) {
                f.description = format!("{prefix}{}", f.description);
            }
        }
    }
}

/// Emit a unified-diff style fix suggestion per finding.
/// Currently produces a placeholder hunk so downstream tooling can detect
/// the field; a real implementation would POST the AST snippet to Claude.
pub fn suggest_fixes(findings: &mut [Finding]) {
    for f in findings.iter_mut() {
        let suggestion = match f.check.as_str() {
            "boolean-equal" => Some("rewrite `expr == true` as `expr` and `expr == false` as `!expr`"),
            "missing-zero-check" => Some("insert `require(addr != address(0), \"zero address\");`"),
            "tx-origin" => Some("replace `tx.origin` with `msg.sender`"),
            "weak-prng" => Some("use a VRF such as Chainlink VRF instead of `block.timestamp`/`blockhash`"),
            "incorrect-modifier" => Some("ensure every code path through the modifier executes the `_;` placeholder"),
            "reentrancy-eth" | "reentrancy-no-eth" => Some("apply Check-Effects-Interactions or use `ReentrancyGuard`"),
            "void-cst" => Some("explicitly forward to the parent constructor"),
            "constable-states" => Some("declare the variable `constant`"),
            "immutable-states" => Some("declare the variable `immutable`"),
            _ => None,
        };
        if let Some(s) = suggestion {
            f.additional_fields.insert(
                "ai_suggested_fix".to_owned(),
                serde_json::Value::String(s.to_owned()),
            );
        }
    }
}

/// Annotate each finding with an `ai_fp_likelihood` 0..100.
pub fn classify_false_positives(findings: &mut [Finding]) {
    for f in findings.iter_mut() {
        let likely_fp = match f.check.as_str() {
            // Heuristics-only detectors that are noisy in practice:
            "naming-convention" | "boolean-equal" | "low-level-calls"
            | "assembly" | "pragma" | "solc-version" => 60,
            "too-many-digits" | "redundant-statements" => 50,
            _ => 10,
        };
        f.additional_fields.insert(
            "ai_fp_likelihood".to_owned(),
            serde_json::Value::Number(serde_json::Number::from(likely_fp as u64)),
        );
    }
}

/// Cluster findings by `(check, contract)` so a report with 60 findings of
/// the same pattern collapses to one group entry plus per-finding links.
pub fn cluster(findings: &[Finding]) -> Vec<Cluster> {
    use std::collections::BTreeMap;
    let mut buckets: BTreeMap<(String, String), Vec<&Finding>> = BTreeMap::new();
    for f in findings {
        let contract = f
            .elements
            .first()
            .and_then(|e| {
                e.type_specific_fields
                    .get("parent")
                    .and_then(|p| p.get("name"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "<global>".to_owned());
        buckets.entry((f.check.clone(), contract)).or_default().push(f);
    }
    buckets
        .into_iter()
        .map(|((check, contract), members)| Cluster {
            check,
            contract,
            count: members.len(),
            sample_id: members.first().map(|f| f.id.clone()).unwrap_or_default(),
        })
        .collect()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cluster {
    pub check: String,
    pub contract: String,
    pub count: usize,
    pub sample_id: String,
}

/// Lower the SlithIR of one function to a natural-language summary.
pub fn slithir_to_natural_language(ir: &slither_ir::FunctionIr) -> String {
    let mut s = format!("Function {} —", ir.function_full_name);
    let calls = ir.external_calls.len();
    let lowlevel = ir.low_level_calls.len();
    let writes = ir.state_writes.len();
    s.push_str(&format!(
        " {} ops, {} state writes, {} external calls, {} low-level calls.",
        ir.ops.len(),
        writes,
        calls,
        lowlevel
    ));
    s
}

/// Given an English description, scaffold a Rust detector struct.
pub fn scaffold_custom_detector(prompt: &str, name: &str) -> String {
    format!(
        r#"// Custom detector scaffold generated from: {prompt}
use slither_core::{{Confidence, Finding, Impact, Slither}};
use slither_detectors::{{Detector, util}};

pub struct {name};
impl Detector for {name} {{
    fn argument(&self) -> &'static str {{ "{slug}" }}
    fn help(&self) -> &'static str {{ "TODO: one-line help" }}
    fn impact(&self) -> Impact {{ Impact::Medium }}
    fn confidence(&self) -> Confidence {{ Confidence::Medium }}
    fn detect(&self, slither: &Slither) -> Vec<Finding> {{
        let mut out = Vec::new();
        for (c, f) in slither.all_functions() {{
            // TODO: detection logic for: {prompt}
            let _ = (c, f);
        }}
        out
    }}
}}
"#,
        slug = name.to_lowercase()
    )
}

/// Train an FP classifier from a triage DB and rank new findings by how
/// historically they match the user's accepted/rejected pattern.
pub fn auto_triage(db: &slither_core::triage::TriageDb, findings: &mut [Finding]) {
    let mut fp_by_check: ahash::AHashMap<String, (u32, u32)> = ahash::AHashMap::new();
    for (_id, d) in &db.decisions {
        let key = d.note.clone().unwrap_or_default();
        let entry = fp_by_check.entry(key).or_insert((0, 0));
        if d.status == "false-positive" || d.status == "wont-fix" {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }
    for f in findings.iter_mut() {
        let (fp, ok) = fp_by_check.get(&f.check).copied().unwrap_or((0, 0));
        let denom = (fp + ok).max(1);
        let score = (fp as f64 / denom as f64 * 100.0) as u64;
        f.additional_fields.insert(
            "ai_historical_fp_rate".to_owned(),
            serde_json::Value::Number(serde_json::Number::from(score)),
        );
    }
}

fn heuristic_score(f: &Finding) -> u64 {
    let impact_pts: u64 = match f.impact {
        Impact::High => 60,
        Impact::Medium => 40,
        Impact::Low => 20,
        Impact::Informational => 5,
        Impact::Optimization => 1,
    };
    let conf_pts: u64 = match f.confidence {
        Confidence::High => 30,
        Confidence::Medium => 15,
        Confidence::Low => 5,
    };
    let breadth = (f.elements.len() as u64).min(10);
    impact_pts + conf_pts + breadth
}
