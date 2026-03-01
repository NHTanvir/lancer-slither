//! Compare two `OutputResults` for regression CI.
//!
//! Output is a list of `added` (new findings) and `removed` (gone since
//! baseline). Used by `lancer-slither --cross-check baseline.json`.

use crate::finding::{Finding, OutputResults};

#[derive(Debug)]
pub struct Diff {
    pub added: Vec<Finding>,
    pub removed: Vec<Finding>,
    pub same: usize,
}

pub fn diff(old: &OutputResults, new: &OutputResults) -> Diff {
    use ahash::AHashSet;
    let old_ids: AHashSet<String> = old.results.detectors.iter().map(|f| f.id.clone()).collect();
    let new_ids: AHashSet<String> = new.results.detectors.iter().map(|f| f.id.clone()).collect();
    let added: Vec<Finding> = new
        .results
        .detectors
        .iter()
        .filter(|f| !old_ids.contains(&f.id))
        .cloned()
        .collect();
    let removed: Vec<Finding> = old
        .results
        .detectors
        .iter()
        .filter(|f| !new_ids.contains(&f.id))
        .cloned()
        .collect();
    let same = old_ids.intersection(&new_ids).count();
    Diff { added, removed, same }
}
