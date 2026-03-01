const EXPECTED: &[&str] = &[
    "human-summary", "contract-summary", "function-summary", "vars-and-auth",
    "inheritance", "inheritance-graph", "call-graph", "cfg", "data-dependency",
    "loc", "modifiers", "require", "slithir", "slithir-ssa",
    "echidna", "evm", "martin", "halstead", "not-pausable",
    "entry-points", "cheatcode",
];

#[test]
fn printer_roster_complete() {
    let registered: std::collections::BTreeSet<String> = slither_printers::all()
        .iter()
        .map(|p| p.argument().to_owned())
        .collect();
    let mut missing = Vec::new();
    for d in EXPECTED {
        if !registered.contains(*d) {
            missing.push(*d);
        }
    }
    assert!(
        missing.is_empty(),
        "missing printers vs upstream roster: {missing:?}\nregistered: {registered:?}"
    );
}
