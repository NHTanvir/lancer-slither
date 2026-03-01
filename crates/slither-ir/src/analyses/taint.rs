//! Forward/backward taint propagation over IR.

use ahash::AHashSet;

use crate::analyses::data_dependency::DataDependency;
use crate::operations::VariableId;

#[derive(Debug, Default, Clone)]
pub struct Taint {
    pub tainted: AHashSet<VariableId>,
}

impl Taint {
    pub fn forward(&mut self, sources: &[VariableId], deps: &DataDependency) {
        for s in sources {
            self.tainted.insert(*s);
        }
        loop {
            let mut changed = false;
            let snapshot: Vec<_> = self.tainted.iter().copied().collect();
            for (dst, srcs) in deps.deps.iter() {
                if srcs.iter().any(|s| snapshot.contains(s))
                    && self.tainted.insert(*dst)
                {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    pub fn is_tainted(&self, v: VariableId) -> bool {
        self.tainted.contains(&v)
    }
}
