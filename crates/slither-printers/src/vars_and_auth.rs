//! `--print vars-and-auth` — variables written + the access checks gating each write.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct VarsAndAuth;
impl Printer for VarsAndAuth {
    fn argument(&self) -> &'static str { "vars-and-auth" }
    fn help(&self) -> &'static str { "Variables written + auth checks gating each" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for c in slither.all_contracts() {
            writeln!(out, "Contract {}", c.name)?;
            for f in &c.functions {
                for sv in &c.state_variables {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name))).unwrap();
                    if re.is_match(&f.body_excerpt) {
                        let auth: Vec<&String> = f.modifiers.iter().filter(|m| m.contains("only") || m.contains("Owner") || m.contains("Auth")).collect();
                        writeln!(out, "  {} written by {} (auth: {:?})", sv.name, f.signature(), auth)?;
                    }
                }
            }
            writeln!(out)?;
        }
        Ok(())
    }
}
