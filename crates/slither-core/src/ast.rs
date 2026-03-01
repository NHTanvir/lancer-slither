//! Lift the solc *compact* AST (the `ast` key inside a solc combined-json or
//! `build-info` artifact) into our domain model.
//!
//! This module is **best-effort**: the upstream solc AST is huge and we only
//! extract what every detector / printer actually needs. Anything we don't
//! understand is preserved in `SourceUnit::ast_raw` so future detectors can
//! re-walk it.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::model::{
    Contract, ContractKind, DataLocation, Function, FunctionKind, Modifier, Parameter,
    SourceUnit, StateMutability, StateVariable, Visibility,
};
use crate::source::SourceLocation;

/// Lift a solc compact-AST `SourceUnit` JSON node into our [`SourceUnit`].
pub fn lift_source_unit(
    ast: &Value,
    file_path: &Path,
    project_root: Option<&Path>,
    content: &str,
) -> SourceUnit {
    let mut unit = SourceUnit {
        path: file_path.to_path_buf(),
        relative_path: project_root
            .and_then(|r| file_path.strip_prefix(r).ok())
            .map_or_else(|| file_path.to_path_buf(), Path::to_path_buf),
        content: content.to_owned(),
        pragma: None,
        contracts: Vec::new(),
        free_functions: Vec::new(),
        ast_id: ast.get("id").and_then(Value::as_u64).unwrap_or(0),
        imports: Vec::new(),
        ast_raw: Some(ast.clone()),
        storage_layout: None,
        source_map: None,
    };

    let Some(nodes) = ast.get("nodes").and_then(Value::as_array) else {
        return unit;
    };

    for node in nodes {
        match node.get("nodeType").and_then(Value::as_str) {
            Some("PragmaDirective") => {
                if let Some(literals) = node.get("literals").and_then(Value::as_array) {
                    let p: String = literals
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" ");
                    unit.pragma = Some(p);
                }
            }
            Some("ImportDirective") => {
                if let Some(file) = node.get("file").and_then(Value::as_str) {
                    unit.imports.push(file.to_owned());
                }
            }
            Some("ContractDefinition") => {
                if let Some(c) = lift_contract(node, file_path, project_root, content) {
                    unit.contracts.push(c);
                }
            }
            Some("FunctionDefinition") => {
                // free function (Solidity 0.7.1+)
                if let Some(f) =
                    lift_function(node, "", file_path, project_root, content)
                {
                    unit.free_functions.push(f);
                }
            }
            _ => {}
        }
    }

    unit
}

fn lift_contract(
    node: &Value,
    file_path: &Path,
    project_root: Option<&Path>,
    content: &str,
) -> Option<Contract> {
    let name = node.get("name").and_then(Value::as_str)?.to_owned();
    let is_abstract = node
        .get("abstract")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let kind_str = node
        .get("contractKind")
        .and_then(Value::as_str)
        .unwrap_or("contract");
    let kind = ContractKind::from_solc(kind_str, is_abstract);
    let src_str = node.get("src").and_then(Value::as_str).unwrap_or("0:0:0");
    let source = SourceLocation::from_solc_triplet(src_str, file_path, project_root, content)
        .unwrap_or_else(|| SourceLocation::whole_file(file_path, project_root));

    let inheritance = node
        .get("baseContracts")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|b| {
                    b.get("baseName")
                        .and_then(|n| n.get("name"))
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default();

    let linearized_base_contracts = node
        .get("linearizedBaseContracts")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_u64).collect())
        .unwrap_or_default();

    let mut state_variables = Vec::new();
    let mut functions = Vec::new();
    let mut modifiers = Vec::new();

    if let Some(children) = node.get("nodes").and_then(Value::as_array) {
        for c in children {
            match c.get("nodeType").and_then(Value::as_str) {
                Some("VariableDeclaration") => {
                    if let Some(v) = lift_state_var(c, file_path, project_root, content) {
                        state_variables.push(v);
                    }
                }
                Some("FunctionDefinition") => {
                    if let Some(f) = lift_function(c, &name, file_path, project_root, content) {
                        functions.push(f);
                    }
                }
                Some("ModifierDefinition") => {
                    if let Some(m) = lift_modifier(c, file_path, project_root, content) {
                        modifiers.push(m);
                    }
                }
                _ => {}
            }
        }
    }

    Some(Contract {
        name,
        kind,
        source,
        inheritance,
        linearized_base_contracts,
        state_variables,
        functions,
        modifiers,
        imports: Vec::new(),
        ast_id: node.get("id").and_then(Value::as_u64).unwrap_or(0),
        pragma: None,
    })
}

