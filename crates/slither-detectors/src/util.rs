//! Common detector machinery — finding builders, body-excerpt scanners,
//! AST visitors. Kept tiny so each detector can stay declarative.

use slither_core::{
    Confidence, Finding, FindingElement, Impact, Slither, SourceLocation, Visibility,
};

/// Build a finding referencing a single contract.
pub fn finding_for_contract(
    check: &str,
    impact: Impact,
    confidence: Confidence,
    description: String,
    contract_name: &str,
    loc: SourceLocation,
) -> Finding {
    let elements = vec![FindingElement::contract(contract_name, loc.clone())];
    new_finding(check, impact, confidence, description, elements)
}

/// Build a finding referencing one function.
pub fn finding_for_function(
    check: &str,
    impact: Impact,
    confidence: Confidence,
    description: String,
    contract: &str,
    function: &str,
    loc: SourceLocation,
) -> Finding {
    let elements = vec![FindingElement::function(contract, function, loc.clone())];
    new_finding(check, impact, confidence, description, elements)
}

/// Build a finding referencing a state variable.
pub fn finding_for_variable(
    check: &str,
    impact: Impact,
    confidence: Confidence,
    description: String,
    variable: &str,
    loc: SourceLocation,
) -> Finding {
    let elements = vec![FindingElement::variable(variable, loc.clone())];
    new_finding(check, impact, confidence, description, elements)
}

pub fn new_finding(
    check: &str,
    impact: Impact,
    confidence: Confidence,
    description: String,
    elements: Vec<FindingElement>,
) -> Finding {
    let first_md = description.lines().next().unwrap_or("").to_owned();
    let mut f = Finding {
        check: check.to_owned(),
        impact,
        confidence,
        description: description.clone(),
        markdown: description,
        first_markdown_element: first_md,
        id: String::new(),
        elements,
        additional_fields: serde_json::Map::new(),
    };
    f.compute_id();
    f
}

/// Filter functions by visibility (used for "external"/"public" only checks).
pub fn is_publicly_callable(v: Visibility) -> bool {
    matches!(v, Visibility::Public | Visibility::External)
}

/// Filter findings by `--filter-paths` regex.
pub fn passes_path_filter(slither: &Slither, loc: &SourceLocation) -> bool {
    let p = loc.filename_relative.to_string_lossy().to_string();
    if let Some(re) = &slither.config.filter_paths {
        if let Ok(rx) = regex::Regex::new(re) {
            if rx.is_match(&p) {
                return false;
            }
        }
    }
    if let Some(re) = &slither.config.include_paths {
        if let Ok(rx) = regex::Regex::new(re) {
            if !rx.is_match(&p) {
                return false;
            }
        }
    }
    true
}

/// Cheap whole-source regex search. Used by mechanical checks like
/// `low-level-calls`, `tx-origin`, `rtlo`, …
pub fn body_contains(body: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| body.contains(*n))
}

/// Word-boundary search to avoid e.g. `pure` matching inside `_pureName`.
pub fn body_contains_word(body: &str, word: &str) -> bool {
    let pattern = regex::Regex::new(&format!(r"\b{}\b", regex::escape(word))).expect("regex");
    pattern.is_match(body)
}
