//! `--print data-dependency` — per-variable taint sources/sinks.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct DataDependency;
impl Printer for DataDependency {
    fn argument(&self) -> &'static str { "data-dependency" }
    fn help(&self) -> &'static str { "Per-variable taint sources/sinks" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for c in slither.all_contracts() {
            writeln!(out, "Contract {}", c.name)?;
            for f in &c.functions {
                for sv in &c.state_variables {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=\s*([^;]+);", regex::escape(&sv.name))).unwrap();
                    for cap in re.captures_iter(&f.body_excerpt) {
                        writeln!(out, "  {}.{} <- {}", c.name, sv.name, &cap[1])?;
                    }
                }
            }
            writeln!(out)?;
        }
        Ok(())
    }
}
