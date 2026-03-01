//! Dominator tree (Cooper-Harvey-Kennedy simple algorithm).
//!
//! Used by:
//! * [`crate::ssa`] for Phi-node placement at iterated dominance frontiers.
//! * Reachability analyses ("can this block be reached from `entry`?").

use std::collections::BTreeMap;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Bfs;

#[derive(Debug, Default, Clone)]
pub struct DominatorTree {
    pub idom: BTreeMap<NodeIndex, NodeIndex>,
    pub entry: Option<NodeIndex>,
}

impl DominatorTree {
    pub fn compute<N, E>(graph: &DiGraph<N, E>, entry: NodeIndex) -> Self {
        let mut idom: BTreeMap<NodeIndex, NodeIndex> = BTreeMap::new();
        idom.insert(entry, entry);

        let post_order = post_order(graph, entry);
        let mut changed = true;
        while changed {
            changed = false;
            for &b in post_order.iter().rev() {
                if b == entry {
                    continue;
                }
                let preds: Vec<NodeIndex> = graph
                    .neighbors_directed(b, petgraph::Direction::Incoming)
                    .collect();
                let mut new_idom: Option<NodeIndex> = None;
                for p in &preds {
                    if idom.contains_key(p) {
                        new_idom = match new_idom {
                            None => Some(*p),
                            Some(cur) => Some(intersect(&idom, cur, *p)),
                        };
                    }
                }
                if let Some(d) = new_idom {
                    if idom.get(&b) != Some(&d) {
                        idom.insert(b, d);
                        changed = true;
                    }
                }
            }
        }
        Self { idom, entry: Some(entry) }
    }

    pub fn dominators(&self, n: NodeIndex) -> Vec<NodeIndex> {
        let mut out = Vec::new();
        let mut cur = n;
        while let Some(&parent) = self.idom.get(&cur) {
            out.push(parent);
            if parent == cur {
                break;
            }
            cur = parent;
        }
        out
    }
}

fn intersect(
    idom: &BTreeMap<NodeIndex, NodeIndex>,
    mut a: NodeIndex,
    mut b: NodeIndex,
) -> NodeIndex {
    while a != b {
        while a.index() < b.index() {
            match idom.get(&a) {
                Some(&p) if p != a => a = p,
                _ => break,
            }
        }
        while b.index() < a.index() {
            match idom.get(&b) {
                Some(&p) if p != b => b = p,
                _ => break,
            }
        }
        if a == b {
            break;
        }
        // Defensive guard against pathological graphs.
        let pa = idom.get(&a).copied();
        let pb = idom.get(&b).copied();
        if pa == Some(a) || pb == Some(b) {
            return if a.index() < b.index() { a } else { b };
        }
        if let (Some(pa), Some(pb)) = (pa, pb) {
            if a == pa && b == pb {
                return a;
            }
        }
        break;
    }
    a
}

fn post_order<N, E>(graph: &DiGraph<N, E>, entry: NodeIndex) -> Vec<NodeIndex> {
    let mut out = Vec::new();
    let mut visited: std::collections::HashSet<NodeIndex> = std::collections::HashSet::new();
    let mut bfs = Bfs::new(graph, entry);
    while let Some(n) = bfs.next(graph) {
        if visited.insert(n) {
            out.push(n);
        }
    }
    out.reverse();
    out
}
