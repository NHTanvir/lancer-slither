//! Per-IR-node write sets.

use ahash::AHashSet;

use crate::operations::{Op, VariableId};

#[derive(Debug, Default, Clone)]
pub struct WriteSet {
    pub writes: AHashSet<VariableId>,
}

pub fn collect(ops: &[Op]) -> WriteSet {
    let mut set = WriteSet::default();
    for op in ops {
        match op {
            Op::Binary { dst, .. }
            | Op::Unary { dst, .. }
            | Op::Index { dst, .. }
            | Op::Member { dst, .. }
            | Op::Assignment { dst, .. }
            | Op::TypeConversion { dst, .. }
            | Op::Length { dst, .. }
            | Op::Balance { dst, .. }
            | Op::CodeSize { dst, .. }
            | Op::InitArray { dst, .. }
            | Op::Phi { dst, .. } => {
                set.writes.insert(*dst);
            }
            Op::Push { array, .. } | Op::Pop { array } => {
                set.writes.insert(*array);
            }
            Op::Delete { target } => {
                set.writes.insert(*target);
            }
            Op::Call(c)
            | Op::LowLevelCall(c)
            | Op::NewContract(c)
            | Op::NewArray(c)
            | Op::NewStructure(c) => {
                for r in &c.returns {
                    set.writes.insert(*r);
                }
            }
            Op::Send { .. }
            | Op::Transfer { .. }
            | Op::EventCall { .. }
            | Op::Return { .. }
            | Op::Condition { .. }
            | Op::Nop => {}
        }
    }
    set
}
