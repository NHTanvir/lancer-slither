//! `slither-triage.json` — the per-project triage database.
//!
//! Driven by `--triage-mode`. Each finding is presented to the auditor for
//! review and the decision is recorded by `id` so subsequent runs can skip
//! `false-positive` / `wont-fix` findings automatically.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// A single triage decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageDecision {
    /// One of `valid`, `false-positive`, `wont-fix`.
    pub status: String,
    pub note: Option<String>,
    pub recorded_at: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TriageDb {
    /// Keyed by finding ID.
    pub decisions: BTreeMap<String, TriageDecision>,
}

impl TriageDb {
    pub fn load(path: &Path) -> eyre::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let db: Self = serde_json::from_str(&raw).unwrap_or_default();
        Ok(db)
    }
    pub fn save(&self, path: &Path) -> eyre::Result<()> {
        let raw = serde_json::to_string_pretty(self)?;
        std::fs::write(path, raw)?;
        Ok(())
    }
    pub fn is_false_positive(&self, finding_id: &str) -> bool {
        self.decisions
            .get(finding_id)
            .map(|d| d.status == "false-positive" || d.status == "wont-fix")
            .unwrap_or(false)
    }
}
