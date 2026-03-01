//! `--print modifiers` — list modifiers and where they're applied.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct Modifiers;
impl Printer for Modifiers {
    fn argument(&self) -> &'static str { "modifiers" }
    fn help(&self) -> &'static str { "List modifiers + functions they gate" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for c in slither.all_contracts() {
            writeln!(out, "Contract {}", c.name)?;
            for m in &c.modifiers {
                writeln!(out, "  modifier {}", m.name)?;
                for f in &c.functions {
                    if f.modifiers.iter().any(|x| x == &m.name) {
                        writeln!(out, "    used by {}.{}", c.name, f.name)?;
                    }
                }
            }
            writeln!(out)?;
        }
        Ok(())
    }
}
