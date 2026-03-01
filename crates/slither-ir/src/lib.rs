//! `slither-ir` — SlithIR three-address representation.
//!
//! ### Architecture
//!
//! Each function in the `slither-core` domain model is lifted into a
//! [`FunctionIr`] holding:
//!
//! * A sequence of [`operations::Op`] entries in three-address form.
//! * A control-flow graph built over IR ops (`petgraph::DiGraph`).
//! * An SSA form populated by [`ssa::convert_to_ssa`].
//! * Per-variable dataflow lattices in [`analyses::data_dependency`].

#![warn(missing_debug_implementations)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::if_not_else
)]

pub mod analyses;
pub mod callgraph;
pub mod cfg;
pub mod convert;
pub mod dominator;
pub mod inheritance;
pub mod operations;
pub mod ssa;
pub mod types;

use indexmap::IndexMap;

pub use operations::{Op, Variable, VariableId};
pub use types::Type;

/// IR for one Solidity function — opcodes + CFG + SSA + per-fn analyses.
#[derive(Debug, Default, Clone)]
pub struct FunctionIr {
    pub function_full_name: String,
    pub ops: Vec<Op>,
    pub variables: IndexMap<VariableId, Variable>,
    pub blocks: Vec<cfg::BasicBlock>,
    pub cfg: cfg::Cfg,
    pub ssa: ssa::SsaInfo,
    pub reads: ahash::AHashSet<VariableId>,
    pub writes: ahash::AHashSet<VariableId>,
    pub external_calls: Vec<operations::CallSite>,
    pub low_level_calls: Vec<operations::CallSite>,
    pub state_writes: Vec<operations::StateWrite>,
}

impl FunctionIr {
    pub fn new(full_name: impl Into<String>) -> Self {
        Self {
            function_full_name: full_name.into(),
            ..Self::default()
        }
    }
}
