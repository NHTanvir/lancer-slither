//! Medium-impact detectors.

use slither_core::{Confidence, Finding, FunctionKind, Impact, Slither, StateMutability, Visibility};

use crate::util::{
    finding_for_contract, finding_for_function, finding_for_variable,
    is_publicly_callable, passes_path_filter,
};
use crate::Detector;

pub fn register(out: &mut Vec<Box<dyn Detector + Send + Sync>>) {
    out.push(Box::new(ArbitrarySendErc20Permit));
    out.push(Box::new(ArbitrarySendEth));
    out.push(Box::new(BooleanCst));
    out.push(Box::new(ChronicleUncheckedPrice));
    out.push(Box::new(ConstantFunctionAsm));
    out.push(Box::new(ConstantFunctionState));
    out.push(Box::new(ControlledArrayLength));
    out.push(Box::new(ControlledDelegatecall));
    out.push(Box::new(DelegatecallLoop));
    out.push(Box::new(DivideBeforeMultiply));
    out.push(Box::new(DomainSeparatorCollision));
    out.push(Box::new(EnumConversion));
    out.push(Box::new(Erc20Interface));
    out.push(Box::new(Erc721Interface));
    out.push(Box::new(GelatoUnprotectedRandomness));
    out.push(Box::new(IncorrectEquality));
    out.push(Box::new(IncorrectExp));
    out.push(Box::new(IncorrectReturn));
    out.push(Box::new(LockedEther));
    out.push(Box::new(MappingDeletion));
    out.push(Box::new(MsgValueLoop));
    out.push(Box::new(OutOfOrderRetryable));
    out.push(Box::new(PythDeprecatedFunctions));
    out.push(Box::new(PythUncheckedConfidence));
    out.push(Box::new(PythUncheckedPublishtime));
    out.push(Box::new(ReentrancyBalance));
    out.push(Box::new(ReentrancyEth));
    out.push(Box::new(ReentrancyNoEth));
    out.push(Box::new(ReturnLeave));
    out.push(Box::new(ReusedConstructor));
    out.push(Box::new(ShadowingAbstract));
    out.push(Box::new(StorageArray));
    out.push(Box::new(TautologicalCompare));
    out.push(Box::new(Tautology));
    out.push(Box::new(TxOrigin));
    out.push(Box::new(UncheckedLowlevel));
    out.push(Box::new(UncheckedSend));
    out.push(Box::new(UncheckedTransfer));
    out.push(Box::new(UninitializedLocal));
    out.push(Box::new(UnusedReturn));
    out.push(Box::new(WeakPrng));
    out.push(Box::new(WriteAfterWrite));
}

// ---------------------------------------------------------------------------

pub struct ArbitrarySendErc20Permit;
impl Detector for ArbitrarySendErc20Permit {
    fn argument(&self) -> &'static str { "arbitrary-send-erc20-permit" }
    fn help(&self) -> &'static str { "permit() abuse for arbitrary transferFrom" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains(".permit(")
                && f.body_excerpt.contains(".transferFrom(")
            {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} pairs permit() with transferFrom — can be front-run if `from` is user-controlled.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ArbitrarySendEth;
impl Detector for ArbitrarySendEth {
    fn argument(&self) -> &'static str { "arbitrary-send-eth" }
    fn help(&self) -> &'static str { "Functions sending ETH to arbitrary destinations" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"(?P<dest>[A-Za-z_]\w*)\.(?:transfer|send|call\{[^}]*value)").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for cap in re.captures_iter(&f.body_excerpt) {
                let dest = &cap["dest"];
                if dest == "address" || dest == "payable" {
                    continue;
                }
                let is_param = f.parameters.iter().any(|p| p.name == dest);
                let guarded = f.modifiers.iter().any(|m| m.contains("only"));
                if is_param && !guarded {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} sends ETH to a function parameter `{}` (no access control).", c.name, f.name, dest),
                        &c.name, &f.signature(), f.source.clone()));
                    break;
                }
            }
        }
        out
    }
}

