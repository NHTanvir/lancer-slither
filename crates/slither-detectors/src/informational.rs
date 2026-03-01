//! Informational detectors. Code-quality flags; never fail CI by themselves.

use slither_core::{Confidence, Finding, FunctionKind, Impact, Slither, StateMutability, Visibility};

use crate::util::{finding_for_contract, finding_for_function, finding_for_variable, is_publicly_callable};
use crate::Detector;

pub fn register(out: &mut Vec<Box<dyn Detector + Send + Sync>>) {
    out.push(Box::new(Assembly));
    out.push(Box::new(AssertStateChange));
    out.push(Box::new(BooleanEqual));
    out.push(Box::new(CostlyLoop));
    out.push(Box::new(CyclomaticComplexity));
    out.push(Box::new(DeadCode));
    out.push(Box::new(DeprecatedStandards));
    out.push(Box::new(Erc20Indexed));
    out.push(Box::new(FunctionInitState));
    out.push(Box::new(IncorrectUsingFor));
    out.push(Box::new(LowLevelCalls));
    out.push(Box::new(MissingInheritance));
    out.push(Box::new(NamingConvention));
    out.push(Box::new(Pragma));
    out.push(Box::new(RedundantStatements));
    out.push(Box::new(ReentrancyUnlimitedGas));
    out.push(Box::new(SolcVersion));
    out.push(Box::new(TooManyDigits));
    out.push(Box::new(UnimplementedFunctions));
    out.push(Box::new(UnindexedEventAddress));
    out.push(Box::new(UnusedState));
    out.push(Box::new(Tod));
}

pub struct Assembly;
impl Detector for Assembly {
    fn argument(&self) -> &'static str { "assembly" }
    fn help(&self) -> &'static str { "Function contains inline assembly" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.has_assembly {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} contains inline assembly.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct AssertStateChange;
impl Detector for AssertStateChange {
    fn argument(&self) -> &'static str { "assert-state-change" }
    fn help(&self) -> &'static str { "`assert()` with side effects" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let re = regex::Regex::new(r"assert\(\s*[^)]*=\s*[^)]+\)").expect("regex");
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} contains an `assert` with an assignment side-effect.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct BooleanEqual;
impl Detector for BooleanEqual {
    fn argument(&self) -> &'static str { "boolean-equal" }
    fn help(&self) -> &'static str { "Comparison to `true` / `false`" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"==\s*(true|false)\b").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} compares directly to true/false.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct CostlyLoop;
impl Detector for CostlyLoop {
    fn argument(&self) -> &'static str { "costly-loop" }
    fn help(&self) -> &'static str { "SSTORE inside loop body" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let state_names: Vec<&str> = c.state_variables.iter().map(|s| s.name.as_str()).collect();
            for f in &c.functions {
                let body = &f.body_excerpt;
                if !(body.contains("for (") || body.contains("while (")) { continue; }
                for n in &state_names {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(n))).unwrap();
                    if re.is_match(body) {
                        out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} writes state variable `{}` inside a loop (SSTORE per iteration).", c.name, f.name, n),
                            &c.name, &f.signature(), f.source.clone()));
                        break;
                    }
                }
            }
        }
        out
    }
}

