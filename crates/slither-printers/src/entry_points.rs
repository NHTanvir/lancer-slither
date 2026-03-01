//! `--print entry-points` — state-changing entry points + their writes.

use std::io::Write;

use slither_core::{Slither, Visibility};

use crate::Printer;

pub struct EntryPoints;
impl Printer for EntryPoints {
    fn argument(&self) -> &'static str { "entry-points" }
    fn help(&self) -> &'static str { "Public state-modifying entry points + their writes" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for c in slither.all_contracts() {
            for f in &c.functions {
                if !matches!(f.visibility, Visibility::Public | Visibility::External) {
                    continue;
                }
                let writes: Vec<&str> = c.state_variables.iter().filter_map(|sv| {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name))).unwrap();
                    if re.is_match(&f.body_excerpt) { Some(sv.name.as_str()) } else { None }
                }).collect();
                if !writes.is_empty() {
                    writeln!(out, "  {}.{} writes: {}", c.name, f.signature(), writes.join(", "))?;
                }
            }
        }
        Ok(())
    }
}