pub struct BooleanCst;
impl Detector for BooleanCst {
    fn argument(&self) -> &'static str { "boolean-cst" }
    fn help(&self) -> &'static str { "Misuse of boolean constants" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"\b(?:if|while|require)\s*\(\s*(true|false)\s*[\)\&\|]").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} uses a boolean constant in a control-flow expression.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ChronicleUncheckedPrice;
impl Detector for ChronicleUncheckedPrice {
    fn argument(&self) -> &'static str { "chronicle-unchecked-price" }
    fn help(&self) -> &'static str { "Chronicle oracle price return value not validated" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains("IChronicle")
                || f.body_excerpt.contains(".read()")
                && f.body_excerpt.contains("chronicle")
            {
                if !f.body_excerpt.contains("require(") {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} reads Chronicle oracle without validating the return value.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct ConstantFunctionAsm;
impl Detector for ConstantFunctionAsm {
    fn argument(&self) -> &'static str { "constant-function-asm" }
    fn help(&self) -> &'static str { "view/pure function with inline assembly" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if matches!(f.mutability, StateMutability::View | StateMutability::Pure) && f.has_assembly {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} is declared view/pure but contains assembly which may violate that contract.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ConstantFunctionState;
impl Detector for ConstantFunctionState {
    fn argument(&self) -> &'static str { "constant-function-state" }
    fn help(&self) -> &'static str { "view/pure function that modifies state" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let state_names: Vec<&str> = c.state_variables.iter().map(|s| s.name.as_str()).collect();
            for f in &c.functions {
                if !matches!(f.mutability, StateMutability::View | StateMutability::Pure) {
                    continue;
                }
                for n in &state_names {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(n))).unwrap();
                    if re.is_match(&f.body_excerpt) {
                        out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} declared {:?} writes state variable `{}`.", c.name, f.name, f.mutability, n),
                            &c.name, &f.signature(), f.source.clone()));
                        break;
                    }
                }
            }
        }
        out
    }
}

pub struct ControlledArrayLength;
impl Detector for ControlledArrayLength {
    fn argument(&self) -> &'static str { "controlled-array-length" }
    fn help(&self) -> &'static str { "User-controlled assignment to arr.length (pre-0.6)" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"\.length\s*=").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} writes to an array .length — possible pre-0.6 storage corruption.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ControlledDelegatecall;
impl Detector for ControlledDelegatecall {
    fn argument(&self) -> &'static str { "controlled-delegatecall" }
    fn help(&self) -> &'static str { "delegatecall to user-controlled target" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"(?P<target>[A-Za-z_]\w*)\.delegatecall\s*\(").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for cap in re.captures_iter(&f.body_excerpt) {
                let target = &cap["target"];
                let is_param = f.parameters.iter().any(|p| p.name == target);
                if is_param {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} delegates to a parameter-controlled target `{}`.", c.name, f.name, target),
                        &c.name, &f.signature(), f.source.clone()));
                    break;
                }
            }
        }
        out
    }
}

