//! `lancer-slither` library — shared between the CLI binary, the GUI binary,
//! and every companion binary in `src/bin/`.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::module_name_repetitions)]

pub mod ai_rank;
pub mod cli;
pub mod doctor;
pub mod erc;
pub mod flatten;
pub mod gui;
pub mod ide;
pub mod interface_gen;
pub mod mcp;
pub mod mutate;
pub mod prop;
pub mod read_storage;
pub mod self_test;
pub mod server;
pub mod upgradeability;

pub use cli::{Cli, ExitMode};
