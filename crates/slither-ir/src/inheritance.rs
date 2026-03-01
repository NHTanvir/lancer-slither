//! Contract inheritance DAG with C3 linearization.

use std::collections::BTreeMap;

use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
pub struct InheritanceGraph {
    pub graph: DiGraph<String, ()>,
    pub by_name: BTreeMap<String, NodeIndex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearizedOrder {
    pub contract: String,
    /// C3-linearized base contracts in resolution order (head = closest base).
    pub linearization: Vec<String>,
}

impl InheritanceGraph {
    pub fn add(&mut self, name: &str, parents: &[String]) {
        let node = *self.by_name.entry(name.to_owned()).or_insert_with(|| {
            self.graph.add_node(name.to_owned())
        });
        for p in parents {
            let pn = *self
                .by_name
                .entry(p.clone())
                .or_insert_with(|| self.graph.add_node(p.clone()));
            self.graph.add_edge(node, pn, ());
        }
    }

    /// C3 linearization. Returns `None` if a linearization cannot be derived
    /// (e.g. due to an irreconcilable inheritance order — solc rejects these).
    pub fn c3_linearize(&self, name: &str) -> Option<Vec<String>> {
        let node = *self.by_name.get(name)?;
        let parents: Vec<String> = self
            .graph
            .neighbors(node)
            .map(|n| self.graph[n].clone())
            .collect();
        if parents.is_empty() {
            return Some(vec![name.to_owned()]);
        }
        let parent_lins: Vec<Vec<String>> = parents
            .iter()
            .map(|p| self.c3_linearize(p).unwrap_or_else(|| vec![p.clone()]))
            .collect();
        let mut lists: Vec<Vec<String>> = parent_lins.clone();
        lists.push(parents.clone());
        let mut result = vec![name.to_owned()];
        loop {
            lists.retain(|l| !l.is_empty());
            if lists.is_empty() {
                break;
            }
            let mut chosen: Option<String> = None;
            for list in &lists {
                let head = &list[0];
                let is_in_tail = lists.iter().any(|other| other[1..].contains(head));
                if !is_in_tail {
                    chosen = Some(head.clone());
                    break;
                }
            }
            let head = chosen?;
            result.push(head.clone());
            for list in lists.iter_mut() {
                if list.first() == Some(&head) {
                    list.remove(0);
                }
            }
        }
        Some(result)
    }
}