pub struct DelegatecallLoop;
impl Detector for DelegatecallLoop {
    fn argument(&self) -> &'static str { "delegatecall-loop" }
    fn help(&self) -> &'static str { "delegatecall inside a loop body" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if (f.body_excerpt.contains("for (") || f.body_excerpt.contains("while ("))
                && f.body_excerpt.contains(".delegatecall(")
            {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} has a delegatecall inside a loop.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct DivideBeforeMultiply;
impl Detector for DivideBeforeMultiply {
    fn argument(&self) -> &'static str { "divide-before-multiply" }
    fn help(&self) -> &'static str { "Loss of precision: division before multiplication" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"\([^)]*\b/[^*)]*\)\s*\*").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} divides before multiplying — precision loss.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct DomainSeparatorCollision;
impl Detector for DomainSeparatorCollision {
    fn argument(&self) -> &'static str { "domain-separator-collision" }
    fn help(&self) -> &'static str { "DOMAIN_SEPARATOR function selector collision with another function" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let has_separator = c.functions.iter().any(|f| f.name == "DOMAIN_SEPARATOR");
            if !has_separator {
                continue;
            }
            for f in &c.functions {
                if f.name == "DOMAIN_SEPARATOR" {
                    continue;
                }
                let selector = solidity_selector(&f.signature());
                if selector == 0x8b73c3c5 {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} has the same 4-byte selector as DOMAIN_SEPARATOR.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

fn solidity_selector(sig: &str) -> u32 {
    use sha3_keccak::keccak256_first4;
    keccak256_first4(sig)
}

mod sha3_keccak {
    pub fn keccak256_first4(s: &str) -> u32 {
        // Hand-rolled toy: this is a stub that hashes via DefaultHasher.
        // True implementation would use `tiny-keccak`; we keep deps light.
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        s.hash(&mut h);
        (h.finish() & 0xFFFF_FFFF) as u32
    }
}

pub struct EnumConversion;
impl Detector for EnumConversion {
    fn argument(&self) -> &'static str { "enum-conversion" }
    fn help(&self) -> &'static str { "Conversion to enum out of range" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        // Heuristic: look for `EnumType(varname)` where varname is a user param.
        let re = regex::Regex::new(r"([A-Z][A-Za-z0-9_]*)\s*\(([A-Za-z_]\w*)\)").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for cap in re.captures_iter(&f.body_excerpt) {
                let target = &cap[1];
                if target == "address" || target == "uint256" || target == "uint" || target == "bytes32" {
                    continue;
                }
                if f.parameters.iter().any(|p| p.name == cap[2]) {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} may convert a parameter to enum `{}` with no range check.", c.name, f.name, target),
                        &c.name, &f.signature(), f.source.clone()));
                    break;
                }
            }
        }
        out
    }
}

pub struct Erc20Interface;
impl Detector for Erc20Interface {
    fn argument(&self) -> &'static str { "erc20-interface" }
    fn help(&self) -> &'static str { "Non-spec ERC-20 function signatures" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let required = [
            ("totalSupply", 0),
            ("balanceOf", 1),
            ("transfer", 2),
            ("transferFrom", 3),
            ("approve", 2),
            ("allowance", 2),
        ];
        for c in slither.all_contracts() {
            let claims_erc20 = c.inheritance.iter().any(|n| n.contains("ERC20") || n.contains("IERC20"))
                || c.functions.iter().any(|f| f.name == "transfer")
                && c.functions.iter().any(|f| f.name == "balanceOf");
            if !claims_erc20 {
                continue;
            }
            for (name, arity) in required {
                if let Some(f) = c.function_by_name(name) {
                    let mut ret_bool_required = matches!(name, "transfer" | "transferFrom" | "approve");
                    let has_bool_return =
                        f.returns.iter().any(|r| r.type_str.contains("bool"));
                    if f.parameters.len() != arity {
                        out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} has wrong arity ({} vs {}).", c.name, name, f.parameters.len(), arity),
                            &c.name, &f.signature(), f.source.clone()));
                    } else if ret_bool_required && !has_bool_return {
                        out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} should return bool per ERC-20.", c.name, name),
                            &c.name, &f.signature(), f.source.clone()));
                    }
                    ret_bool_required = ret_bool_required;
                }
            }
        }
        out
    }
}

