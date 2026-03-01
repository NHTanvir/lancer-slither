//! High-impact detectors. 15 entries — all exploit-grade issues that should
//! cause `--fail-high` CI gates to block a merge.

use slither_core::{Confidence, Finding, FunctionKind, Impact, Slither, Visibility};

use crate::util::{
    body_contains, finding_for_contract, finding_for_function, finding_for_variable,
    is_publicly_callable, passes_path_filter,
};
use crate::Detector;

/// Push every high-impact detector into `out`.
pub fn register(out: &mut Vec<Box<dyn Detector + Send + Sync>>) {
    out.push(Box::new(AbiEncoderv2Array));
    out.push(Box::new(ArbitrarySendErc20));
    out.push(Box::new(ArrayByReference));
    out.push(Box::new(EncodePackedCollision));
    out.push(Box::new(IncorrectShift));
    out.push(Box::new(MultipleConstructors));
    out.push(Box::new(NameReused));
    out.push(Box::new(ProtectedVars));
    out.push(Box::new(PublicMappingsNested));
    out.push(Box::new(Rtlo));
    out.push(Box::new(ShadowingState));
    out.push(Box::new(Suicidal));
    out.push(Box::new(UninitializedState));
    out.push(Box::new(UninitializedStorage));
    out.push(Box::new(UnprotectedUpgrade));
}

// ---------------------------------------------------------------------------
// abiencoderv2-array
// ---------------------------------------------------------------------------

pub struct AbiEncoderv2Array;
impl Detector for AbiEncoderv2Array {
    fn argument(&self) -> &'static str {
        "abiencoderv2-array"
    }
    fn help(&self) -> &'static str {
        "Storage abiencoderv2 array bug (Solidity < 0.5.10)"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn wiki_title(&self) -> &'static str {
        "Storage ABIEncoderV2 array"
    }
    fn wiki_description(&self) -> &'static str {
        "`solc` versions `0.4.7-0.5.9` contain a compiler bug leading to incorrect ABI encoder output."
    }
    fn wiki_exploit_scenario(&self) -> &'static str {
        "A contract uses pragma `pragma experimental ABIEncoderV2` and encodes a storage array of structs; the resulting calldata is malformed."
    }
    fn wiki_recommendation(&self) -> &'static str {
        "Use Solidity 0.5.10 or later."
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for su in slither.user_source_units() {
            if !passes_path_filter(slither, &slither_core::source::SourceLocation::whole_file(
                &su.path, Some(&slither.project_root),
            )) {
                continue;
            }
            if su.content.contains("pragma experimental ABIEncoderV2") {
                if let Some(p) = &su.pragma {
                    if affected_pragma(p) {
                        out.push(finding_for_contract(
                            self.argument(),
                            self.impact(),
                            self.confidence(),
                            format!(
                                "{} uses ABIEncoderV2 under a vulnerable solc version ({}).",
                                su.relative_path.display(),
                                p
                            ),
                            su.contracts.first().map(|c| c.name.as_str()).unwrap_or(""),
                            slither_core::source::SourceLocation::whole_file(
                                &su.path,
                                Some(&slither.project_root),
                            ),
                        ));
                    }
                }
            }
        }
        out
    }
}

fn affected_pragma(p: &str) -> bool {
    let trim = p.replace(' ', "").to_ascii_lowercase();
    trim.contains("0.4.") || trim.contains("0.5.0") || trim.contains("0.5.1") ||
    trim.contains("0.5.2") || trim.contains("0.5.3") || trim.contains("0.5.4") ||
    trim.contains("0.5.5") || trim.contains("0.5.6") || trim.contains("0.5.7") ||
    trim.contains("0.5.8") || trim.contains("0.5.9")
}

// ---------------------------------------------------------------------------
// arbitrary-send-erc20
// ---------------------------------------------------------------------------

