//! Experimental, local-only Streamable HTTP transport.
//!
//! This is deliberately not a public OAuth service. It binds only to loopback and requires a
//! 256-bit bearer token loaded from a private file. The existing `Server` handles MCP messages;
//! this module only validates HTTP and frames its replies. No headless fallback is introduced.

use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;

use serde_json::{Value, json};
use subtle::ConstantTimeEq;
use tiny_http::{Header, Method, Request, Response, StatusCode};

use crate::{Backend, Server};

const MAX_BODY: usize = 12 * 1024 * 1024;
const VERSIONS: &[&str] = &["2025-03-26", "2025-06-18"];

/// Configuration for the local prototype. A public address is refused, even with a token.
pub struct Options {
    address: SocketAddr,
    token: String,
}

impl Options {
    pub fn from_file(address: &str, path: &Path) -> Result<Self, String> {
        let address: SocketAddr = address.parse().map_err(|e| format!("invalid HTTP address: {e}"))?;
        if !address.ip().is_loopback() {
            return Err("experimental HTTP must bind to a loopback address".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| format!("cannot open HTTP token file: {e}"))?
            .take(257)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("cannot read HTTP token file: {e}"))?;
        if bytes.len() > 256 {
            return Err("HTTP token file is too large".into());
        }
        let text = std::str::from_utf8(&bytes).map_err(|_| "HTTP token must be UTF-8")?;
        let token = text.trim();
        if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("HTTP token must contain exactly 64 random hexadecimal characters".into());
        }
        Ok(Self { address, token: token.into() })
    }
}

/// Bound HTTP endpoint. Requests run serially on the owning MCP/backend thread.
pub struct Endpoint {
    transport: tiny_http::Server,
    authority: String,
    token: String,
}

impl Endpoint {
    pub fn bind(options: Options) -> Result<Self, String> {
        let transport = tiny_http::Server::http(options.address).map_err(|e| format!("HTTP bind failed: {e}"))?;
        let address = transport.server_addr().to_ip().ok_or("HTTP listener has no IP address")?;
        Ok(Self { transport, authority: address.to_string(), token: options.token })
    }

    pub fn address(&self) -> &str {
        &self.authority
    }

    pub fn serve(self, backend: Box<dyn Backend>) -> Result<(), String> {
        let mut server = Server::new(backend);
        loop {
            let request = self.transport.recv().map_err(|e| format!("HTTP receive failed: {e}"))?;
            self.handle(request, &mut server);
        }
    }

    fn handle(&self, mut request: Request, server: &mut Server) {
        let reply = match self.validate(&request) {
            Ok(()) => self.dispatch(&mut request, server),
            Err(reply) => reply,
        };
        // Losing the HTTP reader does not cancel, reconnect or replay a dispatched edit.
        if let Err(e) = request.respond(reply.response()) {
            log::debug!("HTTP client did not receive its response: {e}; request was not replayed");
        }
    }

    fn validate(&self, request: &Request) -> Result<(), Reply> {
        if !request.remote_addr().is_some_and(|a| a.ip().is_loopback()) {
            return Err(Reply::error(403, "loopback clients only"));
        }
        let host = header(request, "Host")?;
        if host.as_deref() != Some(self.authority.as_str()) {
            return Err(Reply::error(403, "Host not allowed"));
        }
        if let Some(origin) = header(request, "Origin")?
            && origin != format!("http://{}", self.authority)
        {
            return Err(Reply::error(403, "Origin not allowed"));
        }
        let authorization = header(request, "Authorization")?.unwrap_or_default();
        let supplied = authorization.strip_prefix("Bearer ").unwrap_or("");
        if !bool::from(supplied.as_bytes().ct_eq(self.token.as_bytes())) {
            return Err(Reply::error(401, "bearer token required"));
        }
        if request.url() != "/mcp" && request.url() != "/health" {
            return Err(Reply::error(404, "not found"));
        }
        if let Some(version) = header(request, "MCP-Protocol-Version")?
            && !VERSIONS.contains(&version.as_str())
        {
            return Err(Reply::error(400, "unsupported MCP-Protocol-Version"));
        }
        Ok(())
    }

