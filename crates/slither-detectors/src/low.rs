//! Low-impact detectors.

use slither_core::{Confidence, Finding, FunctionKind, Impact, Slither, Visibility};

use crate::util::{finding_for_function, finding_for_variable, is_publicly_callable};
use crate::Detector;

pub fn register(out: &mut Vec<Box<dyn Detector + Send + Sync>>) {
    out.push(Box::new(CallsLoop));
    out.push(Box::new(ChainlinkFeedRegistry));
    out.push(Box::new(EventsAccess));
    out.push(Box::new(EventsMaths));
    out.push(Box::new(IncorrectModifier));
    out.push(Box::new(IncorrectUnary));
    out.push(Box::new(MissingZeroCheck));
    out.push(Box::new(OptimismDeprecation));
    out.push(Box::new(ReentrancyBenign));
    out.push(Box::new(ReentrancyEvents));
    out.push(Box::new(ReturnBomb));
    out.push(Box::new(ShadowingBuiltin));
    out.push(Box::new(ShadowingLocal));
    out.push(Box::new(Timestamp));
    out.push(Box::new(UninitializedFptrCst));
    out.push(Box::new(VariableScope));
    out.push(Box::new(VoidCst));
}

pub struct CallsLoop;
impl Detector for CallsLoop {
    fn argument(&self) -> &'static str { "calls-loop" }
    fn help(&self) -> &'static str { "External call inside a loop body (DOS)" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            let body = &f.body_excerpt;
            if (body.contains("for (") || body.contains("while ("))
                && (body.contains(".call(") || body.contains(".transfer(") || body.contains(".send("))
            {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} contains an external call inside a loop.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ChainlinkFeedRegistry;
impl Detector for ChainlinkFeedRegistry {
    fn argument(&self) -> &'static str { "chainlink-feed-registry" }
    fn help(&self) -> &'static str { "Chainlink Feed Registry usage on non-mainnet chain" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains("FeedRegistryInterface") || f.body_excerpt.contains("ChainlinkFeedRegistry") {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} uses Chainlink Feed Registry (mainnet-only).", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct EventsAccess;
impl Detector for EventsAccess {
    fn argument(&self) -> &'static str { "events-access" }
    fn help(&self) -> &'static str { "Access-control state change without event emission" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let mut access_vars = std::collections::BTreeSet::new();
        for n in ["owner", "admin", "_owner", "_admin", "manager", "guardian", "controller"] {
            access_vars.insert(n);
        }
        for c in slither.all_contracts() {
            let writes: Vec<&str> = c.state_variables.iter().filter(|sv| access_vars.contains(&sv.name.as_str())).map(|sv| sv.name.as_str()).collect();
            for f in &c.functions {
                if !is_publicly_callable(f.visibility) { continue; }
                for w in &writes {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(w))).unwrap();
                    if re.is_match(&f.body_excerpt) && !f.body_excerpt.contains("emit ") {
                        out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} sets access variable `{}` without emitting an event.", c.name, f.name, w),
                            &c.name, &f.signature(), f.source.clone()));
                        break;
                    }
                }
            }
        }
        out
    }
}

pub struct EventsMaths;
impl Detector for EventsMaths {
    fn argument(&self) -> &'static str { "events-maths" }
    fn help(&self) -> &'static str { "Math-affecting state change without event emission" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let math_keywords = ["fee", "rate", "ratio", "limit", "max", "min", "threshold", "supply"];
        for c in slither.all_contracts() {
            let math_state: Vec<&str> = c
                .state_variables
                .iter()
                .filter(|sv| math_keywords.iter().any(|k| sv.name.to_lowercase().contains(k)))
                .map(|s| s.name.as_str())
                .collect();
            for f in &c.functions {
                if !is_publicly_callable(f.visibility) { continue; }
                for w in &math_state {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(w))).unwrap();
                    if re.is_match(&f.body_excerpt) && !f.body_excerpt.contains("emit ") {
                        out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} updates math-relevant state `{}` without emitting an event.", c.name, f.name, w),
                            &c.name, &f.signature(), f.source.clone()));
                        break;
                    }
                }
            }
        }
        out
    }
}

pub struct IncorrectModifier;
impl Detector for IncorrectModifier {
    fn argument(&self) -> &'static str { "incorrect-modifier" }
    fn help(&self) -> &'static str { "Modifier without `_;` placeholder in all paths" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for m in &c.modifiers {
                if !m.body_excerpt.contains("_;") {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("Modifier {}.{} never executes its `_;` placeholder.", c.name, m.name),
                        &c.name, &m.name, m.source.clone()));
                }
            }
        }
        out
    }
}

pub struct IncorrectUnary;
impl Detector for IncorrectUnary {
    fn argument(&self) -> &'static str { "incorrect-unary" }
    fn help(&self) -> &'static str { "`=+` typed where `+=` intended" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"=\s*\+\s*\w").expect("regex");
        let re2 = regex::Regex::new(r"=\s*-\s*\w").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) || re2.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} contains a `=+` / `=-` expression that may be a typo for `+=` / `-=`.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct MissingZeroCheck;