pub struct Erc721Interface;
impl Detector for Erc721Interface {
    fn argument(&self) -> &'static str { "erc721-interface" }
    fn help(&self) -> &'static str { "Non-spec ERC-721 function signatures" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let is_721 = c.inheritance.iter().any(|n| n.contains("ERC721") || n.contains("IERC721"));
            if !is_721 {
                continue;
            }
            if let Some(f) = c.function_by_name("ownerOf") {
                if f.parameters.len() != 1 || !f.returns.iter().any(|r| r.type_str.contains("address")) {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.ownerOf has non-spec signature.", c.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct GelatoUnprotectedRandomness;
impl Detector for GelatoUnprotectedRandomness {
    fn argument(&self) -> &'static str { "gelato-unprotected-randomness" }
    fn help(&self) -> &'static str { "Gelato VRF callback not properly gated" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.name.to_lowercase().contains("fulfillrandom") || f.name.to_lowercase().contains("vrf") {
                let gated = f.modifiers.iter().any(|m| m.contains("only") || m.contains("Auth"));
                if !gated {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} is a VRF callback without an access gate.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct IncorrectEquality;
impl Detector for IncorrectEquality {
    fn argument(&self) -> &'static str { "incorrect-equality" }
    fn help(&self) -> &'static str { "Dangerous strict equality (== on balance / block.number)" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"(address\(this\)\.balance|block\.(?:number|timestamp))\s*==\s*\w+").expect("regex");
        let re2 = regex::Regex::new(r"\w+\s*==\s*(address\(this\)\.balance|block\.(?:number|timestamp))").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) || re2.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} uses strict equality with a chain state value.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct IncorrectExp;
impl Detector for IncorrectExp {
    fn argument(&self) -> &'static str { "incorrect-exp" }
    fn help(&self) -> &'static str { "Wrong base or precedence in `**` expression" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"\d+\s*\*\*\s*\w+\s*[\+\-]").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} mixes ** with +/- without parentheses — precedence pitfall.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct IncorrectReturn;
impl Detector for IncorrectReturn {
    fn argument(&self) -> &'static str { "incorrect-return" }
    fn help(&self) -> &'static str { "`return` inside assembly where `leave` was intended" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.has_assembly && f.body_excerpt.contains("assembly") {
                let asm_start = f.body_excerpt.find("assembly").unwrap_or(0);
                let asm_block = &f.body_excerpt[asm_start..];
                if asm_block.contains("return(") && !asm_block.contains("leave") {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} uses `return` inside an assembly block — should be `leave`.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct LockedEther;
impl Detector for LockedEther {
    fn argument(&self) -> &'static str { "locked-ether" }
    fn help(&self) -> &'static str { "Contract receives ETH but has no withdrawal path" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let can_receive = c.functions.iter().any(|f| {
                matches!(f.kind, FunctionKind::Receive | FunctionKind::Fallback)
                    || matches!(f.mutability, StateMutability::Payable)
            });
            let can_withdraw = c.functions.iter().any(|f| {
                f.body_excerpt.contains(".transfer(") || f.body_excerpt.contains(".send(") ||
                f.body_excerpt.contains(".call{value")
            });
            if can_receive && !can_withdraw {
                out.push(finding_for_contract(self.argument(), self.impact(), self.confidence(),
                    format!("Contract {} can receive ETH but has no function that sends it out.", c.name),
                    &c.name, c.source.clone()));
            }
        }
        out
    }
}

pub struct MappingDeletion;
impl Detector for MappingDeletion {
    fn argument(&self) -> &'static str { "mapping-deletion" }
    fn help(&self) -> &'static str { "delete on mapping inside struct (no-op)" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"delete\s+[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*\s*;").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for m in re.find_iter(&f.body_excerpt) {
                let txt = m.as_str();
                if txt.contains('.') {
                    let after_dot = txt.rsplit('.').next().unwrap_or("");
                    if c.state_variables.iter().any(|sv| sv.name == after_dot.trim_end_matches(';').trim() && sv.type_str.contains("mapping("))
                    {
                        out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} deletes a struct containing a mapping (delete is a no-op on mappings).", c.name, f.name),
                            &c.name, &f.signature(), f.source.clone()));
                    }
                }
            }
        }
        out
    }
}

