//! `--print martin` — Martin coupling metrics (afferent/efferent, abstractness, instability).

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct Martin;
impl Printer for Martin {
    fn argument(&self) -> &'static str { "martin" }
    fn help(&self) -> &'static str { "Martin coupling & abstractness metrics" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        let names: Vec<&str> = slither.all_contracts().map(|c| c.name.as_str()).collect();
        writeln!(out, "{:<24} {:>4} {:>4} {:>6} {:>6} {:>6}", "contract", "Ca", "Ce", "I", "A", "D")?;
        for c in slither.all_contracts() {
            let efferent = c.inheritance.len();
            let afferent = slither
                .all_contracts()
                .filter(|other| other.inheritance.iter().any(|n| n == &c.name))
                .count();
            let total_funcs = c.functions.len().max(1);
            let abstract_funcs = c.functions.iter().filter(|f| f.body_excerpt.trim().is_empty()).count();
            let abstractness = abstract_funcs as f64 / total_funcs as f64;
            let instability = if efferent + afferent == 0 { 0.0 } else { efferent as f64 / (efferent + afferent) as f64 };
            let distance = (abstractness + instability - 1.0).abs();
            writeln!(out, "{:<24} {:>4} {:>4} {:>6.2} {:>6.2} {:>6.2}", c.name, afferent, efferent, instability, abstractness, distance)?;
        }
        let _ = names;
        Ok(())
    }
}
