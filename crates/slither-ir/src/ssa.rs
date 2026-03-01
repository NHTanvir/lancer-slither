//! SSA form for SlithIR.
//!
//! Implements a streamlined Cytron-Ferrante: compute dominators, place Phi
//! nodes at iterated dominance frontiers, then rename variables.
//!
//! The current implementation is a scaffold returning per-variable last
//! definitions; full Phi placement is a follow-up.

use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::operations::VariableId;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SsaInfo {
    /// Latest version index per variable.
    pub last_def: AHashMap<VariableId, u32>,
    /// Per-block phi nodes (block_id → list of phi'd variables).
    pub phis: AHashMap<u32, Vec<VariableId>>,
}

impl SsaInfo {
    pub fn record_def(&mut self, var: VariableId) -> u32 {
        let v = self.last_def.entry(var).or_insert(0);
        *v += 1;
        *v
    }
}
