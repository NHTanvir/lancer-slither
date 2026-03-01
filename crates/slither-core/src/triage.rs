//! Compatibility shim — the implementation moved to [`crate::output::triage`]
//! so the on-disk layout matches `plan.md`.

pub use crate::output::triage::{TriageDb, TriageDecision};
