//! `--print require` — list every require/assert in the project.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct Require;
impl Printer for Require {
    fn argument(&self) -> &'static str { "require" }
    fn help(&self) -> &'static str { "List every require/assert" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        let re = regex::Regex::new(r"\b(?:require|assert)\s*\(([^;]+)\)").expect("regex");
        for (c, f) in slither.all_functions() {
            for cap in re.captures_iter(&f.body_excerpt) {
                writeln!(out, "{}.{}: {}", c.name, f.name, &cap[1])?;
            }
        }
        Ok(())
    }
}
