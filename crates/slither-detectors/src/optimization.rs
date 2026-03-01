//! Optimization detectors (gas / size).

use slither_core::{Confidence, Finding, Impact, Slither, StateMutability, Visibility};

use crate::util::{finding_for_function, finding_for_variable};
use crate::Detector;

pub fn register(out: &mut Vec<Box<dyn Detector + Send + Sync>>) {
    out.push(Box::new(CacheArrayLength));
    out.push(Box::new(ConstableStates));
    out.push(Box::new(ExternalFunction));
    out.push(Box::new(ImmutableStates));
    out.push(Box::new(VarReadUsingThis));
}

pub struct CacheArrayLength;
impl Detector for CacheArrayLength {
    fn argument(&self) -> &'static str { "cache-array-length" }
    fn help(&self) -> &'static str { "`.length` in loop header should be cached" }
    fn impact(&self) -> Impact { Impact::Optimization }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"for\s*\([^;]*;\s*[^;]*\.length\s*;").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} reads `.length` from a storage array in a loop header.", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

pub struct ConstableStates;
impl Detector for ConstableStates {
    fn argument(&self) -> &'static str { "constable-states" }
    fn help(&self) -> &'static str { "State variable could be declared constant" }
    fn impact(&self) -> Impact { Impact::Optimization }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for sv in &c.state_variables {
                if sv.is_constant || sv.is_immutable { continue; }
                if sv.initial_value.is_none() { continue; }
                let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name))).unwrap();
                let mutated = c.functions.iter().any(|f| re.is_match(&f.body_excerpt));
                if !mutated {
                    out.push(finding_for_variable(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} is never modified — declare it `constant`.", c.name, sv.name),
                        &sv.name, sv.source.clone()));
                }
            }
        }
        out
    }
}

pub struct ExternalFunction;
impl Detector for ExternalFunction {
    fn argument(&self) -> &'static str { "external-function" }
    fn help(&self) -> &'static str { "Public function never internally called could be external" }
    fn impact(&self) -> Impact { Impact::Optimization }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for f in &c.functions {
                if !matches!(f.visibility, Visibility::Public) { continue; }
                let re = regex::Regex::new(&format!(r"\b{}\s*\(", regex::escape(&f.name))).unwrap();
                let internally_called = c
                    .functions
                    .iter()
                    .filter(|other| other.name != f.name)
                    .any(|other| re.is_match(&other.body_excerpt));
                if !internally_called {
                    out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} is public but never called internally — could be `external`.", c.name, f.name),
                        &c.name, &f.signature(), f.source.clone()));
                }
            }
        }
        out
    }
}

pub struct ImmutableStates;
impl Detector for ImmutableStates {
    fn argument(&self) -> &'static str { "immutable-states" }
    fn help(&self) -> &'static str { "State variable could be declared immutable" }
    fn impact(&self) -> Impact { Impact::Optimization }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for sv in &c.state_variables {
                if sv.is_constant || sv.is_immutable { continue; }
                let re = regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name))).unwrap();
                let writes_outside_constructor = c.functions.iter().any(|f| {
                    !matches!(f.kind, slither_core::FunctionKind::Constructor)
                        && re.is_match(&f.body_excerpt)
                });
                let writes_in_constructor = c
                    .functions
                    .iter()
                    .any(|f| matches!(f.kind, slither_core::FunctionKind::Constructor) && re.is_match(&f.body_excerpt));
                if !writes_outside_constructor && writes_in_constructor {
                    out.push(finding_for_variable(self.argument(), self.impact(), self.confidence(),
                        format!("{}.{} is only written in the constructor — declare it `immutable`.", c.name, sv.name),
                        &sv.name, sv.source.clone()));
                }
            }
        }
        out
    }
}

pub struct VarReadUsingThis;
impl Detector for VarReadUsingThis {
    fn argument(&self) -> &'static str { "var-read-using-this" }
    fn help(&self) -> &'static str { "Reading own state via `this.var` wastes gas" }
    fn impact(&self) -> Impact { Impact::Optimization }
    fn confidence(&self) -> Confidence { Confidence::High }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(r"\bthis\.[A-Za-z_]\w*").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if re.is_match(&f.body_excerpt) {
                out.push(finding_for_function(self.argument(), self.impact(), self.confidence(),
                    format!("{}.{} reads own state via `this.` (external call to self).", c.name, f.name),
                    &c.name, &f.signature(), f.source.clone()));
            }
        }
        out
    }
}

#[allow(dead_code)]
fn _unused(_m: StateMutability) {}
