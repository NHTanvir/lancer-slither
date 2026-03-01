//! Solidity type system reified into Rust enums.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Default)]
pub enum DataLocation {
    Storage,
    Memory,
    Calldata,
    #[default]
    Unspecified,
}

/// Elementary types: `uint256`, `address`, `bool`, etc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ElementaryType {
    Address,
    AddressPayable,
    Bool,
    String,
    Bytes,
    BytesN(u8),
    Uint(u16),
    Int(u16),
    Fixed,
    Ufixed,
    Other(String),
}

/// User-defined types: contracts, structs, enums, user-defined value types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum UserDefinedType {
    Contract(String),
    Struct(String),
    Enum(String),
    UserDefinedValueType(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ArrayType {
    pub element: Box<Type>,
    pub length: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct MappingType {
    pub key: Box<Type>,
    pub value: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub returns: Vec<Type>,
    pub visibility: String,
    pub mutability: String,
}

/// The full Solidity type lattice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Type {
    Elementary(ElementaryType),
    UserDefined(UserDefinedType),
    Array(ArrayType),
    Mapping(MappingType),
    Function(FunctionType),
    Tuple(Vec<Type>),
    /// `Reference<T>` — qualifier-bound pointer.
    Reference(DataLocation, Box<Type>),
    /// Anything we haven't reified yet.
    Raw(String),
}

impl Type {
    pub fn parse_solc_type_string(s: &str) -> Self {
        let trimmed = s.trim();
        if let Some(rest) = trimmed.strip_prefix("uint") {
            if let Ok(n) = rest.parse::<u16>() {
                return Type::Elementary(ElementaryType::Uint(n));
            }
            if rest.is_empty() {
                return Type::Elementary(ElementaryType::Uint(256));
            }
        }
        if let Some(rest) = trimmed.strip_prefix("int") {
            if let Ok(n) = rest.parse::<u16>() {
                return Type::Elementary(ElementaryType::Int(n));
            }
            if rest.is_empty() {
                return Type::Elementary(ElementaryType::Int(256));
            }
        }
        if let Some(rest) = trimmed.strip_prefix("bytes") {
            if let Ok(n) = rest.parse::<u8>() {
                return Type::Elementary(ElementaryType::BytesN(n));
            }
            if rest.is_empty() {
                return Type::Elementary(ElementaryType::Bytes);
            }
        }
        match trimmed {
            "address" => Type::Elementary(ElementaryType::Address),
            "address payable" => Type::Elementary(ElementaryType::AddressPayable),
            "bool" => Type::Elementary(ElementaryType::Bool),
            "string" => Type::Elementary(ElementaryType::String),
            _ => Type::Raw(trimmed.to_owned()),
        }
    }
}
