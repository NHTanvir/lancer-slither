//! `--print echidna` — emit Echidna config + property scaffold.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct Echidna;
impl Printer for Echidna {
    fn argument(&self) -> &'static str { "echidna" }
    fn help(&self) -> &'static str { "Echidna property scaffold + config" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        writeln!(out, "// echidna.yaml")?;
        writeln!(out, "testMode: assertion")?;
        writeln!(out, "testLimit: 50000")?;
        writeln!(out, "shrinkLimit: 5000")?;
        writeln!(out, "// scaffolding")?;
        for c in slither.all_contracts() {
            if c.functions.iter().any(|f| f.name == "transfer") {
                writeln!(out, "// === Echidna ERC-20 properties for {} ===", c.name)?;
                writeln!(out, "contract Echidna_{} is {} {{", c.name, c.name)?;
                writeln!(out, "    function echidna_balanceOf_self() public view returns (bool) {{")?;
                writeln!(out, "        return balanceOf(address(this)) >= 0;")?;
                writeln!(out, "    }}")?;
                writeln!(out, "    function echidna_totalSupply_nonneg() public view returns (bool) {{")?;
                writeln!(out, "        return totalSupply() >= 0;")?;
                writeln!(out, "    }}")?;
                writeln!(out, "}}")?;
            }
        }
        Ok(())
    }
}
