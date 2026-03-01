//! `--print not-pausable` — state-modifying functions not gated by a pause modifier.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct NotPausable;
impl Printer for NotPausable {
    fn argument(&self) -> &'static str { "not-pausable" }
    fn help(&self) -> &'static str { "State-modifying functions not gated by a pause modifier" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for c in slither.all_contracts() {
            for f in &c.functions {
                let modifies_state = c.state_variables.iter().any(|sv| {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name))).unwrap();
                    re.is_match(&f.body_excerpt)
                });
                let pausable = f.modifiers.iter().any(|m| m.to_lowercase().contains("paus"));
                if modifies_state && !pausable {
                    writeln!(out, "  {}.{} mutates state without a pause guard.", c.name, f.name)?;
                }
            }
        }
        Ok(())
    }
}