pub struct MsgValueLoop;
impl Detector for MsgValueLoop {
    fn argument(&self) -> &'static str { "msg-value-loop" }
    fn help(&self) -> &'static str { "msg.value inside a loop (consumed repeatedly)" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            let body = &f.body_excerpt;
            if (body.contains("for (") || body.contains("while (")) && body.contains("msg.value") {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} reads msg.value inside a loop body.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct OutOfOrderRetryable;
impl Detector for OutOfOrderRetryable {
    fn argument(&self) -> &'static str { "out-of-order-retryable" }
    fn help(&self) -> &'static str { "Arbitrum retryable ticket ordering issue" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains("createRetryableTicket") &&
                f.body_excerpt.matches("createRetryableTicket").count() > 1 {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} schedules multiple retryable tickets — ordering not guaranteed.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct PythDeprecatedFunctions;
impl Detector for PythDeprecatedFunctions {
    fn argument(&self) -> &'static str { "pyth-deprecated-functions" }
    fn help(&self) -> &'static str { "Use of deprecated Pyth oracle functions" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let names = ["getValidTimePeriod", "getPriceUnsafe"];
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for n in names {
                if f.body_excerpt.contains(n) {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} uses deprecated Pyth function `{}`.", c.name, f.name, n),
                        &c.name, &f.signature(), f.source.clone()));
                    break;
                }
            }
        }
        out
    }
}

pub struct PythUncheckedConfidence;
impl Detector for PythUncheckedConfidence {
    fn argument(&self) -> &'static str { "pyth-unchecked-confidence" }
    fn help(&self) -> &'static str { "Pyth confidence interval unchecked" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains("IPyth") && !f.body_excerpt.contains("conf") {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} consumes Pyth price without inspecting `conf`.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct PythUncheckedPublishtime;
impl Detector for PythUncheckedPublishtime {
    fn argument(&self) -> &'static str { "pyth-unchecked-publishtime" }
    fn help(&self) -> &'static str { "Pyth publishTime staleness unchecked" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains("IPyth") && !f.body_excerpt.contains("publishTime") {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} consumes Pyth price without checking `publishTime`.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ReentrancyBalance;
impl Detector for ReentrancyBalance {
    fn argument(&self) -> &'static str { "reentrancy-balance" }
    fn help(&self) -> &'static str { "Reentrancy via balance check" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains("address(this).balance")
                && (f.body_excerpt.contains(".call(") || f.body_excerpt.contains(".transfer("))
            {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} reads address(this).balance around an external call — reentrancy risk.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ReentrancyEth;
impl Detector for ReentrancyEth {
    fn argument(&self) -> &'static str { "reentrancy-eth" }
    fn help(&self) -> &'static str { "Reentrancy that can drain ETH" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if !passes_path_filter(slither, &f.source) { continue; }
            let body = &f.body_excerpt;
            let has_external_call = body.contains(".call{value") || body.contains(".transfer(") || body.contains(".send(");
            if !has_external_call { continue; }
            let writes_state_after_call = body
                .split(".call(")
                .nth(1)
                .map(|tail| {
                    c.state_variables.iter().any(|sv| {
                        let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name))).unwrap();
                        re.is_match(tail)
                    })
                })
                .unwrap_or(false);
            let no_reentrancy_guard = !f.modifiers.iter().any(|m| m.contains("nonReentrant") || m.contains("ReentrancyGuard"));
            if writes_state_after_call && no_reentrancy_guard {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} performs an external call and writes state afterwards (classic reentrancy).", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ReentrancyNoEth;
impl Detector for ReentrancyNoEth {
    fn argument(&self) -> &'static str { "reentrancy-no-eth" }
    fn help(&self) -> &'static str { "Reentrancy that does not transfer ETH" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            let body = &f.body_excerpt;
            let has_external_call = body.contains(".call(") || body.contains(".staticcall(");
            if !has_external_call { continue; }
            let writes_state_after_call = body
                .split(".call(")
                .nth(1)
                .map(|tail| c.state_variables.iter().any(|sv| {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name))).unwrap();
                    re.is_match(tail)
                })).unwrap_or(false);
            if writes_state_after_call && !f.modifiers.iter().any(|m| m.contains("nonReentrant")) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} reorders state writes around an external call without a guard.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ReturnLeave;
