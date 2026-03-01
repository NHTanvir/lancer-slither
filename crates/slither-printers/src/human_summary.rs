//! `--print human-summary` — terse per-contract overview.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct HumanSummary;
impl Printer for HumanSummary {
    fn argument(&self) -> &'static str { "human-summary" }
    fn help(&self) -> &'static str { "Per-contract overview" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for c in slither.all_contracts() {
            writeln!(out, "Contract {}", c.name)?;
            writeln!(out, "  source: {}", c.source.pretty())?;
            if !c.inheritance.is_empty() {
                writeln!(out, "  inheritance: {}", c.inheritance.join(", "))?;
            }
            writeln!(out, "  state vars: {}", c.state_variables.len())?;
            writeln!(out, "  functions: {}", c.functions.len())?;
            writeln!(out, "  modifiers: {}", c.modifiers.len())?;
            writeln!(out)?;
        }
        Ok(())
    }
}
