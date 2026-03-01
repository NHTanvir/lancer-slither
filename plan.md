# RESEARCH DOSSIER: Slither

> Part of the Lancer ecosystem. A Rust clone of crytic/slither — Solidity static analyzer. Build with Claude Code.
> Upstream: github.com/crytic/slither (Python, AGPL-3.0, ~0.10.x). Industry-standard Solidity analyzer used by Trail of Bits since 2018. 100+ detectors, 15+ printers, SlithIR intermediate representation, full crytic-compile multi-framework support.

## Why both Slither and Aderyn

We already have `D:/19/aderyn/` (Cyfrin's Rust analyzer with ~90 detectors). Slither is **different** and complementary:
- Industry-standard since 2018 (Trail of Bits, AGPL-3.0) — DeFi audit firms expect Slither output specifically.
- 100+ named detectors, many with no Aderyn equivalent (reentrancy variants, dataflow-driven taints, dependency analyses).
- **SlithIR** — Slither's own three-address intermediate representation distinct from Aderyn's direct AST walk. SlithIR makes reference tracking, taint propagation, and SSA analyses tractable.
- **Printer system**: `slither --print human-summary | call-graph | inheritance | data-dependency | function-summary | vars-and-auth | echidna | evm | cfg | slithir | slithir-ssa` — Aderyn has none of these. Printers are the unsung backbone of audit-firm workflows.
- `slither-mutate`, `slither-prop`, `slither-check-upgradeability`, `slither-read-storage` ecosystem.
- **crytic-compile** dependency: handles Foundry / Hardhat / Truffle / Brownie / Vyper / Dapp / Etherscan address / standard-json / single-file input — Aderyn only does Foundry/Hardhat.
- Keeping Slither in the Lancer box means we can produce Slither-format reports without leaving the suite, plus run a **differential pass** showing which findings only Slither catches vs only Aderyn catches.

## Feature Parity Mandate

**`lancer-slither` MUST ship with 100% feature parity to the upstream `slither` tool — every flag, every subcommand, every output format, every probe / detector / module / payload / mode — plus a native Rust GUI on top.** This is not a "subset clone" — this is a full reimplementation in Rust with a GUI wrapper.

### Parity contract

- **Every CLI flag** documented in upstream `slither --help` (and the long-form man page / docs) must work in `lancer-slither --cli`.
- **Every output format** the upstream tool emits (text, JSON, XML, CSV, SARIF, custom formats) must round-trip through `lancer-slither` byte-equivalent where the format allows.
- **Every probe / detector / module / category / payload class / built-in script** must be ported. None silently dropped.
- **Every supported target type / input format / protocol / file format / language** must work.
- **Every configuration option** (config files, env vars, registry / settings entries) must be honoured.
- **Companion binaries / sub-tools** in the upstream suite (if any) must each get their own binary in this workspace.

### GUI on top

Every feature reachable via CLI is also reachable from the GUI (see `## GUI (Lancer-standard pattern)` below). The GUI is not a subset — it is an alternate frontend over the same `core` library that the CLI uses. The `core` crate is GUI-agnostic.

### Verification

- The `## Granular feature catalog` section below enumerates everything that must be supported.
- Reference: `tests/upstream_parity/` will pin upstream flag combinations + canned outputs as parity tests. CI fails if any flag is unsupported or output diverges.
- A `lancer-slither --self-test` flag runs the full parity matrix against a bundled reference target and prints pass/fail per upstream feature group.

### What is allowed to diverge

- **Performance** — Rust-native, async-first; we are allowed to be faster.
- **Memory footprint** — we are allowed to be smaller.
- **GUI** — entirely new (no upstream GUI to copy except where the upstream ships its own GUI like Zenmap / Wireshark UI / Ghidra GUI / jadx-gui — in which case our GUI must cover every panel the upstream GUI has, plus the Lancer-standard layout from `## GUI` below).
- **AI augmentation** — net-new (see `## AI augmentation ideas`).

### What is NOT allowed to diverge

- Any flag silently dropped, renamed, or behavior-changed compared to upstream.
- Any output format / record schema diverging in field names, types, or semantics.
- Any built-in script / detector / module not ported (mark explicitly with `[N/A — see note]` if genuinely impossible in Rust, e.g. NSE Lua scripts requiring features not in `mlua`).

## What it actually does

Slither (Trail of Bits, AGPL-3.0) takes a Solidity codebase (or Vyper, or an Etherscan address) and:
1. Compiles via `crytic-compile` (auto-detect framework, call right `solc`, normalize artifacts).
2. Parses the solc AST and **lifts to SlithIR** — a high-level three-address IR with explicit value types, storage/memory/calldata reference annotations, side-effect tracking, and an SSA form (`slithir-ssa`).
3. Constructs control-flow graphs per function, call graph across contracts, inheritance DAG, and data-dependency graph.
4. Runs 100+ detectors over these graphs, classifying findings by **impact** (High/Medium/Low/Informational/Optimization) and **confidence** (High/Medium/Low).
5. Optionally runs printers (visualizations, summaries, callgraphs, EVM mapping).
6. Emits JSON / SARIF / Markdown / human-readable text / triage-mode interactive output.
7. Plugs into companion tools: `slither-mutate` (mutation testing), `slither-prop` (Echidna/Medusa property generation), `slither-check-upgradeability` (proxy storage layout diffing), `slither-read-storage` (live storage reader).

## Internal architecture

**Target workspace** (`Cargo.toml`):
```
[workspace]
members = [
  "lancer-slither",
  "slither-core",
  "slither-ir",
  "slither-detectors",
  "slither-printers",
  "crytic-compile-rs",
]
```

- **`lancer-slither`** — CLI binary (clap 4.x). Same flags as upstream `slither` for drop-in compat. Parses args, picks detector/printer/triage mode, calls `slither-core`.
- **`slither-core`** — Orchestration: project discovery, solc invocation, IR construction driver, detector/printer dispatch, output emission, config merging from `slither.config.json`.
- **`slither-ir`** — SlithIR translation from Solidity AST. Three-address-code style with explicit `READ`/`WRITE`/`CALL` nodes, type lattice, reference tracking, SSA pass, data-dependency analysis.
- **`slither-detectors`** — 100+ detectors organized by impact tier.
- **`slither-printers`** — Visualizations and summaries.
- **`crytic-compile-rs`** — Project format detector and solc driver, normalizes output to a `CompilationUnit` struct.

**Key modules in `slither-core/src/`:**

- `project/`
  - `discover.rs` — Walk a directory, detect framework (`foundry.toml`, `hardhat.config.{js,ts}`, `truffle-config.js`, `brownie-config.yaml`, `dapp.toml`, `vapor.toml`), dispatch to `crytic-compile-rs`.
  - `etherscan.rs` — Fetch verified contracts by address from Etherscan / Blockscout / Routescan (so users can `lancer-slither 0xabc...`).
  - `solc_select.rs` — `which solc` + `semver`-based selection of correct solc version per source pragma; auto-download via solc-select-compatible mirror if missing.
- `slithir/`
  - `convert.rs` — AST → SlithIR conversion. Hardest part: model reference types (`storage` / `memory` / `calldata` references) explicitly; expand modifiers inline; resolve inherited function calls; lower expressions to 3-address.
  - `operations/` — IR operation enums: `Binary`, `Unary`, `Index`, `Member`, `Assignment`, `Push`, `Pop`, `Call` (HighLevel/Internal/Library/Solidity/External), `Send`, `Transfer`, `LowLevelCall`, `Delete`, `EventCall`, `NewContract`, `NewArray`, `NewStructure`, `TypeConversion`, `Return`, `Phi` (SSA), `Condition`, `InitArray`, `Length`, `Balance`, `CodeSize`.
  - `types.rs` — Solidity type system reified into Rust enums: `ElementaryType`, `UserDefinedType` (Contract/Struct/Enum/UserDefinedValueType), `ArrayType`, `MappingType`, `FunctionType`, `Reference<T>` (storage/memory/calldata).
  - `analyses/`
    - `data_dependency.rs` — Slither's flagship analysis: for each variable, who can taint it? Used by `arbitrary-send-eth`, `controlled-delegatecall`, `tx-origin`, `unprotected-upgrade`.
    - `evm_mapping.rs` — Map IR ops back to EVM bytecode offsets (for the `evm` printer and `slither-read-storage`).
    - `taint.rs` — Forward/backward taint propagation over the SSA form.
    - `read_var.rs`, `write_var.rs` — Per-IR-node read/write sets, used by reentrancy detectors.
  - `ssa.rs` — Phi insertion + variable renaming.
- `graph/`
  - `cfg.rs` — Per-function control-flow graph (petgraph `DiGraph`).
  - `callgraph.rs` — Cross-contract call graph (HighLevel/Internal/Library edges, separate edge type for low-level).
  - `inheritance.rs` — Contract inheritance DAG with C3 linearization for method resolution.
  - `dominator.rs` — Dominator tree used for SSA and reachability.
- `output/`
  - `json.rs`, `sarif.rs` (via `serde_sarif`), `markdown.rs` (via `tera` templates), `text.rs`, `zip.rs` — emitters.
  - `cross_check.rs` — Compare two runs (regression CI).
  - `triage.rs` — Interactive mode: each finding presented for review; auto-emits `slither-triage.json` of FP marks.

**How detectors plug in:**
1. Each detector is a Rust struct implementing the trait:
```rust
pub trait Detector: Send + Sync + 'static {
    fn argument(&self) -> &'static str;           // CLI name e.g. "reentrancy-eth"
    fn help(&self) -> &'static str;               // One-line help
    fn impact(&self) -> Impact;                   // High|Medium|Low|Informational|Optimization
    fn confidence(&self) -> Confidence;           // High|Medium|Low
    fn wiki_url(&self) -> &'static str;
    fn wiki_title(&self) -> &'static str;
    fn wiki_description(&self) -> &'static str;
    fn wiki_exploit_scenario(&self) -> &'static str;
    fn wiki_recommendation(&self) -> &'static str;
    fn detect(&mut self, slither: &Slither) -> Vec<Finding>;
}
```
2. `define_detectors!` macro lists every struct → registry + CLI mapping.
3. Detectors run in parallel via `rayon` over an immutable `Slither` context.

**How printers plug in:**
```rust
pub trait Printer: Send + Sync + 'static {
    fn argument(&self) -> &'static str;
    fn help(&self) -> &'static str;
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> Result<()>;
}
```

## Granular feature catalog

**CLI surface** (`lancer-slither [target] [flags]`):

*Target forms:*
- Path (`./`, `contracts/`, `src/MyContract.sol`).
- Single Solidity file (`Token.sol`).
- Etherscan address (`0xabc...` — pulls verified source).
- Standard JSON input (`--standard-json contract.json`).
- crytic-compile archive (`--compile-force-framework foundry`).

*Detector control:*
- `--detect <list>` — run only these detectors (comma-separated).
- `--exclude <list>` — exclude these detectors.
- `--include-detectors <list>` — force-include detectors even if excluded by tier/group rules (added 0.10.3).
- `--exclude-dependencies` — skip imported library contracts.
- `--exclude-optimization`, `--exclude-informational`, `--exclude-low`, `--exclude-medium`, `--exclude-high`.
- `--show-ignored-findings` — include findings suppressed by NatSpec.
- `--filter-paths <regex>` — exclude matching source paths from analysis.
- `--include-paths <regex>` — only matching paths (added 0.10.1).

*Printer control:*
- `--print <printer>` — run printer instead of detectors.
- `--list-detectors`, `--list-printers`.

*Output formats:*
- `--json <out.json>` — Slither-native JSON.
- `--sarif <out.sarif>` — for GitHub code-scanning.
- `--markdown <out.md>` — for human consumption.
- `--markdown-root <url>` — hyperlink prefix for code references.
- `--checklist` — markdown checklist format (popular among auditors).
- `--zip <out.zip>` — bundled outputs.
- `--no-fail-pedantic`, `--fail-pedantic`, `--fail-low`, `--fail-medium`, `--fail-high` — CI exit codes.

*crytic-compile pass-through:*
- `--compile-force-framework {foundry|hardhat|truffle|brownie|dapp|vyper|standard|solc}`.
- `--compile-libraries <name:addr,...>`.
- `--compile-remove-metadata`.
- `--compile-custom-build <command>`.
- `--solc <path>`, `--solc-args`, `--solc-disable-warnings`, `--solc-working-dir`, `--solc-remaps`.
- `--solc-solcs-select <0.8.20>` — pin specific solc.
- `--solc-solcs-bin <path>` — use prebuilt solc binary.
- `--ignore-compile`, `--skip-clean` — reuse last build.

*Etherscan mode:*
- `--etherscan-apikey <KEY>`.
- `--etherscan-export-directory <dir>` — cache verified source.
- `--etherscan-only-source-code`, `--etherscan-only-bytecode`.
- `--etherscan-export-dir-format {standard|crytic_compile}`.

*Triage:*
- `--triage-mode` — interactive walk through findings, mark FP, output `slither-triage.json`.
- `--triage-database <file>` — preload prior triage decisions.
- `--change-line-prefix` — for re-running after refactor.

*Config:*
- `--config-file slither.config.json` — load defaults.

**Config file (`slither.config.json`):**
```json
{
  "detectors_to_run": "all",
  "detectors_to_exclude": "naming-convention,solc-version",
  "exclude_informational": true,
  "exclude_optimization": false,
  "exclude_low": false,
  "exclude_dependencies": true,
  "filter_paths": "node_modules|test/mocks",
  "fail_pedantic": false,
  "solc_disable_warnings": true,
  "json": "slither-report.json",
  "sarif": "slither-report.sarif"
}
```

**Project input formats handled by `crytic-compile-rs`:**
- **Foundry** — read `foundry.toml`, run `forge build --build-info`, ingest `out/build-info/*.json`.
- **Hardhat** — read `hardhat.config.{js,ts,cjs}`, run `npx hardhat compile`, ingest `artifacts/build-info/*.json`.
- **Truffle** — `truffle-config.js`, `truffle compile`, `build/contracts/*.json`.
- **Brownie** — `brownie-config.yaml`, `brownie compile`, `build/contracts/*.json`.
- **Dapp** — `dapp.toml`, `dapp build`.
- **Vyper** — `.vy` files compiled via `vyper`, AST normalized to Slither's IR.
- **Etherscan** — by address, multi-chain (`--etherscan-network mainnet|polygon|arbitrum|...`); deduplicates if multi-file verified.
- **Standard JSON** — solc Standard JSON Input format.
- **Single-file** — `.sol` invoked with current PATH solc.
- **embark, waffle, buidler** (legacy auto-detect; pass-through to solc directly).

**High-impact detectors (full list):**
- `abiencoderv2-array` — known pre-0.5.10 ABIEncoderV2 array bug.
- `arbitrary-send-erc20` — `IERC20.transferFrom(from=user_input, ...)`.
- `array-by-reference` — passing storage array to fn taking memory reorders writes.
- `encode-packed-collision` — `abi.encodePacked` collisions with dynamic types of different bit widths.
- `incorrect-shift` — `2 << x` where shift count and value swapped.
- `multiple-constructors` — pre-0.4.22 dual constructor naming pattern (legacy `function ContractName()` + `constructor`).
- `name-reused` — two contracts share a name in the workspace.
- `protected-vars` — vulnerable-pattern detector for protected variables.
- `public-mappings-nested` — nested mapping public getter blocking ABI generation.
- `rtlo` — right-to-left override unicode.
- `shadowing-state` — child contract state var shadows parent's.
- `suicidal` — `selfdestruct` callable by anyone.
- `uninitialized-state` — state variable used before initialization.
- `uninitialized-storage` — uninitialized local storage pointer (aliases slot 0 — classic critical bug).
- `unprotected-upgrade` — upgrade function with no access control.

**(Detectors below moved to medium-impact tier in upstream 0.11.x but listed for legacy reference):**
- `reentrancy-eth` — reentrancy that can drain ETH (state write after external call with ETH).
- `reentrancy-no-eth` — reentrancy without ETH transfer (still dangerous).
- `reentrancy-unlimited-gas` — reentrancy via `addr.call.value(x)("")` (full gas).
- `reentrancy-events` — events emitted after external call (event misordering).
- `reentrancy-benign` — state write after call that returns from same contract (low impact).
- `arbitrary-send-eth` — ETH sent to arbitrary user-controlled address.
- `arbitrary-send-erc20-permit` — same via `permit` cheat.
- `controlled-array-length` — `arr.length = x` where x is user-controlled (deprecated but still in legacy code).
- `controlled-delegatecall` — `delegatecall(target=user_input)`.
- `delegatecall-loop` — delegatecall inside loop (cumulative state corruption).
- `incorrect-equality` — strict equality on `address(this).balance` etc.
- `tautology` — always-true/false condition.
- `boolean-equal` — `x == true`.
- `divide-before-multiply` — `(a / b) * c` precision loss.
- `weak-prng` — `block.timestamp` / `blockhash` as randomness.
- `unchecked-transfer` — ignored `transfer` return value.
- `unchecked-send` — ignored `send` return value.
- `unchecked-lowlevel` — ignored low-level call return value.
- `mapping-deletion` — `delete` on mapping inside struct (no-op).
- `incorrect-modifier` — modifier missing `_` placeholder.
- `void-cst` — empty constructor that should call inherited.
- `enum-conversion` — type conversion to enum out of range.
- `incorrect-exp` — `2 ** x` where wrong base.
- `msg-value-loop` — `msg.value` inside a loop (consumed multiple times).
- `storage-array` — signed integer storage arrays (pre-0.5 ABIv2 bug).

**Medium-impact detectors:**
- `arbitrary-send-erc20-permit` — `permit` cheat enabling arbitrary `transferFrom`.
- `arbitrary-send-eth` — ETH sent to arbitrary user-controlled address.
- `boolean-cst` — boolean constant in non-trivial expression.
- `chronicle-unchecked-price` — Chronicle oracle return value not validated (added 0.11.0).
- `constant-function-asm` — function declared `view`/`pure` containing assembly that may not be.
- `constant-function-state` — function declared `view`/`pure` modifying state.
- `controlled-array-length` — `arr.length = x` (pre-0.6 length assignment).
- `controlled-delegatecall` — `delegatecall(target=user_input)`.
- `delegatecall-loop` — delegatecall inside loop.
- `divide-before-multiply` — `(a / b) * c` precision loss.
- `domain-separator-collision` — EIP-712 domain separator vs fn selector collision.
- `enum-conversion` — type conversion to enum out of range.
- `erc20-interface` — non-spec ERC-20 signatures (renamed from `incorrect-erc20-interface`).
- `erc721-interface` — non-spec ERC-721 signatures (renamed from `incorrect-erc721-interface`).
- `gelato-unprotected-randomness` — Gelato VRF callback not properly gated (added 0.11.0).
- `incorrect-equality` — strict equality on `address(this).balance`.
- `incorrect-exp` — `**` precedence issue.
- `incorrect-return` — `return` used in assembly block when `leave` was intended.
- `locked-ether` — contract receives ETH but has no withdrawal path.
- `mapping-deletion` — `delete` on mapping inside struct.
- `msg-value-loop` — `msg.value` inside a loop.
- `out-of-order-retryable` — Arbitrum retryable ticket ordering (added 0.10.1).
- `pyth-deprecated-functions` — deprecated Pyth oracle functions (added 0.11.0).
- `pyth-unchecked-confidence` — Pyth price `conf` field unchecked (added 0.11.0).
- `pyth-unchecked-publishtime` — Pyth `publishTime` staleness unchecked (added 0.11.0).
- `reentrancy-balance` — reentrancy via balance check (added 0.11.5).
- `reentrancy-eth` — classic reentrancy draining ETH.
- `reentrancy-no-eth` — reentrancy without ETH transfer.
- `return-leave` — using `return` in inline assembly where `leave` is intended.
- `reused-constructor` — pre-0.5 constructor naming confusion.
- `shadowing-abstract` — abstract contract state shadowed by child.
- `storage-array` — signed integer storage arrays (pre-0.5 ABIv2 bug).
- `tautological-compare` — comparison of variable to itself.
- `tautology` — always-true/false condition.
- `tx-origin` — `tx.origin` used in auth.
- `unchecked-lowlevel` — ignored low-level call return.
- `unchecked-send` — ignored `send` return value.
- `unchecked-transfer` — ignored `transfer` return value.
- `uninitialized-local` — local variable used before initialization.
- `unused-return` — function return value never read.
- `weak-prng` — `block.timestamp` / `blockhash` as randomness.
- `write-after-write` — same var assigned twice without read between.

**Low-impact detectors:**
- `calls-loop` — external call inside loop (DOS risk).
- `chainlink-feed-registry` — Chainlink Feed Registry usage on non-mainnet chains (added 0.11.0).
- `events-access` — access-control state change without event emission.
- `events-maths` — math-relevant state change without event.
- `incorrect-modifier` — modifier missing `_` placeholder.
- `incorrect-unary` — `=+` instead of `+=`.
- `missing-zero-check` — setter without `!= address(0)` check.
- `optimism-deprecation` — usage of deprecated Optimism predeploys (added 0.11.0).
- `reentrancy-benign` — state write after call returning from same contract.
- `reentrancy-events` — events emitted after external call.
- `return-bomb` — caller may revert via huge returndata from callee.
- `shadowing-builtin` — identifier shadows a built-in symbol.
- `shadowing-local` — local variable shadows outer state.
- `timestamp` — `block.timestamp` used as condition.
- `uninitialized-fptr-cst` — uninitialized function pointer call in constructor (Solidity bug pre-0.5.8).
- `variable-scope` — local variable declaration shadowed by outer scope.
- `void-cst` — empty constructor that should call inherited.

**Informational detectors:**
- `assembly` — inline assembly present (heads-up for auditor).
- `assert-state-change` — `assert()` with side effects.
- `boolean-equal` — `x == true`.
- `costly-loop` — SSTORE inside loop.
- `cyclomatic-complexity` — high McCabe complexity.
- `dead-code` — unreachable code.
- `deprecated-standards` — deprecated keywords (`sha3`, `throw`, `suicide`).
- `erc20-indexed` — ERC-20 event fields missing `indexed`.
- `function-init-state` — state var initialized via fn call at declaration.
- `incorrect-using-for` — `using X for Y` mismatch (added 0.10.x).
- `low-level-calls` — `addr.call()` present.
- `missing-inheritance` — function looks like it should inherit an interface.
- `naming-convention` — mixedCase / UPPER_CASE / leading underscore rules.
- `pragma` — multiple `pragma solidity` versions across files.
- `redundant-statements` — dead statements.
- `reentrancy-unlimited-gas` — reentrancy via full-gas low-level call.
- `solc-version` — fixed/floating pragma audit.
- `too-many-digits` — `100000000000000000` should be `1e18`.
- `unimplemented-functions` — abstract function not implemented.
- `unindexed-event-address` — event address parameters not indexed (added 0.11.4).
- `unused-state` — unused state variable.
- `tod` — transaction order dependence heuristic.

**Optimization detectors:**
- `cache-array-length` — cache `.length` outside loops (added 0.10.x).
- `constable-states` — could be `constant`.
- `external-function` — public function could be external.
- `immutable-states` — could be `immutable`.
- `var-read-using-this` — reading own state via `this.var` (gas waste).
- (Historical: `similar-names` removed in 0.10.3 for performance; `unused-import` disabled in 0.10.4 for performance.)

**Printer inventory** (`--print <name>`):

*Quick review printers:*
- `human-summary` — terse per-contract overview (state vars, functions, modifiers, inheritance, complexity).
- `inheritance` — DOT graph of inheritance hierarchy.
- `inheritance-graph` — alias, includes function-level edges.
- `contract-summary` — terse per-contract printout.
- `loc` — lines-of-code breakdown (src vs test vs deps).
- `entry-points` — state-changing entry point functions + the variables they write (added 0.11.0).

*In-depth printers:*
- `call-graph` — full call graph as DOT.
- `cfg` — per-function CFG as DOT.
- `data-dependency` — per-variable taint sources/sinks.
- `function-summary` — per-function read/write sets, modifiers, calls.
- `vars-and-auth` — variables written and the access-control checks gating each write (auditor favorite).
- `not-pausable` — list state-modifying functions not gated by a pause modifier.
- `modifiers` — list modifiers and their gating effect.
- `require` — list every require/assert.
- `cheatcode` — identify Foundry `vm.*` cheatcode usage (added 0.11.0).

*Format/export printers:*
- `echidna` — emit Echidna config + property scaffolding.
- `evm` — map each Solidity statement to EVM bytecode PC range.
- `slithir` — dump SlithIR for each function.
- `slithir-ssa` — dump SSA form of SlithIR.

*Metrics printers:*
- `martin` — Martin metrics (afferent/efferent coupling, abstractness, instability).
- `halstead` — Halstead complexity metrics.

**Companion tools (in upstream Slither repo):**
- `slither-mutate` — mutation testing: rewrite operators (`+`→`-`, `<`→`<=`, etc.), re-run tests, report surviving mutants. Revamped in 0.10.2 with Foundry project support.
- `slither-prop` — generate Echidna/Medusa property scaffolds (ERC-20 conformance, ERC-721 conformance, ERC-4626 vault invariants).
- `slither-check-upgradeability` — compare storage layouts of V1 vs V2 implementation contracts for proxy safety. 0.10.4 added multi-reinitializer detection.
- `slither-read-storage` — given a deployed contract address and slot path, dereference storage at a block height. Cross-32-byte boundary slot calc fixed in 0.11.1.
- `slither-find-paths` — show paths from function A to function B in the call graph.
- `slither-format` — auto-apply fixes for some detectors (rare).
- `slither-flat` — flatten multi-file projects into one .sol. Unicode-safe since 0.11.1.
- `slither-check-erc` — verify ERC conformance for a contract (note: upstream binary name is `slither-check-erc`, not `slither-erc`).
- `slither-interface` — generate the Solidity `interface` declaration for a target contract (added with 0.11.x; useful for cross-project integration).
- `slither-doctor` — diagnose installation / compiler / framework detection issues (upstream troubleshooting helper).
- `slither-documentation` — generate NatSpec documentation skeletons for unsdocumented public/external functions.

## Engineering hard parts

**SlithIR three-address conversion.** Solidity expressions can be deeply nested with side effects (`a[b()] = c() + d.e`). SlithIR lowers each to a sequence of primitive ops with explicit temporaries. The hard parts:
- **Reference types.** Solidity has `storage`, `memory`, `calldata` reference qualifiers. A `storage` reference points to a slot; a `memory` reference points to allocated memory. SlithIR encodes these as `Reference<Storage>` / `Reference<Memory>` / `Reference<Calldata>` variants. Misclassifying reference vs value type breaks data-dependency.
- **Tuple expressions.** `(a, b) = (foo(), bar())` lowers to tuple temporaries with order-of-evaluation rules.
- **Try/catch.** Solidity try/catch lowers to a sub-CFG with separate success/failure blocks.
- **Function-type values.** `function() external` is a first-class value; SlithIR represents it as `FunctionType` with target/selector fields.

**Storage / memory / calldata reference tracking.** A storage pointer aliased through a local variable must be tracked: `Foo storage f = arr[i]; f.x = 5;` writes `arr[i].x`. SlithIR's `Index` and `Member` ops carry the reference qualifier; analyses follow it.

**Data-dependency analysis.** Slither's flagship. For every variable, build the set of variables it transitively reads from, scoped by function and across calls. Used by:
- `arbitrary-send-eth` — does the recipient depend on a function parameter (no access gate)?
- `controlled-delegatecall` — does the delegatecall target depend on a function parameter?
- `tx-origin` — does an auth check depend on `tx.origin`?
- `unprotected-upgrade` — does the upgrade function's caller check depend on `msg.sender`?

The hard part is inter-procedural propagation: when function A passes `x` to function B, B's parameters become tainted by A's `x`. SlithIR resolves this via SSA + an iterative fixed-point pass.

**Modifier expansion.** Modifiers are inlined at each callsite to make per-function analysis sound. The `_` placeholder marks where the wrapped fn body goes; modifiers can have multiple `_`s (e.g., before/after hooks) which means the body executes multiple times. SlithIR expansion produces a flattened CFG.

**Inherited contract method resolution.** Solidity inheritance uses C3 linearization. When `child.foo()` is called, the actual function executed depends on the linearized inheritance order. SlithIR resolves each call to a concrete function in the inheritance graph; `super.foo()` resolves to the next-in-line.

**crytic-compile project discovery.** Each framework has its quirks:
- Foundry's `out/build-info/*.json` contains both the AST and the bytecode but is keyed by content hash, not contract name.
- Hardhat sometimes ships an older artifact format; need to detect and adapt.
- Truffle's compile is slow; we re-invoke only on cache miss.
- Brownie wraps Vyper differently than Solidity.
- Etherscan returns either a single source file or a JSON object containing all files; format detection is heuristic.

**solc version pinning.** Source `pragma solidity ^0.8.20` matched against available solc binaries via `semver`. If missing, download from `binaries.soliditylang.org` (sha256-verified). Per-file pragma drives per-file solc choice; mixed-pragma projects compile per-file and merge.

**ABI vs source mapping.** The `evm` printer maps source lines to EVM PCs using solc's source map field (`s:l:f:j;...`). The compressed source-map format requires a stateful decoder. Used for `slither-read-storage` and for IR-to-bytecode debugging.

**Suppression annotations.** NatSpec `// slither-disable-next-line reentrancy-eth` and `// slither-disable-start ... slither-disable-end` blocks. Annotations are parsed alongside the source and consulted post-detection.

**Performance.** 100+ detectors × large workspaces × multiple analyses (CFG, callgraph, dataflow, SSA) — we parallelize at the detector level via `rayon`, share the immutable `Slither` context, and cache per-function analyses.

## Rust library mapping

| Upstream piece | Rust crate to use | Notes |
|---|---|---|
| Solidity AST parsing | `solidity-ast` (from `solidity-ast-rs`, reused from `lancer-aderyn`) | Same AST node types as solc JSON |
| Alternate parser | `solar-parse` + `solar-sema` (Foundry's parser) | Fallback if `solidity-ast` lacks coverage |
| Vyper parsing | `vyper-parser` binding via FFI, or shell-out to `vyper` | Vyper input minority case |
| Graph algorithms | `petgraph` 0.6 | CFG, callgraph, inheritance, dominator |
| SSA construction | hand-rolled on top of petgraph dominator | Cytron-Ferrante algorithm |
| Templating | `tera` 1.x | Markdown report templating |
| SARIF emit | `serde_sarif` 0.4 | GitHub code-scanning compatible |
| JSON | `serde_json` 1.0 | Slither-native JSON |
| CLI | `clap` 4.4 | Mirror Slither's flag names |
| Parallelism | `rayon` 1.11 | Per-detector parallel scan |
| Etherscan HTTP | `octocrab`-style reqwest + `reqwest` 0.12 | Multi-chain support |
| solc binary detection | `which` 6.x | Find `solc` on PATH |
| Version matching | `semver` 1.x | Pragma → solc selection |
| HTTP downloads | `reqwest` 0.12 + `sha2` | Auto-download solc from binaries.soliditylang.org |
| TOML config | `toml` 0.8 | crytic-compile config |
| Process spawn | `duct` or `std::process::Command` | Invoke `solc`, `forge`, `npx hardhat` |
| Errors | `eyre` 0.6 | Match Aderyn |
| Logging | `tracing` 0.1 + `tracing-subscriber` | Structured logs |
| Async runtime | `tokio` 1.40 | Etherscan + LSP |
| Regex | `regex` 1.11 | `--filter-paths`, NatSpec suppression |
| ZIP output | `zip` 2.x | `--zip` bundled output |

(Where `lancer-aderyn` and `lancer-slither` both need `solidity-ast`, we depend on the same crate version to avoid duplicate parsing.)

## Real-world audit workflows

- **Full audit of a Foundry project.** `lancer-slither . --json out.json --markdown out.md --filter-paths "test/|node_modules|lib/forge-std"`. Tool runs crytic-compile-rs (detects Foundry), invokes `forge build --build-info`, lifts all source units to SlithIR, runs all detectors. Auditor opens `out.md`, ranks findings by `Impact * Confidence`.
- **Differential audit (Slither + Aderyn merged).** Lancer GUI runs both in parallel; merges findings by `(file, line, detector-class)`; surfaces overlap and uniques. Auditor sees which tool found what, builds confidence in coverage.
- **Etherscan address quick-look.** `lancer-slither 0x6B175474E89094C44Da98b954EedeAC495271d0F --etherscan-apikey $KEY --print human-summary`. Pulls DAI's verified source, builds IR, dumps a one-page summary — useful before signing a transaction.
- **Upgradeability check (slither-check-upgradeability).** `lancer-slither-check-upgradeability --new ./V2.sol --old ./V1.sol`. Compares storage layouts slot-by-slot; flags incompatible layout changes that would brick a proxy. Reads solc's storageLayout artifact, walks slot/offset trees.
- **Property generation for Echidna/Medusa (slither-prop).** `lancer-slither --print echidna` on an ERC-20 contract → emits an `EchidnaERC20.sol` test harness with conformance properties (`transfer must reduce sender by amount`, `transferFrom must check allowance`, etc.). Output is a starting point for property-based fuzzing.
- **Mutation testing (slither-mutate).** Walk the AST, swap operators (`+`→`-`, `<`→`<=`, `==`→`!=`), re-run forge tests, report mutants that survived (i.e., were not killed by tests) — proxy for test-suite quality.
- **Storage reader.** `lancer-slither-read-storage 0xabc... --variable balanceOf --key 0xdef...` reads live storage from RPC. Uses the storage layout artifact + keccak slot derivation.
- **CI gate.** Pre-commit or GitHub Action: `lancer-slither . --fail-high --sarif slither.sarif`; SARIF auto-uploaded to GitHub Security tab.
- **Triage workflow.** First pass: `lancer-slither . --triage-mode` — each finding presented one at a time; auditor marks `false-positive` / `won't-fix` / `valid`. Marks saved to `slither-triage.json`. Subsequent re-runs honor prior decisions.

## 2024–2026 state

- **0.10.0 (early 2024):** Major Solidity 0.8.19+ support, `user-defined-value-types` plumbed through SlithIR, new `transient` storage location (post-EIP-1153), detector improvements for re-entrancy under transient.
- **0.10.1–0.10.3 (mid 2024):** crytic-compile bumped for Foundry's `build-info` v1 format; Hardhat 2.22 compat; new detectors for `incorrect-using-for` and `cache-array-length`; fixes for SlithIR with custom errors (`error MyError(uint x);`).
- **0.10.4 (late 2024):** EIP-7702 (set-code transactions) awareness; better support for inline-Yul blocks; `martin` and `halstead` printers re-enabled.
- **0.10.5–0.10.6 (Q1 2025):** **MCP server** support added so Claude/Cursor can call individual detectors as MCP tools (catching up with Aderyn); new detectors for `abi-encode-packed-collision` variants, `incorrect-exp`, `domain-separator-collision`; improved SARIF schema; experimental Vyper 0.4 support.
- **0.11.x (mid 2025):** Detector for `transient`-storage reentrancy variants; printer `data-dependency` performance pass; multi-chain Etherscan support (Polygon, Arbitrum, Base, Optimism); experimental `--ai-rank` flag piping findings to an LLM for severity ranking.
- **2026 dev (latest):** Continued MCP server hardening; SlithIR SSA refactor; detector for ERC-4626 vault edge cases (`erc4626-no-zero-deposit`); ERC-7683 cross-chain intent detectors; integration hooks for Halmos (export `check_*` scaffolds), Medusa (export property templates), Echidna (existing).
- **Trail of Bits ecosystem.** Slither remains the canonical SAST in `building-secure-contracts` repo; `solc-select` and `crytic-compile` are sibling projects maintained by ToB and used everywhere.

### Confirmed upstream releases (verified 2026-05-30)

- **0.10.1 (2024-02-29):** `out-of-order-retryable` detector (Arbitrum retryable tickets); `--include-paths` flag; Solidity 0.8.24 support; top-level events; assembly storage tracking; assembly shift detection.
- **0.10.2 (2024-04-08):** `unused-import` detector (later disabled in 0.10.4 for perf); `slither-mutate` revamped with Foundry support; import resolution + alias improvements; reference/declaration API for LSP; accurate implicit return modeling in IR.
- **0.10.3 (2024-06-05):** `--include-detectors` flag (override exclusions); `similar-names` detector REMOVED for performance; FP reductions across detectors.
- **0.10.4 (2024-08-23):** web3.py compat fixes; `solc-version` FP reductions; `arbitrary-send-eth` refined to skip immutable state; `unused-import` disabled; `slither-check-upgradeability` adds multi-reinitializer detection.
- **0.11.0 (2025-02-03):** 7 new detectors — `pyth-deprecated-functions`, `pyth-unchecked-confidence`, `pyth-unchecked-publishtime`, `chronicle-unchecked-price`, `gelato-unprotected-randomness`, `chainlink-feed-registry`, `optimism-deprecation`. 2 new printers — `entry-points`, `cheatcode`. Custom error support in `require`; transient storage support; breaking API changes to Contract variables/calls handling.
- **0.11.1 (2025-04-15):** Unicode in mutator/flattener; call-stack info added to `calls-loop`/`costly-loop`/`delegatecall-loop`/`msg-value-loop`; storage pointer analysis enhancements; fixed slot calc across 32-byte boundaries; `slither-read-storage` struct-only fixes; YUL identifier parsing fix.
- **0.11.2 (2025-04-16):** Storage pointer analysis bug fixes affecting written-variable tracking.
- **0.11.3 (2025-04-18):** Dependency updates for `web3` and `eth-*` packages.
- **0.11.4 (2026-01-15):** `unindexed-event-address` detector; CLZ EVM opcode support; custom Solidity storage layout support; dev environment migrated to `uv` + `ruff`; try/catch parsing fixes; Echidna printer crash fixes; fixed-size array signature fixes.
- **0.11.5 (2026-01-16):** `reentrancy-balance` detector; minimum Python bumped to 3.10; codebase type annotations modernized; enhanced source mapping for `unindexed-event-address`.

### Note 2026-05-30 (deep-research update)

Verified detector / printer / companion-tool roster against upstream `crytic/slither` README and releases pages (latest tag: **0.11.5, 2026-01-16**). Plan revised to add: high-impact `encode-packed-collision`, `multiple-constructors`, `uninitialized-state`, `uninitialized-storage`; medium-impact `chronicle-unchecked-price`, `constant-function-asm`, `constant-function-state`, `gelato-unprotected-randomness`, `incorrect-return`, `locked-ether`, `out-of-order-retryable`, `pyth-deprecated-functions`, `pyth-unchecked-confidence`, `pyth-unchecked-publishtime`, `reentrancy-balance` (0.11.5), `return-leave`, `shadowing-abstract`, `tautological-compare`, `uninitialized-local`; low-impact `chainlink-feed-registry`, `optimism-deprecation`, `return-bomb`, `shadowing-builtin`, `shadowing-local`, `uninitialized-fptr-cst`, `variable-scope`; informational `incorrect-using-for`, `unindexed-event-address`. New printers `entry-points` and `cheatcode` (0.11.0) added. Companion tools list now includes `slither-interface`, `slither-doctor`, `slither-documentation`, and renames `slither-erc` → `slither-check-erc` to match upstream binary name. CLI `--include-detectors` (0.10.3) and `--include-paths` (0.10.1) flags annotated with version-of-introduction. Detector tier reorganization noted: 0.11.x upstream moves several reentrancy + arbitrary-send detectors out of high-impact down to medium-impact; plan retains both lists with cross-reference.

## AI augmentation ideas

- **Rank findings by AI.** Each finding (with surrounding code context, taint trace, and detector confidence) goes to Claude; LLM returns a 0-100 priority score and a 1-sentence justification. Replaces / augments the static Impact×Confidence matrix.
- **Generate human-readable explanations.** Slither's wiki descriptions are good but generic. LLM rewrites per-finding with the actual variable/function names from the codebase (e.g., "In `Vault.withdraw`, the external call to `IERC20(token).transfer(msg.sender, amount)` precedes the state write `balances[msg.sender] -= amount`...").
- **Suggest fixes.** Given a finding + AST snippet, LLM proposes a patch as a unified diff. For mechanical issues (`>` to `>=`, add `require(addr != address(0))`, swap state-write before call), often correct out of the box.
- **Classify false positives.** A persistent FP-classifier model trained on triaged findings. New findings get a "likely-FP" score from the LLM; threshold-suppress.
- **Cross-detector clustering.** A 60-finding report often has 5-10 root causes. LLM clusters by semantic similarity (e.g., "all 14 of these are the same missing-event pattern across setters"), collapsing the report.
- **SlithIR → natural language.** A "what does this function do?" printer that lowers SlithIR to a natural-language summary, useful for diffing PRs.
- **Custom detector scaffolding.** "Write a detector that flags any function calling `_burn` while `paused == true`." LLM emits a Rust detector struct against `slither-core` APIs. Compile-and-load via dylib for the run.
- **Auto-triage of historical reports.** Feed a year of past Slither runs + the audits' actual published findings; LLM learns which detector outputs the team historically promoted vs ignored, ranks future runs accordingly.

## Implementation phases (14-week clone)

SlithIR is genuinely hard — allow extra time vs Aderyn's 10-week plan.

**Week 1 — Workspace bootstrap & AST.** Cargo workspace (6 crates as above). Pull `solidity-ast` crate. Round-trip OpenZeppelin's ERC20.sol AST through our types. Smoke test: `lancer-slither --print human-summary OpenZeppelinERC20.sol` outputs a minimal summary.

**Week 2 — crytic-compile-rs: Foundry & Hardhat.** Project discovery for the two most-used frameworks. Spawn `forge build --build-info` / `npx hardhat compile`, ingest the build-info JSON, normalize to a `CompilationUnit { source_units, asts, storage_layouts, abis, source_maps }` struct.

**Week 3 — crytic-compile-rs: Truffle, Brownie, Vyper, Etherscan, standard-json.** Round out remaining input formats. Multi-chain Etherscan with API key. Verified source dedup logic for multi-file responses.

**Week 4 — SlithIR base.** Translate AST expressions to three-address IR; implement `Binary`, `Unary`, `Index`, `Member`, `Assignment`, `Call` (HighLevel/Internal/Library/Solidity), `Return`, `Condition`. Test against a handful of OpenZeppelin functions.

**Week 5 — SlithIR references.** Reference qualifier tracking (`storage`/`memory`/`calldata`). `Push`, `Pop`, `Delete`, `Length`, `Balance`, `CodeSize` ops. Tuple expressions. Try/catch sub-CFG.

**Week 6 — SlithIR SSA + dataflow.** Phi insertion (Cytron-Ferrante), variable renaming. `data_dependency.rs`: per-variable taint sources. Inter-procedural propagation via fixed-point.

**Week 7 — CFG + callgraph + inheritance graph.** Per-function CFG (petgraph). Cross-contract callgraph with edge types (HighLevel/Internal/Library/External). Inheritance DAG with C3 linearization for method resolution. Modifier expansion at callsites.

**Week 8 — Detector trait + first 15 high-impact detectors.** Implement `Detector` trait + `define_detectors!` macro. Build `reentrancy-eth`, `reentrancy-no-eth`, `reentrancy-unlimited-gas`, `suicidal`, `arbitrary-send-eth`, `arbitrary-send-erc20`, `controlled-delegatecall`, `delegatecall-loop`, `incorrect-shift`, `weak-prng`, `unchecked-transfer`, `unchecked-send`, `unchecked-lowlevel`, `unprotected-upgrade`, `mapping-deletion`. Regression test against SmartBugs Curated.

**Week 9 — Remaining high-impact + 20 medium-impact detectors.** Cover the rest of high tier; build `tx-origin`, `events-access`, `events-maths`, `missing-zero-check`, `calls-loop`, `assembly`, `assert-state-change`, `deprecated-standards`, `unused-return`, `costly-loop`, `pragma`, `function-init-state`, `domain-separator-collision`, `erc20-indexed`, `incorrect-erc20-interface`, `incorrect-erc721-interface`, `boolean-cst`, `boolean-equal`, `incorrect-unary`, `enum-conversion`.

**Week 10 — Low + Informational + Optimization detectors.** Cover remaining ~50 detectors (mechanical: code-quality checks, naming conventions, solc-version, constable-states, immutable-states, external-function, similar-names, dead-code, redundant-statements, var-read-using-this, unused-state, too-many-digits, cyclomatic-complexity, etc.).

**Week 11 — Printers.** `human-summary`, `contract-summary`, `function-summary`, `vars-and-auth`, `inheritance`, `call-graph`, `cfg`, `data-dependency`, `loc`, `modifiers`, `require`, `slithir`, `slithir-ssa`. DOT output for graph printers.

**Week 12 — Echidna/EVM/Martin/Halstead printers + output formats.** `echidna` (property scaffolds), `evm` (PC mapping via source maps), `martin` (coupling metrics), `halstead` (complexity metrics). SARIF emitter via `serde_sarif`. Markdown via `tera`. JSON via `serde_json`. ZIP bundler.

**Week 13 — Triage mode + suppression + companion tools.** Interactive `--triage-mode`; `// slither-disable-next-line` and `slither-disable-start/end` parsing; `lancer-slither-check-upgradeability` (storage-layout differ); `lancer-slither-read-storage` (live RPC reader); `lancer-slither-flat` (flattener); `lancer-slither-mutate` (operator mutation).

**Week 14 — AI augmentation + MCP server + polish.** Wire `--ai-rank` (LLM-scored severity). MCP server module exposing each detector as a tool and each printer as a resource. Performance pass with `rayon` parallelization. Benchmark against upstream Slither on a 100k-line corpus; aim for 5-10x faster wall-clock. Drop-in compat smoke test: run Lancer Slither's JSON output through Slither's own SARIF converter to verify schema match.

---

## GUI (Lancer-standard pattern)

`lancer-slither` ships as a **native GUI application** by default. The CLI surface listed in `## Granular feature catalog` is still available behind a `--cli` flag (or via the `lancer-slither-cli` companion binary), but the **plain `lancer-slither` invocation opens a native desktop window**, matching every other tool in the Lancer ecosystem. No terminal-only mode.

### Framework

Pure-Rust GUI stack — no JavaScript, no Node, single static binary:

| Crate | Role |
|-------|------|
| `eframe` | Cross-platform window + event loop wrapper around `egui` |
| `egui` | Immediate-mode UI widgets (panels, tables, forms, menus, drag-and-drop) |
| `egui_extras` | Sortable tables, image loaders, syntax highlighting, date pickers |
| `egui_plot` | Charts, graphs, timelines (histograms, activity meters, progress) |
| `rfd` | Native file/folder pickers (open file, save dialog) |
| `directories` | Per-OS config dir (`~/.config/lancer-slither/`, `%APPDATA%\lancer-slither\`, etc.) |
| `image` | Screenshot / preview rendering inside egui |
| `arboard` | Clipboard read/write for "copy result" actions |
| `muda` | Native menu bar (Linux / macOS / Windows) |
| `egui-notify` | Toast notifications |

Optional dark/light theme persisted to `lancer-slither-config.json` via `serde_json`.

### Window layout

Three-pane layout matching the Lancer mothership GUI:

```
┌─────────────────────────────────────────────────────────────────────┐
│  File   Tools   Settings   Help                              [≡]    │
├──────────┬────────────────────────────────────────┬─────────────────┤
│          │                                        │                 │
│  Inputs  │           Main view                    │  Detail pane    │
│  / jobs  │  (table of results / source tree /     │  (full record / │
│  / scope │   findings list / packet list / etc.)  │   bytes / src / │
│          │                                        │   remediation)  │
│          │                                        │                 │
├──────────┴────────────────────────────────────────┴─────────────────┤
│  Status: idle           Items: 0          [Open]  [Run]  [Export]   │
└─────────────────────────────────────────────────────────────────────┘
```

- **Left sidebar (200-300px)** — inputs / queued jobs / scope / target list. Drag-and-drop accepted.
- **Center pane (flexible)** — primary result view appropriate to the tool (table for findings, tree for hierarchical data, code editor for source).
- **Right pane (300-400px, collapsible)** — full detail of the currently-selected item.
- **Top menu bar** — File (Open / Save / Export / Recent / Quit), Tools (per-tool actions), Settings (theme, fonts, integrations), Help (About / Docs).
- **Bottom status bar** — current activity, item counts, primary action buttons.

### Cross-tool integration with Lancer + the other 29 clones

Every Lancer-clone GUI exposes a small local HTTP server on a port auto-allocated at launch (`127.0.0.1:<port>`, written to `%TEMP%/lancer-slither.port`):

- The main Lancer GUI discovers running tools by scanning `%TEMP%/lancer-*.port` and lists them in its sidebar.
- Right-clicking any item in any tool can "Send to <other Lancer tool>" — the receiving tool fetches the payload via its own HTTP endpoint.
- A future "Lancer Hub" tray shell can launch / focus any of the 30 GUIs from one menu.
- HTTP endpoints exposed:
  - `GET  /v1/state` — current jobs / progress
  - `POST /v1/import` — accept a target / file / record from another Lancer tool
  - `GET  /v1/export?format=json|csv|sarif|markdown` — pull the current result set in a portable format

### Async work + responsiveness

- Long-running scans run on a Tokio task pool; the GUI thread only renders.
- Progress propagates via `crossbeam::channel` from worker → UI.
- A `Stop` button cancels in-flight work via a `CancellationToken`.
- All file I/O / network I/O / subprocess work goes through async channels — the egui thread never blocks.

### Build output

The existing `build.bat` / `build-windows.bat` / `build-linux.sh` already produce a single binary. With `eframe` linked, the same binary is the GUI app.

- Windows: `build\lancer-slither.exe` (double-click opens the GUI window).
- Linux: `build/lancer-slither` (also installable as a `.desktop` entry via `build-linux.sh --install`).
- macOS: `build/lancer-slither` (bundled into `lancer-slither.app` via `cargo-bundle` in the `build-mac.sh` companion).

For CI / headless / scripting, pass `--cli` to fall through to the CLI surface from `## Granular feature catalog`.

## Build Setup (Lancer-standard pattern)

This clone follows the same build pattern as the main Lancer project — single `.exe` output via `build.bat` on Windows, `.so/elf` via `build-linux.sh` on Linux.

### Project layout

```
lancer-slither/
├── Cargo.toml                  # workspace manifest
├── Cargo.lock                  # committed (binary, not library)
├── crates/
│   ├── lancer-slither/         # CLI binary
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── slither-core/           # orchestration, project discovery, dispatch
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── slither-ir/             # SlithIR conversion, SSA, dataflow
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── slither-detectors/      # 100+ detectors
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── slither-printers/       # 15+ printers
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── crytic-compile-rs/      # multi-framework project loader
│       ├── Cargo.toml
│       └── src/lib.rs
├── build.bat                   # Windows build → build/lancer-slither.exe
├── build-windows.bat           # Windows packaged dist → dist/lancer-slither-windows-x86_64/
├── build-linux.sh              # Linux build → build/lancer-slither
├── .gitignore                  # plan.md (local-only) + target/ + build/ + dist/
├── README.md                   # public-facing (pushed to GitHub)
└── plan.md                     # this file (gitignored — local blueprint, never pushed)
```

### Build commands

**Windows (double-click `build.bat`):**
```
build.bat
```
Produces `build\lancer-slither.exe`. Pauses at end so the window stays open.

**Windows (packaged dist):**
```
build-windows.bat
```
Produces `dist\lancer-slither-windows-x86_64\bin\lancer-slither.exe` plus LICENSE/README — ready to zip.

**Linux / macOS:**
```bash
./build-linux.sh
```
Produces `build/lancer-slither`.

### Requirements
- Rust toolchain (https://rustup.rs)
- Windows: Visual C++ Build Tools 2019/2022 (needed by some crates)
- `solc` discoverable on PATH (or auto-downloaded by `crytic-compile-rs` from binaries.soliditylang.org)
- Optional: `forge`, `npx hardhat`, `truffle`, `brownie`, `vyper` if scanning matching project types

### CI

The build pattern is intentionally identical across Lancer and all 19 clones, so a single Jenkins/GitHub Actions matrix can build everything by calling `build-windows.bat` or `build-linux.sh` per repo.
