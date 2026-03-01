# CLAUDE.md

Working notes for future Claude Code sessions on `lancer-slither`. Keep this
file authoritative — update it whenever architecture or conventions shift.

## What this repo is

A Rust reimplementation of `crytic/slither` (Trail of Bits' Solidity static
analyzer). Part of the Lancer ecosystem: GUI-by-default, CLI-on-demand, every
upstream flag / detector / printer / companion tool ported to a native binary.

Upstream parity target: **crytic/slither 0.11.5** (2026-01-16).

The full research dossier lives in `plan.md` (gitignored — local-only). Do
not push `plan.md` to GitHub.

## Workspace layout

```
.
├── Cargo.toml                       # workspace manifest
├── crates/
│   ├── lancer-slither/              # CLI binary + GUI + 12 companion bins
│   │   ├── src/
│   │   │   ├── main.rs              # entry: no args → GUI, else CLI
│   │   │   ├── lib.rs               # re-exports of modules below
│   │   │   ├── cli.rs               # full clap CLI struct + run()
│   │   │   ├── gui/                 # eframe GUI (three-pane layout)
│   │   │   ├── server.rs            # HTTP server (/v1/*, /mcp)
│   │   │   ├── mcp.rs               # MCP JSON-RPC façade
│   │   │   ├── ai_rank.rs           # all 7 AI augmentation entry points
│   │   │   ├── ide.rs               # --triage-mode interactive loop
│   │   │   ├── doctor.rs            # toolchain diagnostics
│   │   │   ├── self_test.rs         # bundled regression test
│   │   │   ├── erc.rs               # ERC conformance checker
│   │   │   ├── flatten.rs           # multi-file → one .sol
│   │   │   ├── mutate.rs            # operator-mutation testing
│   │   │   ├── prop.rs              # Echidna/Medusa/Halmos scaffold gen
│   │   │   ├── read_storage.rs      # live RPC storage reader
│   │   │   ├── upgradeability.rs    # storage-layout differ
│   │   │   ├── interface_gen.rs     # Solidity interface generator
│   │   │   └── bin/                 # 12 companion bin shims
│   │   └── tests/                   # parity tests
│   ├── slither-core/                # types, project model, output, suppression
│   ├── slither-ir/                  # SlithIR ops, types, SSA, CFG, callgraph
│   ├── slither-detectors/           # all 101 detectors, organized by tier
│   ├── slither-printers/            # all 21 printers
│   └── crytic-compile-rs/           # framework detect + solc + Etherscan
├── assets/self_test.sol             # fixture for --self-test
├── build.bat                        # Windows → build/lancer-slither.exe
├── build-windows.bat                # Windows packaged dist
├── build-linux.sh                   # Linux / macOS → build/lancer-slither
├── build-mac.sh                     # macOS .app bundle via cargo-bundle
├── README.md
├── CLAUDE.md                        # ← this file
└── plan.md                          # gitignored local blueprint
```

## Crate responsibilities

| Crate | What it owns |
|---|---|
| `slither-core` | `Finding`, `Impact`, `Confidence`, `SourceLocation`, `Slither` context, `Contract` / `Function` / `Variable` model, `SlitherConfig`, suppression parser, triage DB, output emitters (JSON / SARIF / Markdown via tera / text / zip / cross-check), AST loader (solc compact JSON → model + regex fallback). |
| `slither-ir` | SlithIR ops (Binary/Unary/Index/Member/Assignment/Push/Pop/Delete/Length/Balance/CodeSize/TypeConversion/Send/Transfer/Call/LowLevelCall/NewContract/NewArray/NewStructure/EventCall/Return/Phi/Condition/InitArray/Nop), type lattice, SSA scaffolding, CFG, callgraph with edge kinds, inheritance with C3 linearization, dominator tree, data-dependency fixed-point, taint propagation, source-map decoder. |
| `slither-detectors` | The `Detector` trait + `define_detectors!` macro + all 101 detectors split by tier (high.rs / medium.rs / low.rs / informational.rs / optimization.rs) + parallel runner (rayon). |
| `slither-printers` | The `Printer` trait + 21 printers (one file per printer). |
| `crytic-compile-rs` | Framework auto-detect (Foundry / Hardhat / Truffle / Brownie / Dapp / Vyper / standard-json / single-file / Etherscan), build-info ingest, solc binary discovery + auto-download from `binaries.soliditylang.org`, multi-chain Etherscan client. |
| `lancer-slither` | CLI binary, eframe GUI, 12 companion bins, HTTP server, MCP server, AI augmentation. |

## Conventions

* **Errors**: `eyre::Result<T>` throughout. `color-eyre` installed in `main`.
* **Logging**: `tracing` + `tracing-subscriber` with `RUST_LOG` env filter.
* **Parallelism**: `rayon` for the detector sweep. Detector trait objects are
  `Box<dyn Detector + Send + Sync>`.
