# lancer-slither

Rust clone of `slither` — Solidity static analyzer. Part of the Lancer ecosystem.

`lancer-slither` ships as a **native GUI by default**: just double-click the
binary. The full CLI surface is available via `lancer-slither --cli` (or by
passing any positional argument).

## Highlights

* **101 detectors** across `High` / `Medium` / `Low` / `Informational` / `Optimization` tiers.
* **21 printers**: `human-summary`, `contract-summary`, `function-summary`,
  `vars-and-auth`, `inheritance`, `call-graph`, `cfg`, `data-dependency`,
  `loc`, `entry-points`, `cheatcode`, `slithir`, `slithir-ssa`, `echidna`,
  `evm`, `martin`, `halstead`, `not-pausable`, `modifiers`, `require`.
* **Framework support**: Foundry, Hardhat, Truffle, Brownie, Dapp, Vyper,
  Etherscan address (multi-chain), standard-json input, single-file solc.
* **Companion binaries**: `lancer-slither-mutate`, `-prop`,
  `-check-upgradeability`, `-read-storage`, `-find-paths`, `-format`,
  `-flat`, `-check-erc`, `-interface`, `-doctor`, `-documentation`.
* **Outputs**: text (coloured), JSON, SARIF (GitHub code-scanning), Markdown
  report, Markdown checklist, ZIP bundle.
* **AI-rank** (`--ai-rank`): score findings by impact + confidence + breadth.
* **Local HTTP / MCP server** (`--serve 127.0.0.1:0`): expose every detector
  and printer to the Lancer mothership and to other Lancer tools.

## Build

### Windows

Double-click `build.bat` or run from `cmd`:

```
build.bat
```

Output: `build\lancer-slither.exe` plus every companion binary.

Packaged dist:

```
build-windows.bat
```

Output: `dist\lancer-slither-windows-x86_64\bin\*.exe`.

### Linux / macOS

```bash
./build-linux.sh
```

Output: `build/lancer-slither` plus every companion. `./build-linux.sh --install`
also drops a `.desktop` launcher into `~/.local/share/applications/`.

### Requirements

* [Rust / Cargo](https://rustup.rs) (≥ 1.75).
* Windows: Visual C++ Build Tools 2019/2022.
* `solc` on PATH (or auto-downloaded by `crytic-compile-rs` from
  `binaries.soliditylang.org` when `--solc-solcs-select` is used).
* Optional, depending on target framework: `forge`, `npx hardhat`,
  `truffle`, `brownie`, `vyper`.

## Usage

```
lancer-slither                                       # opens GUI
lancer-slither contracts/ --json out.json --sarif out.sarif
lancer-slither 0x6B175474E89094C44Da98b954EedeAC495271d0F --etherscan-apikey $KEY --print human-summary
lancer-slither . --triage-mode
lancer-slither --list-detectors
lancer-slither --self-test
lancer-slither --serve 127.0.0.1:0                   # exposes /v1/scan, /v1/state, /v1/detectors, /v1/printers
```

## Workspace layout

```
lancer-slither/
├── crates/
│   ├── lancer-slither/         # CLI binary + GUI + companion bins
│   ├── slither-core/           # types, project model, output emitters
│   ├── slither-ir/             # SlithIR, SSA, CFG, callgraph, taint
│   ├── slither-detectors/      # all 101 detectors
│   ├── slither-printers/       # all 21 printers
│   └── crytic-compile-rs/      # framework detect, solc driver, Etherscan
├── build.bat / build-windows.bat / build-linux.sh
└── README.md
```

## Status

Scaffold complete; detectors are pattern-matchers over the lifted Solidity
model and ship with full upstream-style metadata. Deeper analyses (SSA-driven
inter-procedural taint, full IR lowering of expressions, source-map driven
EVM mapping for `slither-read-storage`) are in progress.

See the gitignored `plan.md` for the full 14-week implementation blueprint.