fn lift_state_var(
    node: &Value,
    file_path: &Path,
    project_root: Option<&Path>,
    content: &str,
) -> Option<StateVariable> {
    let name = node.get("name").and_then(Value::as_str)?.to_owned();
    let type_str = node
        .get("typeDescriptions")
        .and_then(|t| t.get("typeString"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    let visibility = Visibility::from_solc(
        node.get("visibility").and_then(Value::as_str).unwrap_or(""),
    );
    let is_constant = node.get("constant").and_then(Value::as_bool).unwrap_or(false);
    let is_immutable = node.get("mutability").and_then(Value::as_str) == Some("immutable");
    let src_str = node.get("src").and_then(Value::as_str).unwrap_or("0:0:0");
    let source = SourceLocation::from_solc_triplet(src_str, file_path, project_root, content)
        .unwrap_or_default();
    let initial_value = node
        .get("value")
        .and_then(|v| v.get("hexValue").or_else(|| v.get("value")))
        .and_then(Value::as_str)
        .map(str::to_owned);
    Some(StateVariable {
        name,
        type_str,
        visibility,
        is_constant,
        is_immutable,
        initial_value,
        source,
    })
}

fn lift_function(
    node: &Value,
    contract: &str,
    file_path: &Path,
    project_root: Option<&Path>,
    content: &str,
) -> Option<Function> {
    let kind_str = node.get("kind").and_then(Value::as_str).unwrap_or("function");
    let kind = match kind_str {
        "constructor" => FunctionKind::Constructor,
        "fallback" => FunctionKind::Fallback,
        "receive" => FunctionKind::Receive,
        "freeFunction" => FunctionKind::FreeFunction,
        _ => FunctionKind::Function,
    };
    let name = node
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_else(|| match kind {
            FunctionKind::Constructor => "constructor",
            FunctionKind::Fallback => "fallback",
            FunctionKind::Receive => "receive",
            _ => "",
        })
        .to_owned();
    let visibility = Visibility::from_solc(
        node.get("visibility").and_then(Value::as_str).unwrap_or(""),
    );
    let mutability = StateMutability::from_solc(
        node.get("stateMutability")
            .and_then(Value::as_str)
            .unwrap_or(""),
    );
    let is_virtual = node.get("virtual").and_then(Value::as_bool).unwrap_or(false);
    let parameters = node
        .get("parameters")
        .and_then(|p| p.get("parameters"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|n| lift_parameter(n, file_path, project_root, content))
                .collect()
        })
        .unwrap_or_default();
    let returns = node
        .get("returnParameters")
        .and_then(|p| p.get("parameters"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|n| lift_parameter(n, file_path, project_root, content))
                .collect()
        })
        .unwrap_or_default();
    let mods = node
        .get("modifiers")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    m.get("modifierName")
                        .and_then(|n| n.get("name"))
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default();
    let overrides = node
        .get("overrides")
        .and_then(|o| o.get("overrides"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("name").and_then(Value::as_str).map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let src_str = node.get("src").and_then(Value::as_str).unwrap_or("0:0:0");
    let source = SourceLocation::from_solc_triplet(src_str, file_path, project_root, content)
        .unwrap_or_default();
    let body_excerpt = extract_body_excerpt(content, &source);
    let has_assembly = body_excerpt.contains("assembly");
    Some(Function {
        name,
        kind,
        visibility,
        mutability,
        parameters,
        returns,
        modifiers: mods,
        source,
        contract: contract.to_owned(),
        body_excerpt,
        ast_id: node.get("id").and_then(Value::as_u64).unwrap_or(0),
        has_assembly,
        is_virtual,
        overrides,
    })
}

fn lift_parameter(
    node: &Value,
    file_path: &Path,
    project_root: Option<&Path>,
    content: &str,
) -> Option<Parameter> {
    let name = node.get("name").and_then(Value::as_str).unwrap_or("").to_owned();
    let type_str = node
        .get("typeDescriptions")
        .and_then(|t| t.get("typeString"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    let data_location =
        match node.get("storageLocation").and_then(Value::as_str).unwrap_or("default") {
            "storage" => DataLocation::Storage,
            "memory" => DataLocation::Memory,
            "calldata" => DataLocation::Calldata,
            _ => DataLocation::Unspecified,
        };
    let src_str = node.get("src").and_then(Value::as_str).unwrap_or("0:0:0");
    let source = SourceLocation::from_solc_triplet(src_str, file_path, project_root, content)
        .unwrap_or_default();
    let indexed = node.get("indexed").and_then(Value::as_bool).unwrap_or(false);
    Some(Parameter {
        name,
        type_str,
        data_location,
        source,
        indexed,
    })
}

fn lift_modifier(
    node: &Value,
    file_path: &Path,
    project_root: Option<&Path>,
    content: &str,
) -> Option<Modifier> {
    let name = node.get("name").and_then(Value::as_str)?.to_owned();
    let parameters = node
        .get("parameters")
        .and_then(|p| p.get("parameters"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|n| lift_parameter(n, file_path, project_root, content))
                .collect()
        })
        .unwrap_or_default();
    let src_str = node.get("src").and_then(Value::as_str).unwrap_or("0:0:0");
    let source = SourceLocation::from_solc_triplet(src_str, file_path, project_root, content)
        .unwrap_or_default();
    let body_excerpt = extract_body_excerpt(content, &source);
    Some(Modifier {
        name,
        parameters,
        source,
        body_excerpt,
    })
}

fn extract_body_excerpt(content: &str, src: &SourceLocation) -> String {
    if src.start + src.length > content.len() || src.length == 0 {
        return String::new();
    }
    content[src.start..src.start + src.length].to_owned()
}

/// Fallback: if we don't have a real solc AST, do a regex-based scrape of
/// `.sol` content to extract the top-level contracts/functions. Used by
/// single-file mode and by Etherscan flattened sources when solc is missing.
pub fn fallback_scrape(file_path: &Path, project_root: Option<&Path>, content: &str) -> SourceUnit {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static PRAGMA: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"pragma\s+solidity\s+([^;]+);").expect("pragma re"));
    static CONTRACT: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?ms)(?P<abstract>abstract\s+)?(?P<kind>contract|library|interface)\s+(?P<name>[A-Za-z_]\w*)(?:\s+is\s+(?P<bases>[^\{]+))?\s*\{",
        )
        .expect("contract re")
    });
    static FUNCTION: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?m)function\s+(?P<name>[A-Za-z_]\w*)\s*\(",
        )
        .expect("fn re")
    });

    let mut unit = SourceUnit {
        path: file_path.to_path_buf(),
        relative_path: project_root
            .and_then(|r| file_path.strip_prefix(r).ok())
            .map_or_else(|| file_path.to_path_buf(), Path::to_path_buf),
        content: content.to_owned(),
        pragma: PRAGMA
            .captures(content)
            .and_then(|c| c.name("pragma").or_else(|| c.get(1)).map(|m| m.as_str().to_owned())),
        contracts: Vec::new(),
        free_functions: Vec::new(),
        ast_id: 0,
        imports: Vec::new(),
        ast_raw: None,
        storage_layout: None,
        source_map: None,
    };

    for m in CONTRACT.captures_iter(content) {
        let name = m.name("name").map(|x| x.as_str().to_owned()).unwrap_or_default();
        let kind_str = m.name("kind").map(|x| x.as_str()).unwrap_or("contract");
        let is_abstract = m.name("abstract").is_some();
        let kind = ContractKind::from_solc(kind_str, is_abstract);
        let mat = m.get(0).expect("regex 0");
        let source = SourceLocation::from_span(
            mat.start(),
            mat.len(),
            file_path,
            project_root,
            content,
        );
        let inheritance = m
            .name("bases")
            .map(|b| {
                b.as_str()
                    .split(',')
                    .map(|s| s.trim().split_whitespace().next().unwrap_or("").to_owned())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let body_start = find_block(content, mat.end() - 1);
        let body_end = find_matching_close_brace(content, body_start);
        let body = &content.get(body_start..body_end).unwrap_or("");
        let functions = FUNCTION
            .captures_iter(body)
            .filter_map(|fm| {
                let fname = fm.name("name").map(|x| x.as_str().to_owned()).unwrap_or_default();
                let fmat = fm.get(0).expect("fn regex 0");
                let abs_start = body_start + fmat.start();
                Some(Function {
                    name: fname,
                    kind: FunctionKind::Function,
                    visibility: Visibility::Public,
                    mutability: StateMutability::Nonpayable,
                    parameters: Vec::new(),
                    returns: Vec::new(),
                    modifiers: Vec::new(),
                    source: SourceLocation::from_span(
                        abs_start,
                        fmat.len(),
                        file_path,
                        project_root,
                        content,
                    ),
                    contract: name.clone(),
                    body_excerpt: String::new(),
                    ast_id: 0,
                    has_assembly: false,
                    is_virtual: false,
                    overrides: Vec::new(),
                })
            })
            .collect();
        unit.contracts.push(Contract {
            name,
            kind,
            source,
            inheritance,
            linearized_base_contracts: Vec::new(),
            state_variables: Vec::new(),
            functions,
            modifiers: Vec::new(),
            imports: Vec::new(),
            ast_id: 0,
            pragma: unit.pragma.clone(),
        });
    }

    unit
}

fn find_block(s: &str, from: usize) -> usize {
    s.get(from..)
        .and_then(|tail| tail.find('{').map(|p| from + p + 1))
        .unwrap_or(from)
}

fn find_matching_close_brace(s: &str, from: usize) -> usize {
    let bytes = s.as_bytes();
    let mut depth = 1i64;
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

#[allow(dead_code)]
fn _unused(_: PathBuf) {}
