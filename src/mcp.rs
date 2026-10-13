//! Minimal MCP (JSON-RPC 2.0 over stdio, newline-delimited) server.
use crate::{parse_query, Client};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn tools() -> Value {
    let id = json!({"type": "integer"});
    let res = json!({"type": "string", "enum": crate::RESOURCES});
    json!([
        {"name": "wger_list", "description": "List a wger resource (routine, day, slot, slot-entry, *-config, workoutlog, workoutsession, exercise-translation, exerciseinfo, language...). query is a map of filters, e.g. {\"routine\":\"80246\"}.",
         "inputSchema": {"type": "object", "properties": {"resource": res, "query": {"type": "object", "additionalProperties": {"type": "string"}}, "all": {"type": "boolean"}}, "required": ["resource"]}},
        {"name": "wger_get", "description": "Get one object of a wger resource by id.",
         "inputSchema": {"type": "object", "properties": {"resource": res, "id": id}, "required": ["resource", "id"]}},
        {"name": "wger_create", "description": "Create an object (POST). data is the JSON body per the wger API schema.",
         "inputSchema": {"type": "object", "properties": {"resource": res, "data": {"type": "object"}}, "required": ["resource", "data"]}},
        {"name": "wger_update", "description": "Partially update an object (PATCH).",
         "inputSchema": {"type": "object", "properties": {"resource": res, "id": id, "data": {"type": "object"}}, "required": ["resource", "id", "data"]}},
        {"name": "wger_delete", "description": "Delete an object. Requires confirm=true.",
         "inputSchema": {"type": "object", "properties": {"resource": res, "id": id, "confirm": {"type": "boolean"}}, "required": ["resource", "id", "confirm"]}},
        {"name": "wger_routine_view", "description": "Routine read views: structure (days/slots/entries/configs), logs, stats, date-sequence-display, date-sequence-gym.",
         "inputSchema": {"type": "object", "properties": {"id": id, "view": {"type": "string", "enum": ["structure", "logs", "stats", "date-sequence-display", "date-sequence-gym"]}}, "required": ["id", "view"]}},
        {"name": "wger_translation_set", "description": "Create or update an exercise translation in a language (code like 'pt' or id). Description is HTML.",
         "inputSchema": {"type": "object", "properties": {"exercise": id, "lang": {"type": "string"}, "name": {"type": "string"}, "description": {"type": "string"}, "license_author": {"type": "string"}}, "required": ["exercise", "lang"]}}
    ])
}

fn s<'a>(a: &'a Value, k: &str) -> Result<&'a str> {
    a[k].as_str().ok_or_else(|| anyhow!("missing string '{k}'"))
}
fn n(a: &Value, k: &str) -> Result<u64> {
    a[k].as_u64()
        .ok_or_else(|| anyhow!("missing integer '{k}'"))
}

pub fn call(c: &Client, name: &str, a: &Value) -> Result<Value> {
    match name {
        "wger_list" => {
            let q: Vec<String> = a["query"]
                .as_object()
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| {
                            format!(
                                "{k}={}",
                                v.as_str()
                                    .map(String::from)
                                    .unwrap_or_else(|| v.to_string())
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            c.list(
                s(a, "resource")?,
                &parse_query(&q)?,
                a["all"].as_bool().unwrap_or(false),
            )
        }
        "wger_get" => c.get(s(a, "resource")?, n(a, "id")?),
        "wger_create" => c.create(s(a, "resource")?, &a["data"]),
        "wger_update" => c.update(s(a, "resource")?, n(a, "id")?, &a["data"]),
        "wger_delete" => c.delete(
            s(a, "resource")?,
            n(a, "id")?,
            a["confirm"].as_bool().unwrap_or(false),
        ),
        "wger_routine_view" => c.routine_view(n(a, "id")?, s(a, "view")?),
        "wger_translation_set" => c.translation_set(
            n(a, "exercise")?,
            s(a, "lang")?,
            a["name"].as_str(),
            a["description"].as_str(),
            a["license_author"].as_str(),
        ),
        _ => Err(anyhow!("unknown tool '{name}'")),
    }
}

pub fn handle(c: &Client, msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned()?; // notifications get no reply
    let method = msg["method"].as_str().unwrap_or("");
    let result = match method {
        "initialize" => json!({
            "protocolVersion": msg["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "wger", "version": env!("CARGO_PKG_VERSION")}
        }),
        "ping" => json!({}),
        "tools/list" => json!({"tools": tools()}),
        "tools/call" => {
            let p = &msg["params"];
            match call(c, p["name"].as_str().unwrap_or(""), &p["arguments"]) {
                Ok(v) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&v).unwrap()}]})
                }
                Err(e) => {
                    json!({"content": [{"type": "text", "text": e.to_string()}], "isError": true})
                }
            }
        }
        _ => {
            return Some(
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": format!("method not found: {method}")}}),
            )
        }
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

pub fn serve(c: &Client) -> Result<()> {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(m) => handle(c, &m),
            Err(e) => Some(
                json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": e.to_string()}}),
            ),
        };
        if let Some(r) = reply {
            writeln!(out, "{}", r)?;
            out.flush()?;
        }
    }
    Ok(())
}
