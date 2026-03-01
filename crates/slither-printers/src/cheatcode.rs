//! `--print cheatcode` — list Foundry cheatcode usage.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct Cheatcode;
impl Printer for Cheatcode {
    fn argument(&self) -> &'static str { "cheatcode" }
    fn help(&self) -> &'static str { "Identify Foundry `vm.*` cheatcode usage" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        let re = regex::Regex::new(r"\bvm\.([A-Za-z_]\w*)\s*\(").expect("regex");
        let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for (c, f) in slither.all_functions() {
            for cap in re.captures_iter(&f.body_excerpt) {
                writeln!(out, "  {}.{} -> vm.{}", c.name, f.name, &cap[1])?;
                *counts.entry(cap[1].to_owned()).or_insert(0) += 1;
            }
        }
        if !counts.is_empty() {
            writeln!(out, "Totals:")?;
            for (k, v) in counts {
                writeln!(out, "  vm.{} x {}", k, v)?;
            }
        }
        Ok(())
    }
}