pub struct CyclomaticComplexity;
impl Detector for CyclomaticComplexity {
    fn argument(&self) -> &'static str { "cyclomatic-complexity" }
    fn help(&self) -> &'static str { "Function with high McCabe complexity" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            let cc = compute_cc(&f.body_excerpt);
            if cc > 11 {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} has cyclomatic complexity {} (threshold 11).", c.name, f.name, cc),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

fn compute_cc(body: &str) -> usize {
    1 + body.matches(" if ").count()
        + body.matches(" else if ").count()
        + body.matches(" for ").count()
        + body.matches(" while ").count()
        + body.matches(" case ").count()
        + body.matches(" && ").count()
        + body.matches(" || ").count()
}

pub struct DeadCode;
impl Detector for DeadCode {
    fn argument(&self) -> &'static str { "dead-code" }
    fn help(&self) -> &'static str { "Function never reachable from any public entrypoint" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let referenced: std::collections::BTreeSet<&str> = c
                .functions
                .iter()
                .flat_map(|f| {
                    let re = regex::Regex::new(r"\b([A-Za-z_]\w*)\s*\(").expect("regex");
                    re.captures_iter(&f.body_excerpt)
                        .filter_map(|cap| cap.get(1).map(|m| m.as_str()))
                        .collect::<Vec<_>>()
                })
                .collect();
            for f in &c.functions {
                if matches!(f.visibility, Visibility::Internal | Visibility::Private)
                    && !referenced.contains(f.name.as_str())
                    && !matches!(f.kind, FunctionKind::Constructor | FunctionKind::Fallback | FunctionKind::Receive)
                {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} is never referenced.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct DeprecatedStandards;
impl Detector for DeprecatedStandards {
    fn argument(&self) -> &'static str { "deprecated-standards" }
    fn help(&self) -> &'static str { "Use of deprecated keywords (sha3, throw, suicide, …)" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let bad = ["sha3(", "throw;", "suicide(", "callcode(", "var "];
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for b in bad {
                if f.body_excerpt.contains(b) {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} uses deprecated form `{}`.", c.name, f.name, b),
                        &c.name, &f.signature(), f.source.clone()));
                    break;
                }
            }
        }
        out
    }
}

pub struct Erc20Indexed;
impl Detector for Erc20Indexed {
    fn argument(&self) -> &'static str { "erc20-indexed" }
    fn help(&self) -> &'static str { "ERC-20 event without indexed parameters" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let re = regex::Regex::new(r"event\s+(Transfer|Approval)\s*\(([^)]+)\)").expect("regex");
            for cap in re.captures_iter(c.source.filename_short.as_str().is_empty().then(|| "").unwrap_or(""))
            {
                let _ = cap;
            }
            if let Some(su_idx) = slither.contracts_by_name.get(&c.name).and_then(|v| v.first()) {
                let su = &slither.source_units[su_idx.0];
                for cap in re.captures_iter(&su.content) {
                    let params = &cap[2];
                    if !params.contains("indexed") {
                        out.push(finding_for_contract(self.argument(), self.impact(), self.confidence(),
                            format!("Event `{}` in {} has no indexed parameters.", &cap[1], c.name),
                            &c.name, c.source.clone()));
                        break;
                    }
                }
            }
        }
        out
    }
}

pub struct FunctionInitState;
impl Detector for FunctionInitState {
    fn argument(&self) -> &'static str { "function-init-state" }
    fn help(&self) -> &'static str { "State var initialised via fn call at declaration" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for sv in &c.state_variables {
                if let Some(v) = &sv.initial_value {
                    if v.contains('(') && v.contains(')') {
                        out.push(finding_for_variable(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} is initialised via a function call at declaration.", c.name, sv.name),
                            &sv.name, sv.source.clone()));
                    }
                }
            }
        }
        out
    }
}

pub struct IncorrectUsingFor;
impl Detector for IncorrectUsingFor {
    fn argument(&self) -> &'static str { "incorrect-using-for" }
    fn help(&self) -> &'static str { "`using L for T` mismatch" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let re = regex::Regex::new(r"using\s+(\w+)\s+for\s+(\w+)").expect("regex");
        for su in slither.user_source_units() {
            for cap in re.captures_iter(&su.content) {
                let lib = &cap[1];
                let typ = &cap[2];
                if !slither.contracts_by_name.contains_key(lib) {
                    out.push(finding_for_contract(self.argument(), self.impact(), self.confidence(),
                        format!("`using {} for {}` in {} references an unknown library.", lib, typ, su.relative_path.display()),
                        su.contracts.first().map(|c| c.name.as_str()).unwrap_or(""),
                        slither_core::source::SourceLocation::whole_file(&su.path, Some(&slither.project_root))));
                }
            }
        }
        out
    }
}