    fn dispatch(&self, request: &mut Request, server: &mut Server) -> Reply {
        if request.url() == "/health" {
            return if request.method() == &Method::Get {
                Reply::json(
                    200,
                    json!({"service":"vectorcraft-native-http-prototype",
                    "version":env!("CARGO_PKG_VERSION"), "scope":"loopback-only"}),
                )
            } else {
                Reply::error(405, "use GET for health")
            };
        }
        if request.method() != &Method::Post {
            return Reply::error(405, "this stateless endpoint accepts POST; standalone SSE is not offered");
        }
        let content_type = match header(request, "Content-Type") {
            Ok(Some(value)) => value,
            _ => return Reply::error(415, "Content-Type must be application/json"),
        };
        if content_type.split(';').next().map(str::trim) != Some("application/json") {
            return Reply::error(415, "Content-Type must be application/json");
        }
        let accept = match header(request, "Accept") {
            Ok(Some(value)) => value,
            _ => return Reply::error(406, "Accept must include application/json and text/event-stream"),
        };
        let accepted: Vec<&str> = accept.split(',').filter_map(|v| v.split(';').next().map(str::trim)).collect();
        if !accepted.contains(&"application/json") || !accepted.contains(&"text/event-stream") {
            return Reply::error(406, "Accept must include application/json and text/event-stream");
        }
        if request.body_length().is_some_and(|size| size > MAX_BODY) {
            return Reply::error(413, "request body too large");
        }
        let mut bytes = Vec::new();
        if let Err(e) = request.as_reader().take((MAX_BODY + 1) as u64).read_to_end(&mut bytes) {
            return Reply::error(400, format!("cannot read request: {e}"));
        }
        if bytes.len() > MAX_BODY {
            return Reply::error(413, "request body too large");
        }
        let mut message: Value = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(_) => return Reply::rpc_error(400, -32700, "invalid JSON"),
        };
        if !message.is_object() || message.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return Reply::rpc_error(400, -32600, "expected a single JSON-RPC 2.0 message");
        }
        let Some(method) = message.get("method").and_then(Value::as_str).map(str::to_owned) else {
            // The transport permits client responses even though this prototype asks no questions.
            if message.get("id").is_some_and(|id| id.is_string() || id.is_number())
                && (message.get("result").is_some() ^ message.get("error").is_some())
            {
                return Reply::empty(202);
            }
            return Reply::rpc_error(400, -32600, "missing JSON-RPC method");
        };
        if message.get("params").is_some_and(|params| !params.is_object()) || message.get("id").is_some_and(|id| !id.is_string() && !id.is_number()) {
            return Reply::rpc_error(400, -32600, "invalid JSON-RPC id or params");
        }
        if message.get("id").is_none() {
            // No edit is a notification; this prototype offers no cancellation/progress stream.
            return Reply::empty(202);
        }
        let version = match header(request, "MCP-Protocol-Version") {
            Ok(Some(version)) => version,
            _ => "2025-03-26".into(),
        };
        if method == "initialize" {
            let asked = message.pointer("/params/protocolVersion").and_then(Value::as_str).unwrap_or(crate::PROTOCOL_VERSION);
            if !VERSIONS.contains(&asked) {
                message["params"]["protocolVersion"] = json!(crate::PROTOCOL_VERSION);
            }
        } else {
            // HTTP is stateless: one client's negotiated revision cannot leak into another's.
            let _ = server.handle(json!({"jsonrpc":"2.0","id":"http-version",
                "method":"initialize","params":{"protocolVersion":version}}));
        }
        if method == "logging/setLevel" {
            return Reply::json(
                200,
                json!({"jsonrpc":"2.0","id":message.get("id"),
                "error":{"code":-32601,"message":"HTTP prototype does not offer logging streams"}}),
            );
        }
        match server.handle(message) {
            Some(mut response) => {
                if method == "initialize"
                    && let Some(capabilities) = response.pointer_mut("/result/capabilities").and_then(Value::as_object_mut)
                {
                    capabilities.remove("logging");
                }
                Reply::json(200, response)
            }
            None => Reply::empty(202),
        }
    }
}

fn header(request: &Request, name: &str) -> Result<Option<String>, Reply> {
    let mut matches = request.headers().iter().filter(|h| h.field.to_string().eq_ignore_ascii_case(name));
    let value = matches.next().map(|h| h.value.to_string());
    if matches.next().is_some() {
        return Err(Reply::error(400, format!("duplicate {name} header")));
    }
    Ok(value)
}

struct Reply {
    status: u16,
    body: String,
}

impl Reply {
    fn empty(status: u16) -> Self {
        Self { status, body: String::new() }
    }
    fn json(status: u16, body: Value) -> Self {
        Self { status, body: body.to_string() }
    }
    fn error(status: u16, message: impl Into<String>) -> Self {
        Self::json(status, json!({"error":message.into()}))
    }
    fn rpc_error(status: u16, code: i64, message: &str) -> Self {
        Self::json(status, json!({"jsonrpc":"2.0","id":null,"error":{"code":code,"message":message}}))
    }
    fn response(self) -> Response<std::io::Cursor<Vec<u8>>> {
        let mut response = Response::from_string(self.body).with_status_code(StatusCode(self.status));
        let mut headers = vec![("Content-Type", "application/json"), ("Cache-Control", "no-store"), ("X-Content-Type-Options", "nosniff")];
        if self.status == 401 {
            headers.push(("WWW-Authenticate", "Bearer realm=\"vectorcraft-local-prototype\""));
        }
        if self.status == 405 {
            headers.push(("Allow", "POST"));
        }
        for (name, value) in headers {
            if let Ok(header) = Header::from_bytes(name, value) {
                response.add_header(header);
            }
        }
        response
    }
}

#[cfg(test)]
mod tests;
