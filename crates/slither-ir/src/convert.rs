//! Lift `slither-core::Function` → [`FunctionIr`].
//!
//! This is the AST → SlithIR pass. We do best-effort lowering of the
//! function body excerpt: detector logic that needs deep IR will look at
//! `Function::body_excerpt` and `Function.has_assembly` for now.

use slither_core::Function;

use crate::operations::{Op, Variable, VariableId};
use crate::types::Type;
use crate::FunctionIr;

#[derive(Debug, Default)]
pub struct IrBuilder {
    next_var: u32,
}

impl IrBuilder {
    pub fn fresh(&mut self, name: impl Into<String>, typ: Type) -> Variable {
        self.next_var += 1;
        Variable {
            id: VariableId(self.next_var),
            name: name.into(),
            typ,
            is_state: false,
            is_parameter: false,
            is_return: false,
            is_constant: false,
            is_temporary: true,
        }
    }
}

/// Lift one Solidity function into an IR stub. Real lowering is a follow-up.
pub fn lift_function(f: &Function) -> FunctionIr {
    let mut ir = FunctionIr::new(f.full_name());
    let mut b = IrBuilder::default();
    for p in &f.parameters {
        let mut v = b.fresh(&p.name, Type::parse_solc_type_string(&p.type_str));
        v.is_parameter = true;
        v.is_temporary = false;
        ir.variables.insert(v.id, v);
    }
    for r in &f.returns {
        let mut v = b.fresh(
            if r.name.is_empty() { "ret".to_owned() } else { r.name.clone() },
            Type::parse_solc_type_string(&r.type_str),
        );
        v.is_return = true;
        v.is_temporary = false;
        ir.variables.insert(v.id, v);
    }
    ir.ops.push(Op::Nop);
    ir
}
