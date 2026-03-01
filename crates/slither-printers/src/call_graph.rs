//! `--print call-graph` — DOT.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct CallGraph;
impl Printer for CallGraph {
    fn argument(&self) -> &'static str { "call-graph" }
    fn help(&self) -> &'static str { "Cross-contract call graph as DOT" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        writeln!(out, "digraph callGraph {{")?;
        writeln!(out, "  node [shape=box];")?;
        let names: ahash::AHashSet<String> = slither
            .all_functions()
            .map(|(c, f)| format!("{}.{}", c.name, f.name))
            .collect();
        for (c, f) in slither.all_functions() {
            let src = format!("{}.{}", c.name, f.name);
            for callee in &names {
                let target = callee.rsplit('.').next().unwrap_or("");
                let re = regex::Regex::new(&format!(r"\b{}\s*\(", regex::escape(target))).unwrap();
                if &src != callee && re.is_match(&f.body_excerpt) {
                    writeln!(out, "  \"{}\" -> \"{}\";", src, callee)?;
                }
            }
        }
        writeln!(out, "}}")?;
        Ok(())
    }
}
