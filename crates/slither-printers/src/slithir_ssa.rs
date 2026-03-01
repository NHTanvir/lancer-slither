//! `--print slithir-ssa` — dump SSA form.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct SlithIrSsa;
impl Printer for SlithIrSsa {
    fn argument(&self) -> &'static str { "slithir-ssa" }
    fn help(&self) -> &'static str { "Dump SSA-form SlithIR" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for (c, f) in slither.all_functions() {
            let ir = slither_ir::convert::lift_function(f);
            writeln!(out, "{}.{} (SSA)", c.name, f.signature())?;
            for (id, def) in ir.ssa.last_def.iter() {
                writeln!(out, "  {:?} v{}", id, def)?;
            }
            for op in &ir.ops {
                writeln!(out, "  {:?}", op)?;
            }
            writeln!(out)?;
        }
        Ok(())
    }
}
