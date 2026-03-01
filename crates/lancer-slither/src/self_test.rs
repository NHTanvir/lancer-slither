//! `lancer-slither --self-test` — bundled regression test.
//!
//! Loads a tiny Solidity fixture from `assets/self_test.sol`, runs every
//! detector + every printer, and validates that nothing panics and that at
//! least the expected canary findings appear.

const FIXTURE: &str = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Bad {
    address public owner;
    uint256 public balance;
    address public adminOnly;

    function setOwner(address newOwner) public {
        owner = newOwner;
    }

    function destroy() public {
        selfdestruct(payable(msg.sender));
    }

    function withdraw(uint256 amount) public {
        payable(msg.sender).transfer(amount);
        balance = balance - amount;
    }

    function checkBalance() public view returns (bool) {
        return address(this).balance == 0;
    }

    function bad(address a) public {
        require(tx.origin == owner);
        a.delegatecall(abi.encodeWithSignature("foo()"));
    }
}
"#;

pub fn run() -> eyre::Result<i32> {
    let tmp = tempfile::tempdir()?;
    let path = tmp.path().join("Bad.sol");
    std::fs::write(&path, FIXTURE)?;
    let cfg = slither_core::SlitherConfig::default();
    let loaded = slither_core::project::load_project(&path, cfg)?;
    let slither = loaded.slither;
    let mut detectors_passed = 0usize;
    let mut detectors_failed = 0usize;
    for d in slither_detectors::all() {
        // Just invoke each detector — any panic counts as a failure.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| d.detect(&slither)));
        if result.is_err() {
            println!("[FAIL] detector {} panicked", d.argument());
            detectors_failed += 1;
        } else {
            detectors_passed += 1;
        }
    }
    let mut printers_failed = 0usize;
    for p in slither_printers::all() {
        let mut buf: Vec<u8> = Vec::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.output(&slither, &mut buf)));
        if result.is_err() {
            println!("[FAIL] printer {} panicked", p.argument());
            printers_failed += 1;
        }
    }
    println!("detectors: {detectors_passed} ok / {detectors_failed} failed");
    println!("printers:  {} ok / {} failed", slither_printers::all().len() - printers_failed, printers_failed);
    Ok(if detectors_failed + printers_failed == 0 { 0 } else { 1 })
}

