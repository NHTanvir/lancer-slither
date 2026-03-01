//! Output emitters.
//!
//! Each emitter takes an [`OutputResults`] + a sink and writes the desired
//! representation. The CLI dispatches to whichever emitters were enabled by
//! `--json`, `--sarif`, `--markdown`, `--checklist`, `--zip`.

pub mod cross_check;
pub mod json;
pub mod markdown;
pub mod sarif;
pub mod text;
pub mod triage;
pub mod zip;
