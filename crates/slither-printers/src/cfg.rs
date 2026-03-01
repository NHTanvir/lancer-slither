//! `--print cfg` — DOT control-flow graph per function.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct Cfg;
impl Printer for Cfg {
    fn argument(&self) -> &'static str { "cfg" }
    fn help(&self) -> &'static str { "Per-function CFG as DOT" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for (c, f) in slither.all_functions() {
            writeln!(out, "digraph \"{}_{}\" {{", c.name, f.name)?;
            writeln!(out, "  label = \"{}.{}\";", c.name, f.name)?;
            let mut blocks: Vec<(usize, String)> = Vec::new();
            for (i, line) in f.body_excerpt.lines().enumerate() {
                let l = line.trim();
                if l.is_empty() { continue; }
                blocks.push((i, l.to_owned()));
            }
            for (i, b) in &blocks {
                let safe = b.replace('"', "\\\"").chars().take(80).collect::<String>();
                writeln!(out, "  n{} [label=\"{}\"];", i, safe)?;
            }
            for w in blocks.windows(2) {
                writeln!(out, "  n{} -> n{};", w[0].0, w[1].0)?;
            }
            writeln!(out, "}}")?;
        }
        Ok(())
    }
}