* **Solidity types**: lifted from solc compact AST JSON in
  `slither-core/src/ast.rs`. When solc artifacts are missing, the same module
  has a regex-based `fallback_scrape` that produces a coarser model.
* **Source mapping**: `solc` emits `start:length:file_id` triplets;
  `SourceLocation::from_solc_triplet` parses them, derives line + column.
* **Suppression**: `// slither-disable-next-line <names>`,
  `// slither-disable-line <names>`, paired `start`/`end` blocks. Parsed once
  per file at `Slither` construction; consulted in `run_all`.

## Adding a new detector

1. Pick the tier file (`high.rs` / `medium.rs` / `low.rs` /
   `informational.rs` / `optimization.rs`).
2. Add a unit struct + `impl Detector` block with the canonical metadata
   (CLI argument, help one-liner, impact, confidence, wiki strings).
3. Add `out.push(Box::new(MyDetector))` (or `define_detectors!(out, [..])`)
   to that file's `register()`.
4. Append the CLI argument to
   `crates/lancer-slither/tests/detector_roster.rs` so the parity test
   keeps a record of intended coverage.
5. (Optional) Add a fixture line to `assets/self_test.sol` so `--self-test`
   exercises the new detector.

## Adding a new printer

1. New file under `crates/slither-printers/src/<name>.rs` with a unit
   struct + `impl Printer`.
2. Register in `crates/slither-printers/src/lib.rs` (`mod` line + entry in
   `all()`).
3. Append to `crates/lancer-slither/tests/printer_roster.rs`.

## CLI surface (every upstream flag)

`crates/lancer-slither/src/cli.rs` mirrors every flag in upstream
`slither --help` for 0.11.5. The CI test `tests/upstream_parity_flags.rs`
pins the expected long-name list — update both sides together when the
upstream flag set changes.

## GUI

eframe + egui 0.28. Three-pane layout (`gui/views/sidebar.rs`,
`findings_table.rs`, `detail.rs`) plus menu + status bar. The GUI thread
only renders; scans run on a `crossbeam_channel` worker thread.

A `Stop` command and `--serve` HTTP server are present. `muda` is wired so
the future Lancer-mothership tray integration can hook a native menu.

## HTTP / MCP server

`--serve 127.0.0.1:<port>` boots `tiny_http` and writes the port to
`%TEMP%/lancer-slither.port`. Endpoints:

* `GET  /v1/state`        — current jobs
* `POST /v1/scan`         — scan a target (body: `{"target":"path"}`)
* `GET  /v1/detectors`    — every detector + impact/confidence/help
* `GET  /v1/printers`     — every printer + help
* `POST /v1/import`       — accept payload from another Lancer tool
* `GET  /v1/export`       — current results as `json` / `csv` / `markdown` / `sarif`
* `POST /mcp`             — JSON-RPC (initialize / tools/list / tools/call / resources/list / resources/read)

## Build

* Windows: `build.bat` (drops every binary into `build\`) or
  `build-windows.bat` (packaged dist).
* Linux / macOS: `./build-linux.sh` (`--install` also drops a `.desktop`
  entry).
* macOS bundle: `./build-mac.sh --bundle` (needs `cargo install cargo-bundle`).

`Cargo.lock` should be committed once `cargo build` has been run (binary
crate convention).

## What "done" means here

* Every upstream-listed detector / printer / companion tool present.
* `lancer-slither --list-detectors` and `--list-printers` print the full roster.
* `lancer-slither --self-test` exits zero against `assets/self_test.sol`.
* `cargo test -p lancer-slither` passes the parity tests
  (`upstream_parity_flags`, `detector_roster`, `printer_roster`).
* No emoji in committed files. No `Co-Authored-By` trailers, no
  "Generated with Claude Code" footer in any commit.
* `plan.md` is **gitignored**. Never push it.

## Anti-patterns to avoid

* Don't store Solidity source content twice — `SourceUnit::content` is
  the canonical copy; detectors should read from there or from
  `Function::body_excerpt`.
* Don't introduce async on the GUI thread; long-running work goes through
  `crossbeam_channel` to the worker thread spawned in `gui::launch`.
* Don't tear out the regex `fallback_scrape` AST loader — it lets users
  run the tool with no solc installed for quick triage.
* Don't add new top-level files in `D:\`'s root; everything lives inside
  this repo. Per user rule, installs and projects must target `D:\`.

## Useful commands

```bash
cargo build --release              # all 13 binaries
cargo run -p lancer-slither -- .   # CLI on cwd
cargo test -p lancer-slither       # parity tests
cargo run -p lancer-slither        # GUI (no args)
cargo run -p lancer-slither -- --list-detectors
cargo run -p lancer-slither -- --self-test
cargo run -p lancer-slither -- --serve 127.0.0.1:7400
```
