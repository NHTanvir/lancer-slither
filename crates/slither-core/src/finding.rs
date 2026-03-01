//! `Finding` — what every detector produces.
//!
//! Field shape matches upstream Slither's `--json` output so that downstream
//! consumers (CI tools, GitHub Action SARIF uploader, custom triage scripts)
//! see byte-identical records.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::source::SourceLocation;

/// Severity tier of a finding. Order: High > Medium > Low > Informational > Optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum Impact {
    /// Critical, exploit-grade issue.
    High = 4,
    /// Important issue; usually exploitable under conditions.
    Medium = 3,
    /// Minor issue or hardening.
    Low = 2,
    /// Code-quality / style flag.
    Informational = 1,
    /// Gas / size optimization opportunity.
    Optimization = 0,
}

impl Impact {
    pub fn cli_token(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Informational => "informational",
            Self::Optimization => "optimization",
        }
    }

    pub fn from_cli(token: &str) -> Option<Self> {
        Some(match token.to_ascii_lowercase().as_str() {
            "high" => Self::High,
            "medium" => Self::Medium,
            "low" => Self::Low,
            "informational" | "info" => Self::Informational,
            "optimization" | "opt" => Self::Optimization,
            _ => return None,
        })
    }
}

impl fmt::Display for Impact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
            Self::Informational => "Informational",
            Self::Optimization => "Optimization",
        })
    }
}

/// Confidence tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum Confidence {
    High = 2,
    Medium = 1,
    Low = 0,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        })
    }
}

/// What kind of Solidity entity a finding element refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FindingElementKind {
    Function,
    Contract,
    Variable,
    Modifier,
    Node,
    Expression,
    Pragma,
    Import,
    Enum,
    Struct,
    Event,
    Other,
}

/// One element of a finding: usually the offending function/contract/variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingElement {
    #[serde(rename = "type")]
    pub kind: FindingElementKind,
    pub name: String,
    pub source_mapping: SourceLocation,
    /// e.g. `{"contract":"Vault","function":"withdraw"}`.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub type_specific_fields: serde_json::Map<String, serde_json::Value>,
    /// Anything additional, free-form.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub additional_fields: serde_json::Map<String, serde_json::Value>,
}

impl FindingElement {
    pub fn contract(name: impl Into<String>, loc: SourceLocation) -> Self {
        Self {
            kind: FindingElementKind::Contract,
            name: name.into(),
            source_mapping: loc,
            type_specific_fields: serde_json::Map::new(),
            additional_fields: serde_json::Map::new(),
        }
    }
    pub fn function(
        contract: impl Into<String>,
        name: impl Into<String>,
        loc: SourceLocation,
    ) -> Self {
        let mut map = serde_json::Map::new();
        map.insert(
            "parent".to_owned(),
            serde_json::json!({"type":"contract","name": contract.into()}),
        );
        map.insert(
            "signature".to_owned(),
            serde_json::Value::String(name.into()),
        );
        Self {
            kind: FindingElementKind::Function,
            name: String::new(),
            source_mapping: loc,
            type_specific_fields: map,
            additional_fields: serde_json::Map::new(),
        }
    }
    pub fn variable(name: impl Into<String>, loc: SourceLocation) -> Self {
        Self {
            kind: FindingElementKind::Variable,
            name: name.into(),
            source_mapping: loc,
            type_specific_fields: serde_json::Map::new(),
            additional_fields: serde_json::Map::new(),
        }
    }
    pub fn node(loc: SourceLocation) -> Self {
        Self {
            kind: FindingElementKind::Node,
            name: String::new(),
            source_mapping: loc,
            type_specific_fields: serde_json::Map::new(),
            additional_fields: serde_json::Map::new(),
        }
    }
}

/// One Slither finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Stable identifier (CLI argument), e.g. `"reentrancy-eth"`.
    pub check: String,
    pub impact: Impact,
    pub confidence: Confidence,
    /// Free-form human description.
    pub description: String,
    /// Same content as `description` but with Markdown links to source.
    pub markdown: String,
    /// One-line opener (first sentence of the markdown).
    pub first_markdown_element: String,
    /// Hash-based unique ID across runs.
    pub id: String,
    /// Source elements (function, contract, variable, node).
    pub elements: Vec<FindingElement>,
    /// Free-form additional structured data.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub additional_fields: serde_json::Map<String, serde_json::Value>,
}

impl Finding {
    /// Compute a stable deterministic ID from the finding fields.
    pub fn compute_id(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.check.hash(&mut hasher);
        self.description.hash(&mut hasher);
        for e in &self.elements {
            e.name.hash(&mut hasher);
            e.source_mapping.filename_relative.hash(&mut hasher);
            e.source_mapping.start.hash(&mut hasher);
            e.source_mapping.length.hash(&mut hasher);
        }
        self.id = format!("{:016x}", hasher.finish());
    }
}

/// Aggregate output of a run.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OutputResults {
    pub success: bool,
    pub error: Option<String>,
    pub results: ResultsBucket,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResultsBucket {
    #[serde(default)]
    pub detectors: Vec<Finding>,
    #[serde(default)]
    pub printers: Vec<PrinterOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterOutput {
    pub printer: String,
    pub description: String,
    #[serde(default)]
    pub elements: serde_json::Value,
}
