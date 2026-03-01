//! Solidity domain model — `Contract`, `Function`, `Variable`, …
//!
//! These types are built from the solc AST by the `ast` module. Every detector
//! and printer reads from this model.

use std::collections::BTreeMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::source::SourceLocation;

/// Solidity version we discovered for a particular source unit.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SolidityVersion {
    /// The raw pragma string, e.g. `"^0.8.20"`.
    pub raw: String,
    /// The resolved `solc` binary version that was used.
    pub resolved: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    External,
    Internal,
    #[default]
    Private,
}

impl Visibility {
    pub fn from_solc(s: &str) -> Self {
        match s {
            "public" => Self::Public,
            "external" => Self::External,
            "internal" => Self::Internal,
            "private" => Self::Private,
            _ => Self::Private,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum StateMutability {
    Pure,
    View,
    #[default]
    Nonpayable,
    Payable,
}

impl StateMutability {
    pub fn from_solc(s: &str) -> Self {
        match s {
            "pure" => Self::Pure,
            "view" => Self::View,
            "payable" => Self::Payable,
            _ => Self::Nonpayable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FunctionKind {
    Function,
    Constructor,
    Fallback,
    Receive,
    FreeFunction,
    Modifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ContractKind {
    Contract,
    Interface,
    Library,
    AbstractContract,
}

impl ContractKind {
    pub fn from_solc(s: &str, is_abstract: bool) -> Self {
        match s {
            "interface" => Self::Interface,
            "library" => Self::Library,
            _ if is_abstract => Self::AbstractContract,
            _ => Self::Contract,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum DataLocation {
    Storage,
    Memory,
    Calldata,
    #[default]
    Unspecified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_str: String,
    pub data_location: DataLocation,
    pub source: SourceLocation,
    pub indexed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVariable {
    pub name: String,
    pub type_str: String,
    pub visibility: Visibility,
    pub is_constant: bool,
    pub is_immutable: bool,
    pub initial_value: Option<String>,
    pub source: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub type_str: String,
    pub source: SourceLocation,
    pub data_location: DataLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifier {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub source: SourceLocation,
    /// Raw body text — used by printers like `--print modifiers`.
    #[serde(default)]
    pub body_excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub kind: FunctionKind,
    pub visibility: Visibility,
    pub mutability: StateMutability,
    pub parameters: Vec<Parameter>,
    pub returns: Vec<Parameter>,
    pub modifiers: Vec<String>,
    pub source: SourceLocation,
    pub contract: String,
    /// Raw body text used by simple grep-style detectors.
    #[serde(default)]
    pub body_excerpt: String,
    /// solc AST node ID for callgraph cross-reference.
    #[serde(default)]
    pub ast_id: u64,
    /// Did the body contain any inline assembly block?
    #[serde(default)]
    pub has_assembly: bool,
    /// Function is virtual (overridable).
    #[serde(default)]
    pub is_virtual: bool,
    /// Function overrides another (uses `override`).
    #[serde(default)]
    pub overrides: Vec<String>,
}

impl Function {
    pub fn full_name(&self) -> String {
        format!("{}.{}", self.contract, self.name)
    }
    pub fn signature(&self) -> String {
        let params: Vec<String> = self.parameters.iter().map(|p| p.type_str.clone()).collect();
        format!("{}({})", self.name, params.join(","))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub name: String,
    pub kind: ContractKind,
    pub source: SourceLocation,
    pub inheritance: Vec<String>,
    pub linearized_base_contracts: Vec<u64>,
    pub state_variables: Vec<StateVariable>,
    pub functions: Vec<Function>,
    pub modifiers: Vec<Modifier>,
    /// AST node IDs of imported entities.
    #[serde(default)]
    pub imports: Vec<String>,
    /// solc AST node ID.
    #[serde(default)]
    pub ast_id: u64,
    /// Pragmas reachable in this file.
    #[serde(default)]
    pub pragma: Option<String>,
}

impl Contract {
    pub fn function_by_name(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }
}

/// A source unit (.sol file).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceUnit {
    pub path: std::path::PathBuf,
    pub relative_path: std::path::PathBuf,
    pub content: String,
    pub pragma: Option<String>,
    pub contracts: Vec<Contract>,
    /// Free / file-level functions (Solidity 0.7.1+).
    pub free_functions: Vec<Function>,
    /// AST root node ID.
    pub ast_id: u64,
    /// All `import` statements.
    pub imports: Vec<String>,
    /// solc compact AST as the raw JSON, kept for reanalysis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast_raw: Option<serde_json::Value>,
    /// Storage layout JSON (when available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_layout: Option<serde_json::Value>,
    /// solc source-map field (for evm printer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map: Option<String>,
}

/// Lookup index — contracts and functions globally addressable by name.
#[derive(Debug, Default, Clone)]
pub struct ContractIndex {
    pub by_name: IndexMap<String, Vec<u64>>,
    pub by_id: BTreeMap<u64, (usize, usize)>, // (source_unit_idx, contract_idx)
}
