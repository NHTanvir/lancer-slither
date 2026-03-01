//! Per-IR-node read sets.
//!
//! Used by the reentrancy detector family: a node that reads a state variable
//! after an external call is the reentrancy "read-after-call" signal.

use ahash::AHashSet;

use crate::operations::{Op, VariableId};

#[derive(Debug, Default, Clone)]
pub struct ReadSet {
    pub reads: AHashSet<VariableId>,
}

pub fn collect(ops: &[Op]) -> ReadSet {
    let mut set = ReadSet::default();
    for op in ops {
        match op {
            Op::Binary { lhs, rhs, .. } => {
                set.reads.insert(*lhs);
                set.reads.insert(*rhs);
            }
            Op::Unary { operand, .. } => {
                set.reads.insert(*operand);
            }
            Op::Index { base, index, .. } => {
                set.reads.insert(*base);
                set.reads.insert(*index);
            }
            Op::Member { base, .. } => {
                set.reads.insert(*base);
            }
            Op::Assignment { src, .. } | Op::TypeConversion { src, .. } => {
                set.reads.insert(*src);
            }
            Op::Push { array, value } => {
                set.reads.insert(*array);
                set.reads.insert(*value);
            }
            Op::Pop { array } => {
                set.reads.insert(*array);
            }
            Op::Length { target, .. }
            | Op::Balance { target, .. }
            | Op::CodeSize { target, .. } => {
                set.reads.insert(*target);
            }
            Op::Send { target, amount } | Op::Transfer { target, amount } => {
                set.reads.insert(*target);
                set.reads.insert(*amount);
            }
            Op::Call(c)
            | Op::LowLevelCall(c)
            | Op::NewContract(c)
            | Op::NewArray(c)
            | Op::NewStructure(c) => {
                for a in &c.arguments {
                    set.reads.insert(*a);
                }
            }
            Op::EventCall { args, .. } | Op::Return { values: args } => {
                for a in args {
                    set.reads.insert(*a);
                }
            }
            Op::InitArray { values, .. } => {
                for v in values {
                    set.reads.insert(*v);
                }
            }
            Op::Phi { operands, .. } => {
                for v in operands {
                    set.reads.insert(*v);
                }
            }
            Op::Condition { var } => {
                set.reads.insert(*var);
            }
            Op::Delete { .. } | Op::Nop => {}
        }
    }
    set
}