pub struct LowLevelCalls;
impl Detector for LowLevelCalls {
    fn argument(&self) -> &'static str { "low-level-calls" }
    fn help(&self) -> &'static str { "Use of `addr.call()` / `delegatecall` / `staticcall`" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains(".call(") || f.body_excerpt.contains(".delegatecall(") || f.body_excerpt.contains(".staticcall(") {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} uses low-level calls.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct MissingInheritance;
impl Detector for MissingInheritance {
    fn argument(&self) -> &'static str { "missing-inheritance" }
    fn help(&self) -> &'static str { "Contract looks like it should inherit a standard interface" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let signals = ["transfer", "balanceOf", "approve", "allowance"];
            let matched: usize = signals.iter().filter(|s| c.function_by_name(s).is_some()).count();
            let inherits_erc20 = c.inheritance.iter().any(|n| n.contains("ERC20") || n.contains("IERC20"));
            if matched >= 3 && !inherits_erc20 {
                out.push(finding_for_contract(self.argument(), self.impact(), self.confidence(),
                    format!("Contract {} looks like ERC-20 but does not inherit IERC20.", c.name),
                    &c.name, c.source.clone()));
            }
        }
        out
    }
}

pub struct NamingConvention;
impl Detector for NamingConvention {
    fn argument(&self) -> &'static str { "naming-convention" }
    fn help(&self) -> &'static str { "Naming does not follow Solidity style guide" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            if !is_pascal_case(&c.name) {
                out.push(finding_for_contract(self.argument(), self.impact(), self.confidence(),
                    format!("Contract name `{}` is not PascalCase.", c.name),
                    &c.name, c.source.clone()));
            }
            for sv in &c.state_variables {
                if sv.is_constant && !is_screaming_snake(&sv.name) {
                    out.push(finding_for_variable(self.argument(), self.impact(), self.confidence(),
                        format!("Constant `{}` is not SCREAMING_SNAKE_CASE.", sv.name),
                        &sv.name, sv.source.clone()));
                } else if !sv.is_constant && !is_camel_case(&sv.name) && !sv.name.starts_with('_') {
                    out.push(finding_for_variable(self.argument(), self.impact(), self.confidence(),
                        format!("Variable `{}` is not mixedCase.", sv.name),
                        &sv.name, sv.source.clone()));
                }
            }
        }
        out
    }
}

fn is_pascal_case(s: &str) -> bool {
    s.chars().next().map(char::is_uppercase).unwrap_or(false) && !s.contains('_')
}
fn is_camel_case(s: &str) -> bool {
    s.chars().next().map(|c| c.is_lowercase() || c == '_').unwrap_or(false) && !s.contains('_')
}
fn is_screaming_snake(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

pub struct Pragma;
impl Detector for Pragma {
    fn argument(&self) -> &'static str { "pragma" }
    fn help(&self) -> &'static str { "Multiple solc pragma versions across files" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let pragmas: std::collections::BTreeSet<String> = slither
            .user_source_units()
            .filter_map(|su| su.pragma.clone())
            .collect();
        if pragmas.len() > 1 {
            let mut elements = Vec::new();
            for su in slither.user_source_units() {
                if let Some(_p) = &su.pragma {
                    elements.push(slither_core::FindingElement::contract(
                        su.contracts.first().map(|c| c.name.clone()).unwrap_or_default(),
                        slither_core::source::SourceLocation::whole_file(&su.path, Some(&slither.project_root)),
                    ));
                }
            }
            let f = crate::util::new_finding(self.argument(), self.impact(), self.confidence(),
                format!("Multiple pragma versions in the project: {:?}", pragmas),
                elements,
            );
            return vec![f];
        }
        Vec::new()
    }
}

pub struct RedundantStatements;
impl Detector for RedundantStatements {
    fn argument(&self) -> &'static str { "redundant-statements" }
    fn help(&self) -> &'static str { "Statements that have no effect" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let re = regex::Regex::new(r"(?m)^\s*\w+\s*;\s*$").expect("regex");
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} contains a statement with no effect.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ReentrancyUnlimitedGas;
impl Detector for ReentrancyUnlimitedGas {
    fn argument(&self) -> &'static str { "reentrancy-unlimited-gas" }
    fn help(&self) -> &'static str { "External low-level call passing all remaining gas" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if f.body_excerpt.contains(".call{value") || f.body_excerpt.contains(".call(") {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} uses an unlimited-gas low-level call.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct SolcVersion;
impl Detector for SolcVersion {
    fn argument(&self) -> &'static str { "solc-version" }
    fn help(&self) -> &'static str { "Solidity pragma uses a known-buggy or floating version" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for su in slither.user_source_units() {
            if let Some(p) = &su.pragma {
                if p.contains('^') || p.contains('>') {
                    out.push(finding_for_contract(self.argument(), self.impact(), self.confidence(),
                        format!("{} uses a floating pragma `{}`.", su.relative_path.display(), p),
                        su.contracts.first().map(|c| c.name.as_str()).unwrap_or(""),
                        slither_core::source::SourceLocation::whole_file(&su.path, Some(&slither.project_root))));
                }
            }
        }
        out
    }
}

