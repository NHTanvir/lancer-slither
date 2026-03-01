//! `--print loc` — lines of code per source unit, split by src/test/deps.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct Loc;
impl Printer for Loc {
    fn argument(&self) -> &'static str { "loc" }
    fn help(&self) -> &'static str { "LoC breakdown (src / test / deps)" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        let mut src = 0usize;
        let mut test = 0usize;
        let mut deps = 0usize;
        for su in &slither.source_units {
            let lines = su.content.lines().count();
            let path = su.relative_path.to_string_lossy().to_lowercase();
            if path.contains("/test") || path.contains("\\test") || path.starts_with("test") {
                test += lines;
            } else if slither.dependency_paths.iter().any(|d| su.path.starts_with(d))
                || path.contains("/lib/") || path.contains("\\lib\\") || path.contains("node_modules")
            {
                deps += lines;
            } else {
                src += lines;
            }
        }
        writeln!(out, "loc breakdown:")?;
        writeln!(out, "  src:  {}", src)?;
        writeln!(out, "  test: {}", test)?;
        writeln!(out, "  deps: {}", deps)?;
        Ok(())
    }
}
