//! Per-variable dataflow lattice.
//!
//! For each variable, accumulates the set of variables it transitively reads
//! from. Used by `arbitrary-send-eth`, `controlled-delegatecall`, `tx-origin`,
//! `unprotected-upgrade`, etc.

use ahash::AHashMap;
use ahash::AHashSet;

use crate::operations::{Op, VariableId};

#[derive(Debug, Default, Clone)]
pub struct DataDependency {
    pub deps: AHashMap<VariableId, AHashSet<VariableId>>,
}

impl DataDependency {
    /// One pass over IR ops; fixed point handled in [`compute_to_fixed_point`].
    pub fn step(&mut self, ops: &[Op]) {
        for op in ops {
            match op {
                Op::Binary { dst, lhs, rhs, .. } => {
                    self.add(*dst, *lhs);
                    self.add(*dst, *rhs);
                }
                Op::Unary { dst, operand, .. } => {
                    self.add(*dst, *operand);
                }
                Op::Index { dst, base, index } => {
                    self.add(*dst, *base);
                    self.add(*dst, *index);
                }
                Op::Member { dst, base, .. } => {
                    self.add(*dst, *base);
                }
                Op::Assignment { dst, src } | Op::TypeConversion { dst, src, .. } => {
                    self.add(*dst, *src);
                }
                Op::Length { dst, target }
                | Op::Balance { dst, target }
                | Op::CodeSize { dst, target } => {
                    self.add(*dst, *target);
                }
                Op::InitArray { dst, values } => {
                    for v in values {
                        self.add(*dst, *v);
                    }
                }
                Op::Phi { dst, operands } => {
                    for o in operands {
                        self.add(*dst, *o);
                    }
                }
                _ => {}
            }
        }
    }

    fn add(&mut self, dst: VariableId, src: VariableId) {
        self.deps.entry(dst).or_default().insert(src);
    }

    pub fn compute_to_fixed_point(&mut self) {
        loop {
            let mut changed = false;
            let snapshot: Vec<(VariableId, Vec<VariableId>)> = self
                .deps
                .iter()
                .map(|(k, v)| (*k, v.iter().copied().collect()))
                .collect();
            for (dst, srcs) in snapshot {
                for src in srcs {
                    let extras: Vec<VariableId> = self
                        .deps
                        .get(&src)
                        .map(|s| s.iter().copied().collect())
                        .unwrap_or_default();
                    let entry = self.deps.entry(dst).or_default();
                    for e in extras {
                        if entry.insert(e) {
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    pub fn depends_on(&self, dst: VariableId, src: VariableId) -> bool {
        self.deps.get(&dst).map(|s| s.contains(&src)).unwrap_or(false)
    }
}