pub struct TooManyDigits;
impl Detector for TooManyDigits {
    fn argument(&self) -> &'static str { "too-many-digits" }
    fn help(&self) -> &'static str { "Literal with many digits should use scientific notation" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"\b\d{7,}\b").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} uses a long numeric literal — consider scientific notation.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct UnimplementedFunctions;
impl Detector for UnimplementedFunctions {
    fn argument(&self) -> &'static str { "unimplemented-functions" }
    fn help(&self) -> &'static str { "Abstract function not implemented" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for f in &c.functions {
                if f.body_excerpt.trim().is_empty() && matches!(f.mutability, StateMutability::Nonpayable | StateMutability::Payable | StateMutability::View) {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} is declared without a body.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct UnindexedEventAddress;
impl Detector for UnindexedEventAddress {
    fn argument(&self) -> &'static str { "unindexed-event-address" }
    fn help(&self) -> &'static str { "Event has unindexed address parameter" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let re = regex::Regex::new(r"event\s+(\w+)\s*\(([^)]+)\)").expect("regex");
        for su in slither.user_source_units() {
            for cap in re.captures_iter(&su.content) {
                let params = &cap[2];
                let parts: Vec<&str> = params.split(',').collect();
                for part in parts {
                    if part.contains("address") && !part.contains("indexed") {
                        out.push(finding_for_contract(self.argument(), self.impact(), self.confidence(),
                            format!("event {} in {} has unindexed address parameter.", &cap[1], su.relative_path.display()),
                            su.contracts.first().map(|c| c.name.as_str()).unwrap_or(""),
                            slither_core::source::SourceLocation::whole_file(&su.path, Some(&slither.project_root))));
                        break;
                    }
                }
            }
        }
        out
    }
}

pub struct UnusedState;
impl Detector for UnusedState {
    fn argument(&self) -> &'static str { "unused-state" }
    fn help(&self) -> &'static str { "State variable declared but never used" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for sv in &c.state_variables {
                let re = regex::Regex::new(&format!(r"\b{}\b", regex::escape(&sv.name))).unwrap();
                let used = c.functions.iter().any(|f| re.is_match(&f.body_excerpt));
                if !used {
                    out.push(finding_for_variable(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} is never used.", c.name, sv.name),
                        &sv.name, sv.source.clone()));
                }
            }
        }
        out
    }
}

pub struct Tod;
impl Detector for Tod {
    fn argument(&self) -> &'static str { "tod" }
    fn help(&self) -> &'static str { "Transaction-order-dependence heuristic" }
    fn impact(&self) -> Impact { Impact::Informational }
    fn confidence(&self) -> Confidence { Confidence::Medium }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let state_names: Vec<&str> = c.state_variables.iter().map(|s| s.name.as_str()).collect();
            let mut public_writers: ahash::AHashMap<&str, Vec<&str>> = ahash::AHashMap::new();
            for f in &c.functions {
                if !is_publicly_callable(f.visibility) { continue; }
                for n in &state_names {
                    let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(n))).unwrap();
                    if re.is_match(&f.body_excerpt) {
                        public_writers.entry(*n).or_default().push(f.name.as_str());
                    }
                }
            }
            for (var, writers) in &public_writers {
                if writers.len() > 1 {
                    if let Some(f) = c.function_by_name(writers[0]) {
                        out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                            format!("{}.{} is one of {} public writers of `{}` — transaction order may matter.", c.name, f.name, writers.len(), var),
                            &c.name, &f.signature(), f.source.clone()));
                    }
                }
            }
        }
        out
    }
}
