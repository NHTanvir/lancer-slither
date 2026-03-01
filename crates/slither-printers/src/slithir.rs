//! `--print slithir` — dump SlithIR for each function.

use std::io::Write;

use slither_core::Slither;
use slither_ir::convert::lift_function;

use crate::Printer;

pub struct SlithIr;
impl Printer for SlithIr {
    fn argument(&self) -> &'static str { "slithir" }
    fn help(&self) -> &'static str { "Dump SlithIR for each function" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for (c, f) in slither.all_functions() {
            let ir = lift_function(f);
            writeln!(out, "{}.{}", c.name, f.signature())?;
            for (id, v) in &ir.variables {
                writeln!(out, "  var {:?} = {} : {:?}", id, v.name, v.typ)?;
            }
            for op in &ir.ops {
                writeln!(out, "  {:?}", op)?;
            }
            writeln!(out)?;
            let _ = c;
        }
        Ok(())
    }
}
