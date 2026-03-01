//! `--print contract-summary` — slightly more verbose than human-summary.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct ContractSummary;
impl Printer for ContractSummary {
    fn argument(&self) -> &'static str { "contract-summary" }
    fn help(&self) -> &'static str { "Per-contract terse printout" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for c in slither.all_contracts() {
            writeln!(out, "Contract: {}", c.name)?;
            for f in &c.functions {
                writeln!(out, "  fn {} {:?} {:?}", f.signature(), f.visibility, f.mutability)?;
            }
            for sv in &c.state_variables {
                writeln!(out, "  var {} {:?} {} const={} immut={}", sv.name, sv.visibility, sv.type_str, sv.is_constant, sv.is_immutable)?;
            }
            writeln!(out)?;
        }
        Ok(())
    }
}