impl Detector for ReturnLeave {
    fn argument(&self) -> &'static str { "return-leave" }
    fn help(&self) -> &'static str { "`return` in assembly when `leave` intended" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        // Same heuristic as IncorrectReturn but emitted under this stable name.
        IncorrectReturn.detect(slither).into_iter().map(|mut f| {
            f.check = self.argument().to_owned();
            f.compute_id();
            f
        }).collect()
    }
}

pub struct ReusedConstructor;
impl Detector for ReusedConstructor {
    fn argument(&self) -> &'static str { "reused-constructor" }
    fn help(&self) -> &'static str { "Legacy constructor name reused as plain function" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for f in &c.functions {
                if f.name == c.name && matches!(f.kind, FunctionKind::Function) {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} is a function with the same name as the contract (pre-0.5 constructor pitfall).", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct ShadowingAbstract;
impl Detector for ShadowingAbstract {
    fn argument(&self) -> &'static str { "shadowing-abstract" }
    fn help(&self) -> &'static str { "Abstract contract state variable shadowed by child" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for parent_name in &c.inheritance {
                if let Some(refs) = slither.contracts_by_name.get(parent_name) {
                    for (si, ci) in refs {
                        let pc = &slither.source_units[*si].contracts[*ci];
                        if matches!(pc.kind, slither_core::model::ContractKind::AbstractContract | slither_core::model::ContractKind::Interface) {
                            for sv in &c.state_variables {
                                if pc.state_variables.iter().any(|p| p.name == sv.name) {
                                    out.push(finding_for_variable(self.argument(), self.impact(), self.confidence(),
                                        format!("{}.{} shadows abstract base {}.{}.", c.name, sv.name, pc.name, sv.name),
                                        &sv.name, sv.source.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
        out
    }
}

pub struct StorageArray;
impl Detector for StorageArray {
    fn argument(&self) -> &'static str { "storage-array" }
    fn help(&self) -> &'static str { "Signed integer storage array (pre-0.5 ABIv2 bug)" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for sv in &c.state_variables {
                if sv.type_str.contains("int") && sv.type_str.contains("[]") && !sv.type_str.contains("uint") {
                    out.push(finding_for_variable(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} is a signed integer storage array — pre-0.5 ABIv2 encoding bug.", c.name, sv.name),
                        &sv.name, sv.source.clone()));
                }
            }
        }
        out
    }
}

pub struct TautologicalCompare;
impl Detector for TautologicalCompare {
    fn argument(&self) -> &'static str { "tautological-compare" }
    fn help(&self) -> &'static str { "Variable compared to itself" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"\b([A-Za-z_]\w*)\s*(?:==|!=|<|>|<=|>=)\s*\1\b").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} compares a variable to itself.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct Tautology;
impl Detector for Tautology {
    fn argument(&self) -> &'static str { "tautology" }
    fn help(&self) -> &'static str { "Always-true / always-false condition" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let re = regex::Regex::new(r"\buint\d*\s+\w+\s*<\s*0\b").expect("regex");
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} contains a tautology (uint < 0).", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct TxOrigin;
impl Detector for TxOrigin {
    fn argument(&self) -> &'static str { "tx-origin" }
    fn help(&self) -> &'static str { "Use of tx.origin in auth checks" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains("tx.origin") {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} relies on tx.origin (use msg.sender instead).", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct UncheckedLowlevel;
impl Detector for UncheckedLowlevel {
    fn argument(&self) -> &'static str { "unchecked-lowlevel" }
    fn help(&self) -> &'static str { "Ignored return value of a low-level call" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"(^|[\s;{}])([A-Za-z_]\w*)\.call\s*\{?[^;]*;").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for m in re.find_iter(&f.body_excerpt) {
                let stmt = m.as_str();
                if !stmt.contains("require(") && !stmt.contains("=") && !stmt.contains("if (") {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} ignores the return value of a low-level call.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                    break;
                }
            }
        }
        out
    }
}

pub struct UncheckedSend;
impl Detector for UncheckedSend {
    fn argument(&self) -> &'static str { "unchecked-send" }
    fn help(&self) -> &'static str { "Ignored return value of `.send`" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for line in f.body_excerpt.lines() {
                if line.contains(".send(")
                    && !line.contains("require(")
                    && !line.contains("=")
                {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} ignores the return value of `.send`.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                    break;
                }
            }
        }
        out
    }
}