impl Detector for MissingZeroCheck {
    fn argument(&self) -> &'static str { "missing-zero-check" }
    fn help(&self) -> &'static str { "address setter without zero-address validation" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            let addr_params: Vec<&str> = f
                .parameters
                .iter()
                .filter(|p| p.type_str.contains("address"))
                .map(|p| p.name.as_str())
                .collect();
            for p in addr_params {
                if !f.body_excerpt.contains(&format!("{} != address(0)", p))
                    && !f.body_excerpt.contains(&format!("{}!=address(0)", p))
                {
                    let re = regex::Regex::new(&format!(r"\b\w+\s*=\s*{}", regex::escape(p))).unwrap();
                    if re.is_match(&f.body_excerpt) {
                        out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} assigns parameter `{}` to state without zero-address check.", c.name, f.name, p),
                            &c.name, &f.signature(), f.source.clone()));
                        break;
                    }
                }
            }
        }
        out
    }
}

pub struct OptimismDeprecation;
impl Detector for OptimismDeprecation {
    fn argument(&self) -> &'static str { "optimism-deprecation" }
    fn help(&self) -> &'static str { "Use of deprecated Optimism predeploy" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let deprecated = ["0x4200000000000000000000000000000000000000", "OVM_StateManager", "OVM_L2CrossDomainMessenger"];
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for d in deprecated {
                if f.body_excerpt.contains(d) {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} references deprecated Optimism predeploy `{}`.", c.name, f.name, d),
                        &c.name, &f.signature(), f.source.clone()));
                    break;
                }
            }
        }
        out
    }
}

pub struct ReentrancyBenign;
impl Detector for ReentrancyBenign {
    fn argument(&self) -> &'static str { "reentrancy-benign" }
    fn help(&self) -> &'static str { "State write after external call (same contract)" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            let body = &f.body_excerpt;
            if (body.contains(".call(") || body.contains(".staticcall("))
                && !f.modifiers.iter().any(|m| m.contains("nonReentrant"))
            {
                if c.state_variables.iter().any(|sv| {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name))).unwrap();
                    re.is_match(body)
                }) {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} writes state after an external call (potentially benign).", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct ReentrancyEvents;
impl Detector for ReentrancyEvents {
    fn argument(&self) -> &'static str { "reentrancy-events" }
    fn help(&self) -> &'static str { "Event emitted after external call" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            let body = &f.body_excerpt;
            if let Some(call_pos) = body.find(".call(") {
                let tail = &body[call_pos..];
                if tail.contains("emit ") {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} emits an event after an external call.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct ReturnBomb;
impl Detector for ReturnBomb {
    fn argument(&self) -> &'static str { "return-bomb" }
    fn help(&self) -> &'static str { "Callee can revert via huge returndata" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains(".call(") && !f.body_excerpt.contains("returndatasize") {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} performs a low-level call without bounding returndata size.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ShadowingBuiltin;
impl Detector for ShadowingBuiltin {
    fn argument(&self) -> &'static str { "shadowing-builtin" }
    fn help(&self) -> &'static str { "Identifier shadows a Solidity built-in" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let builtins = ["assert", "require", "revert", "block", "msg", "tx", "now", "this", "keccak256", "sha3"];
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for sv in &c.state_variables {
                if builtins.contains(&sv.name.as_str()) {
                    out.push(finding_for_variable(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} shadows a Solidity built-in symbol.", c.name, sv.name),
                        &sv.name, sv.source.clone()));
                }
            }
        }
        out
    }
}

pub struct ShadowingLocal;
impl Detector for ShadowingLocal {
    fn argument(&self) -> &'static str { "shadowing-local" }
    fn help(&self) -> &'static str { "Local variable shadows outer state/parameter" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let state_names: Vec<&str> = c.state_variables.iter().map(|sv| sv.name.as_str()).collect();
            for f in &c.functions {
                for sn in &state_names {
                    let re = regex::Regex::new(&format!(r"\b(?:uint|int|bool|address|bytes)\d*\s+{}\b", regex::escape(sn))).unwrap();
                    if re.is_match(&f.body_excerpt) {
                        out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} declares a local that shadows state variable `{}`.", c.name, f.name, sn),
                            &c.name, &f.signature(), f.source.clone()));
                        break;
                    }
                }
            }
        }
        out
    }
}

pub struct Timestamp;
impl Detector for Timestamp {
    fn argument(&self) -> &'static str { "timestamp" }
    fn help(&self) -> &'static str { "block.timestamp used as condition" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains("block.timestamp") {
                let used_in_cond = f.body_excerpt.contains("require(") || f.body_excerpt.contains("if (");
                if used_in_cond {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} uses block.timestamp as a control-flow input.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct UninitializedFptrCst;
impl Detector for UninitializedFptrCst {
    fn argument(&self) -> &'static str { "uninitialized-fptr-cst" }
    fn help(&self) -> &'static str { "Uninitialized function-pointer call in constructor (Solidity <0.5.8)" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for f in &c.functions {
                if matches!(f.kind, FunctionKind::Constructor) && f.body_excerpt.contains("function (") {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} may call an uninitialized function pointer (pre-0.5.8 solc bug).", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct VariableScope;
impl Detector for VariableScope {
    fn argument(&self) -> &'static str { "variable-scope" }
    fn help(&self) -> &'static str { "Local variable declaration scope ambiguity" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Low }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let _ = slither;
        Vec::new()
    }
}

pub struct VoidCst;
impl Detector for VoidCst {
    fn argument(&self) -> &'static str { "void-cst" }
    fn help(&self) -> &'static str { "Empty constructor that should explicitly call inherited" }
    fn impact(&self) -> Impact { Impact::Low }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            if c.inheritance.is_empty() { continue; }
            for f in &c.functions {
                if matches!(f.kind, FunctionKind::Constructor) && f.body_excerpt.trim().ends_with("{}") {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} is an empty constructor — explicit inherited constructor calls may be missing.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

#[allow(dead_code)]
fn _unused(_v: Visibility) {}
