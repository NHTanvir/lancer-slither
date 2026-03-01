//! Per-IR analyses (data-dependency, taint, read/write sets, EVM mapping).

pub mod data_dependency;
pub mod evm_mapping;
pub mod read_var;
pub mod read_write;
pub mod taint;
pub mod write_var;
