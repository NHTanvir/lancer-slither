//! `--print evm` — map source statements to EVM PC ranges using the
//! deployed-bytecode source map (when available).

use std::io::Write;

use slither_core::Slither;
use slither_ir::analyses::evm_mapping::decode;

use crate::Printer;

pub struct Evm;
impl Printer for Evm {
    fn argument(&self) -> &'static str { "evm" }
    fn help(&self) -> &'static str { "Map source statements to EVM PC ranges" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for su in slither.user_source_units() {
            if let Some(map) = &su.source_map {
                let entries = decode(map);
                writeln!(out, "{}: {} PC entries", su.relative_path.display(), entries.len())?;
                for (pc, e) in entries.iter().enumerate().take(50) {
                    writeln!(out, "  pc {:5}  src offset {:>6} length {:>4} file {} jump {}",
                        pc, e.start, e.length, e.file_id, e.jump)?;
                }
            } else {
                writeln!(out, "{}: (no source map available — solc artifact missing)", su.relative_path.display())?;
            }
        }
        Ok(())
    }
}
