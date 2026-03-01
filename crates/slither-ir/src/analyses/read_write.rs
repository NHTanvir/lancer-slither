//! Compatibility re-export — implementations moved to `read_var.rs` and
//! `write_var.rs` to match the plan layout.

use ahash::AHashSet;

use crate::operations::{Op, VariableId};

#[derive(Debug, Default, Clone)]
pub struct ReadWriteSet {
    pub reads: AHashSet<VariableId>,
    pub writes: AHashSet<VariableId>,
}

pub fn collect(ops: &[Op]) -> ReadWriteSet {
    let r = super::read_var::collect(ops);
    let w = super::write_var::collect(ops);
    ReadWriteSet { reads: r.reads, writes: w.writes }
}
