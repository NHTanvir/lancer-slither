//! Auto-detect framework when none was specified.

use std::path::Path;

use crate::{CompilationUnit, CompileOptions};

pub fn compile(target: &Path, opts: &CompileOptions) -> eyre::Result<CompilationUnit> {
    let fw = super::detect(target).unwrap_or(super::Framework::Solc);
    match fw {
        super::Framework::Foundry => super::foundry::compile(target, opts),
        super::Framework::Hardhat => super::hardhat::compile(target, opts),
        super::Framework::Truffle => super::truffle::compile(target, opts),
        super::Framework::Brownie => super::brownie::compile(target, opts),
        super::Framework::Dapp => super::dapp::compile(target, opts),
        super::Framework::Vyper => super::vyper::compile(target, opts),
        _ => super::solc_direct::compile(target, opts),
    }
}
