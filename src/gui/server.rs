//! Local HTTP integration server for cross-Lancer-tool communication.
//!
//! Plan ## Cross-tool integration: every Lancer-clone GUI exposes a small local
//! HTTP server on `127.0.0.1:<port>` (auto-allocated), with the port written to
//! `%TEMP%/lancer-frida.port`. Endpoints:
//!   - `GET  /v1/state`                         — current jobs / progress
//!   - `POST /v1/import`                         — accept a payload from another tool
//!   - `GET  /v1/export?format=json|csv|sarif|markdown` — export the result set
//!
//! Hand-rolled minimal HTTP/1.1 (one request per connection) to stay
//! dependency-free.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

/// State shared between the GUI and the integration server.
#[derive(Default)]
pub struct Integration {
    /// Human-readable status line (mirrors the GUI status bar).
    pub status: String,
    /// Currently selected target, if any.
    pub target: Option<String>,
    /// Result rows the GUI is currently showing (exported on demand).
    pub results: Vec<String>,
    /// Payloads received from other Lancer tools via POST /v1/import.
    pub inbox: Vec<String>,
    /// Bound port (0 until the server starts).
    pub port: u16,
}

/// Start the integration server on a background thread. Returns the bound port.
pub fn start(state: Arc<Mutex<Integration>>) -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();

    if let Ok(mut s) = state.lock() {
        s.port = port;
    }

    // Advertise the port to the Lancer mothership / sibling tools.
    let port_file = std::env::temp_dir().join("lancer-frida.port");
    let _ = std::fs::write(&port_file, port.to_string());

    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let state = Arc::clone(&state);
                // Each connection is short-lived; handle inline.
                let _ = handle(stream, state);
            }
        }
    });

    Ok(port)
}

fn handle(mut stream: TcpStream, state: Arc<Mutex<Integration>>) -> std::io::Result<()> {
    let peer_ok = stream
        .peer_addr()
        .map(|a| a.ip().is_loopback())
        .unwrap_or(false);
    if !peer_ok {
        return Ok(()); // localhost only
    }

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();

    // Drain headers, capturing Content-Length.
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }

    let (status, body) = route(&method, &path, &mut reader, content_length, &state);

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn route(
    method: &str,
    path: &str,
    reader: &mut BufReader<TcpStream>,
    content_length: usize,
    state: &Arc<Mutex<Integration>>,
) -> (&'static str, String) {
    let (route, query) = match path.split_once('?') {
        Some((r, q)) => (r, q),
        None => (path, ""),
    };

    match (method, route) {
        ("GET", "/v1/state") => {
            let s = state.lock().unwrap();
            let json = serde_json::json!({
                "tool": "lancer-frida",
                "status": s.status,
                "target": s.target,
                "items": s.results.len(),
                "inbox": s.inbox.len(),
                "port": s.port,
            });
            ("200 OK", json.to_string())
        }
        ("POST", "/v1/import") => {
            let mut buf = vec![0u8; content_length];
            if reader.read_exact(&mut buf).is_ok() {
                let payload = String::from_utf8_lossy(&buf).into_owned();
                let mut s = state.lock().unwrap();
                s.inbox.push(payload);
                ("200 OK", serde_json::json!({ "accepted": true }).to_string())
            } else {
                ("400 Bad Request", serde_json::json!({ "error": "bad body" }).to_string())
            }
        }
        ("GET", "/v1/export") => {
            let format = query
                .split('&')
                .find_map(|kv| kv.strip_prefix("format="))
                .unwrap_or("json");
            let s = state.lock().unwrap();
            ("200 OK", export(&s.results, format))
        }
        _ => ("404 Not Found", serde_json::json!({ "error": "not found" }).to_string()),
    }
}

fn export(results: &[String], format: &str) -> String {
    match format {
        "csv" => {
            let mut out = String::from("index,value\n");
            for (i, r) in results.iter().enumerate() {
                out.push_str(&format!("{i},{}\n", csv_escape(r)));
            }
            out
        }
        "markdown" => {
            let mut out = String::from("| # | value |\n|---|-------|\n");
            for (i, r) in results.iter().enumerate() {
                out.push_str(&format!("| {i} | {} |\n", r.replace('|', "\\|")));
            }
            out
        }
        "sarif" => serde_json::json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": { "driver": { "name": "lancer-frida", "rules": [] } },
                "results": results.iter().map(|r| serde_json::json!({
                    "message": { "text": r }
                })).collect::<Vec<_>>(),
            }]
        })
        .to_string(),
        _ => serde_json::json!({ "results": results }).to_string(),
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
