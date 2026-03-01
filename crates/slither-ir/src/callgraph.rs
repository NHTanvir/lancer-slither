//! Cross-contract call graph.

use std::collections::BTreeMap;

use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};

/// Edge type — important for downstream analysis like reentrancy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EdgeKind {
    HighLevel,
    Internal,
    Library,
    LowLevel,
    Super,
    Solidity,
}

#[derive(Debug, Default, Clone)]
pub struct CallGraph {
    pub graph: DiGraph<String, EdgeKind>,
    pub by_full_name: BTreeMap<String, NodeIndex>,
}

impl CallGraph {
    pub fn add_edge(&mut self, from: &str, to: &str, kind: EdgeKind) {
        let a = *self
            .by_full_name
            .entry(from.to_owned())
            .or_insert_with(|| self.graph.add_node(from.to_owned()));
        let b = *self
            .by_full_name
            .entry(to.to_owned())
            .or_insert_with(|| self.graph.add_node(to.to_owned()));
        self.graph.add_edge(a, b, kind);
    }

    pub fn paths(&self, from: &str, to: &str, max: usize) -> Vec<Vec<String>> {
        let Some(&start) = self.by_full_name.get(from) else {
            return Vec::new();
        };
        let Some(&end) = self.by_full_name.get(to) else {
            return Vec::new();
        };
        let mut paths = Vec::new();
        let mut stack = vec![(start, vec![start])];
        while let Some((node, path)) = stack.pop() {
            if node == end {
                paths.push(path.iter().map(|n| self.graph[*n].clone()).collect());
                if paths.len() >= max {
                    break;
                }
                continue;
            }
            for nb in self.graph.neighbors(node) {
                if !path.contains(&nb) {
                    let mut np = path.clone();
                    np.push(nb);
                    stack.push((nb, np));
                }
            }
        }
        paths
    }
}