pub struct ArbitrarySendErc20;
impl Detector for ArbitrarySendErc20 {
    fn argument(&self) -> &'static str {
        "arbitrary-send-erc20"
    }
    fn help(&self) -> &'static str {
        "transferFrom with arbitrary `from` address"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn wiki_title(&self) -> &'static str {
        "Arbitrary `from` in transferFrom"
    }
    fn wiki_description(&self) -> &'static str {
        "Calls `IERC20.transferFrom` with a `from` argument that is not `msg.sender` allows the caller to steal tokens of any user who has approved the contract."
    }
    fn wiki_exploit_scenario(&self) -> &'static str {
        "Alice approves `Vault` for unlimited USDC. Bob calls `Vault.swap(amount, from=alice)` and steals her allowance."
    }
    fn wiki_recommendation(&self) -> &'static str {
        "Hard-code `msg.sender` as the `from` parameter of `transferFrom`."
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re = regex::Regex::new(
            r"\.transferFrom\s*\(\s*(?P<from>[A-Za-z_]\w*)\s*,",
        )
        .expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if !passes_path_filter(slither, &f.source) {
                continue;
            }
            for cap in re.captures_iter(&f.body_excerpt) {
                let from = &cap["from"];
                if from == "msg" {
                    continue;
                }
                let param = f.parameters.iter().any(|p| p.name == from);
                let modifier_guard = f
                    .modifiers
                    .iter()
                    .any(|m| m.contains("only") || m.contains("Auth"));
                if param && !modifier_guard {
                    out.push(finding_for_function(
                        self.argument(),
                        self.impact(),
                        self.confidence(),
                        format!(
                            "{}.{} uses transferFrom with arbitrary `from={}` (no access gate).",
                            c.name, f.name, from
                        ),
                        &c.name,
                        &f.signature(),
                        f.source.clone(),
                    ));
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// array-by-reference
// ---------------------------------------------------------------------------

pub struct ArrayByReference;
impl Detector for ArrayByReference {
    fn argument(&self) -> &'static str {
        "array-by-reference"
    }
    fn help(&self) -> &'static str {
        "Modifying storage array by value (memory)"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for p in &f.parameters {
                if p.type_str.ends_with("[]") && matches!(p.data_location, slither_core::model::DataLocation::Memory)
                {
                    let used_for_storage = c.state_variables.iter().any(|sv| {
                        sv.type_str.contains(p.type_str.trim_end_matches(' '))
                    });
                    if used_for_storage {
                        out.push(finding_for_function(
                            self.argument(),
                            self.impact(),
                            self.confidence(),
                            format!(
                                "{}.{} takes a memory copy of an array shape used as storage in {} — writes are silently dropped.",
                                c.name, f.name, c.name
                            ),
                            &c.name,
                            &f.signature(),
                            f.source.clone(),
                        ));
                    }
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// encode-packed-collision
// ---------------------------------------------------------------------------

pub struct EncodePackedCollision;
impl Detector for EncodePackedCollision {
    fn argument(&self) -> &'static str {
        "encode-packed-collision"
    }
    fn help(&self) -> &'static str {
        "abi.encodePacked of dynamic types with collision potential"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let re =
            regex::Regex::new(r"abi\.encodePacked\s*\(([^)]+)\)").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for cap in re.captures_iter(&f.body_excerpt) {
                let args = &cap[1];
                let dyn_count = ["string", "bytes", "[]"].iter().map(|t| args.matches(t).count()).sum::<usize>();
                if dyn_count >= 2 {
                    out.push(finding_for_function(
                        self.argument(),
                        self.impact(),
                        self.confidence(),
                        format!(
                            "{}.{} calls abi.encodePacked with multiple dynamic arguments — collision risk.",
                            c.name, f.name
                        ),
                        &c.name,
                        &f.signature(),
                        f.source.clone(),
                    ));
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// incorrect-shift
// ---------------------------------------------------------------------------

pub struct IncorrectShift;
impl Detector for IncorrectShift {
    fn argument(&self) -> &'static str {
        "incorrect-shift"
    }
    fn help(&self) -> &'static str {
        "Shift operand order swapped"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        // Pattern: literal `<<` variable, e.g. `2 << x` where shift count
        // and value got swapped. We look for `\d+\s*<<\s*[A-Za-z_]` and
        // `\d+\s*>>\s*[A-Za-z_]`.
        let re = regex::Regex::new(r"\b(\d+)\s*(?:<<|>>)\s*([A-Za-z_]\w*)").expect("regex");
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            for _ in re.captures_iter(&f.body_excerpt) {
                out.push(finding_for_function(
                    self.argument(),
                    self.impact(),
                    self.confidence(),
                    format!(
                        "{}.{} has a constant value on the LHS of a shift — likely operand order bug.",
                        c.name, f.name
                    ),
                    &c.name,
                    &f.signature(),
                    f.source.clone(),
                ));
                break;
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// multiple-constructors
// ---------------------------------------------------------------------------

pub struct MultipleConstructors;
impl Detector for MultipleConstructors {
    fn argument(&self) -> &'static str {
        "multiple-constructors"
    }
    fn help(&self) -> &'static str {
        "Pre-0.4.22 dual constructor pattern"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            let has_constructor = c.functions.iter().any(|f| matches!(f.kind, FunctionKind::Constructor));
            let has_legacy = c.functions.iter().any(|f| f.name == c.name && matches!(f.kind, FunctionKind::Function));
            if has_constructor && has_legacy {
                out.push(finding_for_contract(
                    self.argument(),
                    self.impact(),
                    self.confidence(),
                    format!(
                        "Contract {} declares both a `constructor` and a legacy `function {}` — only one will run.",
                        c.name, c.name
                    ),
                    &c.name,
                    c.source.clone(),
                ));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// name-reused
// ---------------------------------------------------------------------------

pub struct NameReused;
impl Detector for NameReused {
    fn argument(&self) -> &'static str {
        "name-reused"
    }
    fn help(&self) -> &'static str {
        "Two contracts in workspace share a name"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (name, refs) in &slither.contracts_by_name {
            if refs.len() > 1 {
                for (i, (si, ci)) in refs.iter().enumerate() {
                    let c = &slither.source_units[*si].contracts[*ci];
                    out.push(finding_for_contract(
                        self.argument(),
                        self.impact(),
                        self.confidence(),
                        format!(
                            "Contract name `{}` is reused (definition #{}).",
                            name,
                            i + 1
                        ),
                        name,
                        c.source.clone(),
                    ));
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// protected-vars
// ---------------------------------------------------------------------------

pub struct ProtectedVars;
impl Detector for ProtectedVars {
    fn argument(&self) -> &'static str {
        "protected-vars"
    }
    fn help(&self) -> &'static str {
        "Critical state variable written without protective modifier"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let sensitive = ["owner", "admin", "implementation", "logic", "_owner", "_admin"];
        for c in slither.all_contracts() {
            for sv in &c.state_variables {
                if !sensitive.iter().any(|n| sv.name.eq_ignore_ascii_case(n)) {
                    continue;
                }
                for f in &c.functions {
                    if !is_publicly_callable(f.visibility) {
                        continue;
                    }
                    if f.modifiers.iter().any(|m| {
                        m.contains("only")
                            || m.contains("Owner")
                            || m.contains("Admin")
                            || m.contains("Auth")
                    }) {
                        continue;
                    }
                    let pattern =
                        regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name))).unwrap();
                    if pattern.is_match(&f.body_excerpt) {
                        out.push(finding_for_function(
                            self.argument(),
                            self.impact(),
                            self.confidence(),
                            format!(
                                "{}.{} writes critical state variable `{}` without an access modifier.",
                                c.name, f.name, sv.name
                            ),
                            &c.name,
                            &f.signature(),
                            f.source.clone(),
                        ));
                    }
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// public-mappings-nested
// ---------------------------------------------------------------------------

pub struct PublicMappingsNested;
impl Detector for PublicMappingsNested {
    fn argument(&self) -> &'static str {
        "public-mappings-nested"
    }
    fn help(&self) -> &'static str {
        "Public nested mapping not ABI-encodable"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for sv in &c.state_variables {
                if matches!(sv.visibility, Visibility::Public)
                    && sv.type_str.matches("mapping(").count() >= 2
                {
                    out.push(finding_for_variable(
                        self.argument(),
                        self.impact(),
                        self.confidence(),
                        format!(
                            "{}.{} is a public nested mapping — solc < 0.5.0 cannot generate its getter.",
                            c.name, sv.name
                        ),
                        &sv.name,
                        sv.source.clone(),
                    ));
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// rtlo
// ---------------------------------------------------------------------------

pub struct Rtlo;
impl Detector for Rtlo {
    fn argument(&self) -> &'static str {
        "rtlo"
    }
    fn help(&self) -> &'static str {
        "Right-to-left override unicode hidden in source"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let needle = '\u{202E}';
        for su in slither.user_source_units() {
            if su.content.contains(needle) {
                let _ = body_contains;
                let line = su
                    .content
                    .lines()
                    .enumerate()
                    .find(|(_, l)| l.contains(needle))
                    .map(|(i, _)| i + 1)
                    .unwrap_or(0);
                let mut loc = slither_core::source::SourceLocation::whole_file(
                    &su.path,
                    Some(&slither.project_root),
                );
                if line > 0 {
                    loc.lines = vec![line];
                }
                out.push(finding_for_contract(
                    self.argument(),
                    self.impact(),
                    self.confidence(),
                    format!(
                        "{} contains a right-to-left override unicode character (U+202E).",
                        su.relative_path.display()
                    ),
                    su.contracts.first().map(|c| c.name.as_str()).unwrap_or(""),
                    loc,
                ));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// shadowing-state
// ---------------------------------------------------------------------------

pub struct ShadowingState;
impl Detector for ShadowingState {
    fn argument(&self) -> &'static str {
        "shadowing-state"
    }
    fn help(&self) -> &'static str {
        "Child state variable shadows parent"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for parent in &c.inheritance {
                let Some(parents) = slither.contracts_by_name.get(parent) else {
                    continue;
                };
                for (si, ci) in parents {
                    let pc = &slither.source_units[*si].contracts[*ci];
                    for sv in &c.state_variables {
                        if pc.state_variables.iter().any(|p| p.name == sv.name) {
                            out.push(finding_for_variable(
                                self.argument(),
                                self.impact(),
                                self.confidence(),
                                format!(
                                    "{}.{} shadows {}.{} (state-var shadowing).",
                                    c.name, sv.name, parent, sv.name
                                ),
                                &sv.name,
                                sv.source.clone(),
                            ));
                        }
                    }
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// suicidal
// ---------------------------------------------------------------------------

pub struct Suicidal;
impl Detector for Suicidal {
    fn argument(&self) -> &'static str {
        "suicidal"
    }
    fn help(&self) -> &'static str {
        "Functions allowing anyone to destruct the contract"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn wiki_recommendation(&self) -> &'static str {
        "Protect access to `selfdestruct` with an owner-only modifier."
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let re = regex::Regex::new(r"\b(selfdestruct|suicide)\s*\(").expect("regex");
        for (c, f) in slither.user_functions() {
            if !is_publicly_callable(f.visibility) {
                continue;
            }
            if !re.is_match(&f.body_excerpt) {
                continue;
            }
            let guard = f.modifiers.iter().any(|m| {
                m.contains("only")
                    || m.contains("Owner")
                    || m.contains("Admin")
                    || m.contains("Auth")
            });
            let has_check = f.body_excerpt.contains("msg.sender ==")
                || f.body_excerpt.contains("require(msg.sender")
                || f.body_excerpt.contains("revert");
            if !guard && !has_check {
                out.push(finding_for_function(
                    self.argument(),
                    self.impact(),
                    self.confidence(),
                    format!(
                        "{}.{} calls selfdestruct without an access check — anyone can destroy the contract.",
                        c.name, f.name
                    ),
                    &c.name,
                    &f.signature(),
                    f.source.clone(),
                ));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// uninitialized-state
// ---------------------------------------------------------------------------

pub struct UninitializedState;
impl Detector for UninitializedState {
    fn argument(&self) -> &'static str {
        "uninitialized-state"
    }
    fn help(&self) -> &'static str {
        "State variable read before initialization"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in slither.all_contracts() {
            for sv in &c.state_variables {
                if sv.initial_value.is_some() {
                    continue;
                }
                if sv.is_constant || sv.is_immutable {
                    continue;
                }
                let constructor_inits = c
                    .functions
                    .iter()
                    .filter(|f| matches!(f.kind, FunctionKind::Constructor))
                    .any(|f| {
                        let p =
                            regex::Regex::new(&format!(r"\b{}\s*=", regex::escape(&sv.name)))
                                .unwrap();
                        p.is_match(&f.body_excerpt)
                    });
                if constructor_inits {
                    continue;
                }
                let read_anywhere = c.functions.iter().any(|f| {
                    let p =
                        regex::Regex::new(&format!(r"\b{}\b", regex::escape(&sv.name))).unwrap();
                    p.is_match(&f.body_excerpt)
                });
                if read_anywhere {
                    out.push(finding_for_variable(
                        self.argument(),
                        self.impact(),
                        self.confidence(),
                        format!(
                            "{}.{} is never initialized but is read.",
                            c.name, sv.name
                        ),
                        &sv.name,
                        sv.source.clone(),
                    ));
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// uninitialized-storage
// ---------------------------------------------------------------------------

pub struct UninitializedStorage;
impl Detector for UninitializedStorage {
    fn argument(&self) -> &'static str {
        "uninitialized-storage"
    }
    fn help(&self) -> &'static str {
        "Uninitialized local storage pointer (slot-0 alias)"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        let re = regex::Regex::new(
            r"\b[A-Za-z_]\w*\s+storage\s+([A-Za-z_]\w*)\s*;",
        )
        .expect("regex");
        for (c, f) in slither.user_functions() {
            for cap in re.captures_iter(&f.body_excerpt) {
                let name = &cap[1];
                out.push(finding_for_function(
                    self.argument(),
                    self.impact(),
                    self.confidence(),
                    format!(
                        "{}.{} declares uninitialized storage pointer `{}` (aliases slot 0).",
                        c.name, f.name, name
                    ),
                    &c.name,
                    &f.signature(),
                    f.source.clone(),
                ));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// unprotected-upgrade
// ---------------------------------------------------------------------------

pub struct UnprotectedUpgrade;
impl Detector for UnprotectedUpgrade {
    fn argument(&self) -> &'static str {
        "unprotected-upgrade"
    }
    fn help(&self) -> &'static str {
        "Upgrade function callable by anyone"
    }
    fn impact(&self) -> Impact {
        Impact::High
    }
    fn confidence(&self) -> Confidence {
        Confidence::High
    }
    fn detect(&self, slither: &Slither) -> Vec<Finding> {
        let mut out = Vec::new();
        for (c, f) in slither.user_functions() {
            if !is_publicly_callable(f.visibility) {
                continue;
            }
            let is_upgrade = f.name.to_lowercase().contains("upgrade")
                || f.name.to_lowercase().contains("setimplementation")
                || f.name == "initialize";
            if !is_upgrade {
                continue;
            }
            let guard = f.modifiers.iter().any(|m| {
                m.contains("only")
                    || m.contains("Owner")
                    || m.contains("Admin")
                    || m.contains("Auth")
                    || m.contains("initializer")
            }) || f.body_excerpt.contains("require(msg.sender")
                || f.body_excerpt.contains("revert(");
            if !guard {
                out.push(finding_for_function(
                    self.argument(),
                    self.impact(),
                    self.confidence(),
                    format!(
                        "{}.{} can be called by anyone to upgrade the contract.",
                        c.name, f.name
                    ),
                    &c.name,
                    &f.signature(),
                    f.source.clone(),
                ));
            }
        }
        out
    }
}
