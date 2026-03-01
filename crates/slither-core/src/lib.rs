//! slither-core — orchestration & domain model for lancer-slither.
//!
//! This crate is the GUI-agnostic core that both the CLI and the eframe GUI
//! call into. It defines:
//!
//! * The Solidity domain model (`Contract`, `Function`, `Variable`, …).
//! * The `Finding` record + impact/confidence enums.
//! * Project discovery (delegates to [`crytic_compile_rs`]).
//! * Output emitters (JSON / SARIF / Markdown / text / triage / zip).
//! * Suppression annotation parsing (`slither-disable-*`).
//! * The `Slither` analysis context shared with all detectors & printers.

#![warn(clippy::pedantic, missing_debug_implementations)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::if_not_else,
    clippy::needless_pass_by_value,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown
)]

pub mod ast;
pub mod config;
pub mod finding;
pub mod model;
pub mod output;
pub mod project;
pub mod slither;
pub mod source;
pub mod suppression;
pub mod triage;

pub use config::SlitherConfig;
pub use finding::{Confidence, Finding, FindingElement, FindingElementKind, Impact, OutputResults};
pub use model::{
    Contract, Function, FunctionKind, Modifier, Parameter, SolidityVersion, SourceUnit,
    StateMutability, StateVariable, Variable, Visibility,
};
pub use slither::Slither;
pub use source::SourceLocation;
pub use suppression::{SuppressionMap, SuppressionScope};
