//! SlithIR operations.
//!
//! Inspired by upstream `slithir/operations/*.py`. Each `Op` is a single
//! three-address instruction.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::types::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VariableId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub id: VariableId,
    pub name: String,
    pub typ: Type,
    /// `true` if this variable is a contract state variable (not a local).
    pub is_state: bool,
    pub is_parameter: bool,
    pub is_return: bool,
    pub is_constant: bool,
    pub is_temporary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
    LogicalAnd,
    LogicalOr,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    PreInc,
    PostInc,
    PreDec,
    PostDec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallKind {
    /// HighLevel call: `contract.fn()` resolved to a real Function.
    HighLevel { contract: String, function: String },
    /// `super.fn()` resolved.
    Super { contract: String, function: String },
    /// Internal call within same contract / inherited.
    Internal { function: String },
    /// Library call: `using L for T`.
    Library { library: String, function: String },
    /// Builtin Solidity call (`keccak256`, `sha3`, `require`, …).
    Solidity { name: String },
    /// External call: `addr.call{value}(data)` etc.
    LowLevel { method: String },
    /// Untyped low-level external call via interface.
    External { function: String },
    /// `new Contract(...)` / `new Type[n]`.
    NewContract { name: String },
    NewArray { element: String, length: Option<u64> },
    NewStructure { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSite {
    pub kind: CallKind,
    pub arguments: Vec<VariableId>,
    pub returns: Vec<VariableId>,
    /// Source mapping inside the original `.sol`.
    pub source_offset: usize,
    pub source_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateWrite {
    pub variable: VariableId,
    pub source_offset: usize,
    pub source_length: usize,
    /// `true` if the write occurs strictly *after* an external call in the
    /// same block. Used by the reentrancy detectors.
    pub after_external_call: bool,
}

/// One IR operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op {
    Binary {
        dst: VariableId,
        op: BinaryOp,
        lhs: VariableId,
        rhs: VariableId,
    },
    Unary {
        dst: VariableId,
        op: UnaryOp,
        operand: VariableId,
    },
    Index {
        dst: VariableId,
        base: VariableId,
        index: VariableId,
    },
    Member {
        dst: VariableId,
        base: VariableId,
        name: String,
    },
    Assignment {
        dst: VariableId,
        src: VariableId,
    },
    Push {
        array: VariableId,
        value: VariableId,
    },
    Pop {
        array: VariableId,
    },
    Delete {
        target: VariableId,
    },
    Length {
        dst: VariableId,
        target: VariableId,
    },
    Balance {
        dst: VariableId,
        target: VariableId,
    },
    CodeSize {
        dst: VariableId,
        target: VariableId,
    },
    TypeConversion {
        dst: VariableId,
        src: VariableId,
        to: Type,
    },
    Send {
        target: VariableId,
        amount: VariableId,
    },
    Transfer {
        target: VariableId,
        amount: VariableId,
    },
    Call(CallSite),
    /// Low-level call (`addr.call{value: v}(data)`).
    LowLevelCall(CallSite),
    /// `new Contract(args)` constructor call.
    NewContract(CallSite),
    /// `new T[](n)` dynamic array allocation.
    NewArray(CallSite),
    /// Struct literal `Foo({a: 1, b: 2})`.
    NewStructure(CallSite),
    EventCall {
        name: String,
        args: Vec<VariableId>,
    },
    Return {
        values: Vec<VariableId>,
    },
    Condition {
        var: VariableId,
    },
    InitArray {
        dst: VariableId,
        values: Vec<VariableId>,
    },
    Phi {
        dst: VariableId,
        operands: Vec<VariableId>,
    },
    Nop,
}

#[derive(Debug, Default, Clone)]
pub struct OpStream {
    pub ops: Vec<Op>,
    pub variables: IndexMap<VariableId, Variable>,
}
