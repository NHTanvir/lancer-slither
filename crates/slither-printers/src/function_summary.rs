//! `--print function-summary` — per-function read/write sets, modifiers, calls.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct FunctionSummary;
impl Printer for FunctionSummary {
    fn argument(&self) -> &'static str { "function-summary" }
    fn help(&self) -> &'static str { "Per-function r/w sets, modifiers, calls" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for (c, f) in slither.all_functions() {
            writeln!(out, "{}.{}", c.name, f.signature())?;
            writeln!(out, "  visibility: {:?}", f.visibility)?;
            writeln!(out, "  mutability: {:?}", f.mutability)?;
            if !f.modifiers.is_empty() {
                writeln!(out, "  modifiers: {}", f.modifiers.join(", "))?;
            }
            let reads = collect_reads(&c.state_variables, &f.body_excerpt);
            let writes = collect_writes(&c.state_variables, &f.body_excerpt);
            writeln!(out, "  reads:  {}", reads.join(", "))?;
            writeln!(out, "  writes: {}", writes.join(", "))?;
            writeln!(out)?;
        }
        Ok(())
    }
}

fn collect_reads(state: &[slither_core::StateVariable], body: &str) -> Vec<String> {
    state.iter().filter_map(|sv| {
        let re = regex::Regex::new(&format!(r"\b{}\b", regex::escape(&sv.name))).unwrap();
        if re.is_match(body) { Some(sv.name.clone()) } else { None }
    }).collect()
}

fn collect_writes(state: &[slither_core::StateVariable], body: &str) -> Vec<String> {
    state.iter().filter_map(|sv| {
        let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name))).unwrap();
        if re.is_match(body) { Some(sv.name.clone()) } else { None }
    }).collect()
}
