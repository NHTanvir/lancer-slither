//! `--print inheritance` / `--print inheritance-graph` — DOT output.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct Inheritance;
impl Printer for Inheritance {
    fn argument(&self) -> &'static str { "inheritance" }
    fn help(&self) -> &'static str { "Contract inheritance hierarchy as DOT" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        writeln!(out, "digraph inheritance {{")?;
        writeln!(out, "  rankdir=BT")?;
        for c in slither.all_contracts() {
            for parent in &c.inheritance {
                writeln!(out, "  \"{}\" -> \"{}\";", c.name, parent)?;
            }
        }
        writeln!(out, "}}")?;
        Ok(())
    }
}

pub struct InheritanceGraph;
impl Printer for InheritanceGraph {
    fn argument(&self) -> &'static str { "inheritance-graph" }
    fn help(&self) -> &'static str { "Inheritance + function-level edges as DOT" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        writeln!(out, "digraph inheritanceGraph {{")?;
        writeln!(out, "  rankdir=BT")?;
        for c in slither.all_contracts() {
            let funcs: Vec<&str> = c.functions.iter().map(|f| f.name.as_str()).collect();
            writeln!(
                out,
                "  \"{}\" [label=\"{}|{}\" shape=record];",
                c.name,
                c.name,
                funcs.join("\\l")
            )?;
            for parent in &c.inheritance {
                writeln!(out, "  \"{}\" -> \"{}\";", c.name, parent)?;
            }
        }
        writeln!(out, "}}")?;
        Ok(())
    }
}
