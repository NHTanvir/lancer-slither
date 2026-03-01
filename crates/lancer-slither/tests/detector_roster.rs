//! Verify every detector in the upstream 0.11.5 roster is registered.
//! Names taken from `crytic/slither` README + release notes; cross-referenced
//! against `plan.md`.

const EXPECTED: &[&str] = &[
    // high
    "abiencoderv2-array", "arbitrary-send-erc20", "array-by-reference",
    "encode-packed-collision", "incorrect-shift", "multiple-constructors",
    "name-reused", "protected-vars", "public-mappings-nested",
    "rtlo", "shadowing-state", "suicidal", "uninitialized-state",
    "uninitialized-storage", "unprotected-upgrade",
    // medium
    "arbitrary-send-erc20-permit", "arbitrary-send-eth", "boolean-cst",
    "chronicle-unchecked-price", "constant-function-asm", "constant-function-state",
    "controlled-array-length", "controlled-delegatecall", "delegatecall-loop",
    "divide-before-multiply", "domain-separator-collision", "enum-conversion",
    "erc20-interface", "erc721-interface", "gelato-unprotected-randomness",
    "incorrect-equality", "incorrect-exp", "incorrect-return", "locked-ether",
    "mapping-deletion", "msg-value-loop", "out-of-order-retryable",
    "pyth-deprecated-functions", "pyth-unchecked-confidence", "pyth-unchecked-publishtime",
    "reentrancy-balance", "reentrancy-eth", "reentrancy-no-eth",
    "return-leave", "reused-constructor", "shadowing-abstract", "storage-array",
    "tautological-compare", "tautology", "tx-origin",
    "unchecked-lowlevel", "unchecked-send", "unchecked-transfer",
    "uninitialized-local", "unused-return", "weak-prng", "write-after-write",
    // low
    "calls-loop", "chainlink-feed-registry", "events-access", "events-maths",
    "incorrect-modifier", "incorrect-unary", "missing-zero-check", "optimism-deprecation",
    "reentrancy-benign", "reentrancy-events", "return-bomb",
    "shadowing-builtin", "shadowing-local", "timestamp",
    "uninitialized-fptr-cst", "variable-scope", "void-cst",
    // informational
    "assembly", "assert-state-change", "boolean-equal", "costly-loop",
    "cyclomatic-complexity", "dead-code", "deprecated-standards",
    "erc20-indexed", "function-init-state", "incorrect-using-for",
    "low-level-calls", "missing-inheritance", "naming-convention", "pragma",
    "redundant-statements", "reentrancy-unlimited-gas", "solc-version",
    "too-many-digits", "unimplemented-functions", "unindexed-event-address",
    "unused-state", "tod",
    // optimization
    "cache-array-length", "constable-states", "external-function",
    "immutable-states", "var-read-using-this",
];

#[test]
fn detector_roster_complete() {
    let registered: std::collections::BTreeSet<String> = slither_detectors::all()
        .iter()
        .map(|d| d.argument().to_owned())
        .collect();
    let mut missing = Vec::new();
    for d in EXPECTED {
        if !registered.contains(*d) {
            missing.push(*d);
        }
    }
    assert!(
        missing.is_empty(),
        "missing detectors vs upstream roster: {missing:?}\nregistered: {registered:?}"
    );
}

#[test]
fn detector_count_at_least_100() {
    let n = slither_detectors::all().len();
    assert!(n >= 100, "expected ≥100 detectors, got {n}");
}
