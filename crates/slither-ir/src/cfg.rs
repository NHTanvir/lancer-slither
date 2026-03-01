//! Control-flow graph over IR ops.

use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};

/// One basic block — a maximal straight-line sequence of IR ops.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    pub id: u32,
    /// Indices into `FunctionIr::ops`.
    pub op_indices: Vec<u32>,
    /// Successor block IDs.
    pub successors: Vec<u32>,
}

#[derive(Debug, Default, Clone)]
pub struct Cfg {
    pub graph: DiGraph<u32, ()>,
    pub entry: Option<NodeIndex>,
    pub exit: Vec<NodeIndex>,
}

impl Cfg {
    pub fn add_block(&mut self, id: u32) -> NodeIndex {
        self.graph.add_node(id)
    }
    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, ());
    }
}
