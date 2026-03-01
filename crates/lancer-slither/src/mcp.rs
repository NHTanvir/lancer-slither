//! MCP-server-shaped façade over the HTTP server.
//!
//! Implements the **Model Context Protocol** convention of "tools" and
//! "resources":
//!
//! * Each detector is a *tool* — Claude / Cursor call `tools/call` with the
//!   detector argument name and a target.
//! * Each printer is a *resource* — Claude / Cursor call `resources/read`
//!   with the printer name and a target.
//!
//! Transport: JSON-RPC over HTTP at `127.0.0.1:<port>`. The same port file
//! `%TEMP%/lancer-slither.port` advertises the endpoint to other Lancer
//! tools and to MCP-aware editors.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

/// Dispatch one MCP request.
pub fn handle(req: &JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id.clone().unwrap_or(Value::Null);
    let result = match req.method.as_str() {
        "initialize" => Some(initialize()),
        "tools/list" => Some(tools_list()),
        "tools/call" => tools_call(&req.params),
        "resources/list" => Some(resources_list()),
        "resources/read" => resources_read(&req.params),
        _ => None,
    };
    match result {
        Some(v) => JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(v),
            error: None,
        },
        None => JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("unknown method: {}", req.method),
            }),
        },
    }
}

fn initialize() -> Value {
    serde_json::json!({
        "protocolVersion": "2025-03-26",
        "serverInfo": {
            "name": "lancer-slither",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "capabilities": {
            "tools": { "listChanged": false },
            "resources": { "listChanged": false, "subscribe": false },
        },
    })
}

fn tools_list() -> Value {
    let tools: Vec<Value> = slither_detectors::all()
        .iter()
        .map(|d| serde_json::json!({
            "name": d.argument(),
            "description": d.help(),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "target": { "type": "string", "description": "path / address / .sol file" },
                },
                "required": ["target"],
            },
        }))
        .collect();
    serde_json::json!({ "tools": tools })
}

fn tools_call(params: &Value) -> Option<Value> {
    let name = params.get("name").and_then(Value::as_str)?;
    let target = params
        .get("arguments")
        .and_then(|a| a.get("target"))
        .and_then(Value::as_str)?;
    let detector = slither_detectors::find_by_argument(name)?;
    let cfg = slither_core::SlitherConfig::default();
    let loaded = slither_core::project::load_project(std::path::Path::new(target), cfg).ok()?;
    let findings = detector.detect(&loaded.slither);
    Some(serde_json::json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&findings).unwrap_or_default(),
        }],
        "isError": false,
    }))
}

fn resources_list() -> Value {
    let resources: Vec<Value> = slither_printers::all()
        .iter()
        .map(|p| serde_json::json!({
            "uri": format!("slither-printer://{}", p.argument()),
            "name": p.argument(),
            "description": p.help(),
            "mimeType": "text/plain",
        }))
        .collect();
    serde_json::json!({ "resources": resources })
}

fn resources_read(params: &Value) -> Option<Value> {
    let uri = params.get("uri").and_then(Value::as_str)?;
    let name = uri.strip_prefix("slither-printer://")?;
    let target = params
        .get("target")
        .and_then(Value::as_str)
        .unwrap_or(".");
    let printer = slither_printers::find(name)?;
    let cfg = slither_core::SlitherConfig::default();
    let loaded = slither_core::project::load_project(std::path::Path::new(target), cfg).ok()?;
    let mut buf: Vec<u8> = Vec::new();
    printer.output(&loaded.slither, &mut buf).ok()?;
    Some(serde_json::json!({
        "contents": [{
            "uri": uri,
            "mimeType": "text/plain",
            "text": String::from_utf8_lossy(&buf).to_string(),
        }],
    }))
}
