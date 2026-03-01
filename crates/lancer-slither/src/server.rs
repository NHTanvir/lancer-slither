//! HTTP server for Lancer cross-tool integration + MCP server.

use std::io::Read as _;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use tiny_http::{Header, Method, Response, Server};

#[derive(Default, Clone)]
struct ServerState {
    jobs: Arc<Mutex<Vec<JobRecord>>>,
}

#[derive(Clone)]
struct JobRecord {
    id: String,
    target: PathBuf,
    findings: Vec<slither_core::Finding>,
}

pub fn serve(addr: &str) -> eyre::Result<i32> {
    let server = Server::http(addr).map_err(|e| eyre::eyre!("bind {addr}: {e}"))?;
    let port = server
        .server_addr()
        .to_ip()
        .map(|sa| sa.port())
        .unwrap_or(0);
    write_port_file(port)?;
    println!("lancer-slither serving on {addr} (port {port})");
    let state = ServerState::default();
    for mut req in server.incoming_requests() {
        let url = req.url().to_owned();
        let method = req.method().clone();
        let resp = handle(&method, &url, &mut req, &state)?;
        let _ = req.respond(resp);
    }
    Ok(0)
}

fn write_port_file(port: u16) -> eyre::Result<()> {
    let mut p = std::env::temp_dir();
    p.push("lancer-slither.port");
    std::fs::write(p, port.to_string())?;
    Ok(())
}

fn handle(
    method: &Method,
    url: &str,
    req: &mut tiny_http::Request,
    state: &ServerState,
) -> eyre::Result<Response<std::io::Cursor<Vec<u8>>>> {
    let header_json = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
    let header_text = Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap();
    match (method, url) {
        (Method::Get, "/v1/state") => {
            let jobs = state.jobs.lock();
            let body = serde_json::to_vec_pretty(&serde_json::json!({
                "jobs": jobs.iter().map(|j| serde_json::json!({
                    "id": j.id,
                    "target": j.target,
                    "findings": j.findings.len(),
                })).collect::<Vec<_>>()
            }))?;
            Ok(Response::from_data(body).with_header(header_json))
        }
        (Method::Post, "/v1/scan") => {
            let mut body = String::new();
            req.as_reader().read_to_string(&mut body)?;
            let v: serde_json::Value = serde_json::from_str(&body)?;
            let target = v.get("target").and_then(serde_json::Value::as_str).unwrap_or("").to_owned();
            let cfg = slither_core::SlitherConfig::default();
            let loaded = slither_core::project::load_project(std::path::Path::new(&target), cfg)?;
            let findings = slither_detectors::run_all(&loaded.slither);
            let id = format!("job-{}", state.jobs.lock().len() + 1);
            let record = JobRecord {
                id: id.clone(),
                target: PathBuf::from(&target),
                findings: findings.clone(),
            };
            state.jobs.lock().push(record);
            let results = slither_core::OutputResults {
                success: true,
                error: None,
                results: slither_core::finding::ResultsBucket {
                    detectors: findings, printers: Vec::new(),
                },
            };
            let body = serde_json::to_vec_pretty(&results)?;
            Ok(Response::from_data(body).with_header(header_json))
        }
        (Method::Get, "/v1/detectors") => {
            let body = serde_json::to_vec_pretty(
                &slither_detectors::all()
                    .iter()
                    .map(|d| serde_json::json!({
                        "argument": d.argument(),
                        "impact": d.impact().to_string(),
                        "confidence": d.confidence().to_string(),
                        "help": d.help(),
                    }))
                    .collect::<Vec<_>>(),
            )?;
            Ok(Response::from_data(body).with_header(header_json))
        }
        (Method::Get, "/v1/printers") => {
            let body = serde_json::to_vec_pretty(
                &slither_printers::all()
                    .iter()
                    .map(|p| serde_json::json!({"argument": p.argument(), "help": p.help()}))
                    .collect::<Vec<_>>(),
            )?;
            Ok(Response::from_data(body).with_header(header_json))
        }
        (Method::Post, "/v1/import") => {
            let mut body = String::new();
            req.as_reader().read_to_string(&mut body)?;
            println!("/v1/import: {} bytes", body.len());
            Ok(Response::from_string("ok\n").with_header(header_text))
        }
        (Method::Post, "/mcp") => {
            let mut body = String::new();
            req.as_reader().read_to_string(&mut body)?;
            let parsed: Result<crate::mcp::JsonRpcRequest, _> = serde_json::from_str(&body);
            let resp = match parsed {
                Ok(r) => crate::mcp::handle(&r),
                Err(e) => crate::mcp::JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: serde_json::Value::Null,
                    result: None,
                    error: Some(crate::mcp::JsonRpcError {
                        code: -32700,
                        message: format!("parse error: {e}"),
                    }),
                },
            };
            let payload = serde_json::to_vec_pretty(&resp)?;
            Ok(Response::from_data(payload).with_header(header_json))
        }
        (Method::Get, _) if url.starts_with("/v1/export") => {
            let format = url
                .split('?')
                .nth(1)
                .unwrap_or("")
                .split('&')
                .find_map(|p| p.strip_prefix("format="))
                .unwrap_or("json");
            let jobs = state.jobs.lock();
            let findings = jobs.iter().flat_map(|j| j.findings.clone()).collect::<Vec<_>>();
            let body: String = match format {
                "markdown" => slither_core::output::markdown::render(&findings, None),
                "csv" => {
                    let mut s = String::from("check,impact,confidence,file,line\n");
                    for f in findings {
                        for e in &f.elements {
                            s.push_str(&format!(
                                "{},{},{},{},{}\n",
                                f.check,
                                f.impact,
                                f.confidence,
                                e.source_mapping.filename_relative.display(),
                                e.source_mapping.lines.first().copied().unwrap_or(0),
                            ));
                        }
                    }
                    s
                }
                _ => serde_json::to_string_pretty(&findings)?,
            };
            Ok(Response::from_string(body).with_header(header_text))
        }
        _ => Ok(Response::from_string("not found\n")
            .with_header(header_text)
            .with_status_code(404)),
    }
}