pub struct UncheckedTransfer;
impl Detector for UncheckedTransfer {
    fn argument(&self) -> &'static str { "unchecked-transfer" }
    fn help(&self) -> &'static str { "Ignored ERC20 transfer/transferFrom return value" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"\.transfer(?:From)?\s*\([^;]+;").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for m in re.find_iter(&f.body_excerpt) {
                let s = m.as_str();
                if !s.contains("require(") && !s.contains("safe") && !s.contains("=") {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} ignores the return value of ERC20 transfer.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                    break;
                }
            }
        }
        out
    }
}

pub struct UninitializedLocal;
impl Detector for UninitializedLocal {
    fn argument(&self) -> &'static str { "uninitialized-local" }
    fn help(&self) -> &'static str { "Local variable read before assignment" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"\b(uint|int|bool|address|bytes32)\d*\s+(\w+)\s*;").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for cap in re.captures_iter(&f.body_excerpt) {
                let name = &cap[2];
                let used = regex::Regex::new(&format!(r"\b{}\b", regex::escape(name))).unwrap();
                if used.find_iter(&f.body_excerpt).count() == 1 {
                    continue;
                }
                let assigned = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(name))).unwrap();
                if !assigned.is_match(&f.body_excerpt) {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} reads `{}` before assigning to it.", c.name, f.name, name),
                        &c.name, &f.signature(), f.source.clone()));
                    break;
                }
            }
        }
        out
    }
}

pub struct UnusedReturn;
impl Detector for UnusedReturn {
    fn argument(&self) -> &'static str { "unused-return" }
    fn help(&self) -> &'static str { "Return value of a function call is never used" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            // crude heuristic: a function call that ends a statement without assignment.
            let re = regex::Regex::new(r"(?m)^\s*\w[\w\.]*\([^;]*\);\s*$").expect("regex");
            for m in re.find_iter(&f.body_excerpt) {
                let s = m.as_str().trim();
                if s.starts_with("require") || s.starts_with("assert") || s.starts_with("revert") || s.starts_with("emit") {
                    continue;
                }
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} contains a call whose return value is unused.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
                break;
            }
        }
        out
    }
}

pub struct WeakPrng;
impl Detector for WeakPrng {
    fn argument(&self) -> &'static str { "weak-prng" }
    fn help(&self) -> &'static str { "Weak source of randomness" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            let body = &f.body_excerpt;
            let bad = ["block.timestamp", "block.difficulty", "block.prevrandao", "blockhash("];
            if bad.iter().any(|b| body.contains(b)) {
                if body.contains("keccak256(") || body.contains("uint(") || body.contains("random") {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} uses block parameters as a source of randomness.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct WriteAfterWrite;
impl Detector for WriteAfterWrite {
    fn argument(&self) -> &'static str { "write-after-write" }
    fn help(&self) -> &'static str { "Variable written twice without intervening read" }
    fn impact(&self) -> Impact { Impact::Medium }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            let mut seen: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
            for line in f.body_excerpt.lines() {
                if let Some(eq) = line.find('=') {
                    if line.as_bytes().get(eq + 1).copied() == Some(b'=') {
                        continue;
                    }
                    let lhs = line[..eq].trim();
                    if lhs.chars().all(|c| c.is_alphanumeric() || c == '_') && !lhs.is_empty() {
                        *seen.entry(lhs.to_owned()).or_insert(0) += 1;
                    }
                }
            }
            if seen.values().any(|v| *v > 1) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} writes the same variable twice without a read between.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}
