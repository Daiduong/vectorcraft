//! Experimental, local-only Streamable HTTP transport.
//!
//! This is deliberately not a public OAuth service. It binds only to loopback and requires a
//! 256-bit bearer token loaded from a private file. The existing `Server` handles MCP messages;
//! this module only validates HTTP and frames its replies. No headless fallback is introduced.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use subtle::ConstantTimeEq;

use crate::{Backend, Server};

const MAX_BODY: usize = 12 * 1024 * 1024;
const MAX_HEADERS: usize = 16 * 1024;
const REQUEST_DEADLINE: Duration = Duration::from_secs(10);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
const VERSIONS: &[&str] = &["2025-03-26", "2025-06-18"];

/// Configuration for the local prototype. A public address is refused, even with a token.
pub struct Options {
    address: SocketAddr,
    token: String,
}

impl Options {
    /// Load the local bearer token and validate a loopback-only listener address.
    /// Reject oversized files and tokens other than 64 hexadecimal characters.
    pub fn from_file(address: &str, path: &Path) -> Result<Self, String> {
        let address: SocketAddr = address.parse().map_err(|e| format!("invalid HTTP address: {e}"))?;
        if !address.ip().is_loopback() {
            return Err("experimental HTTP must bind to a loopback address".into());
        }
        let file = std::fs::File::open(path).map_err(|e| format!("cannot open HTTP token file: {e}"))?;
        let metadata = file.metadata().map_err(|e| format!("cannot inspect HTTP token file: {e}"))?;
        if !metadata.is_file() {
            return Err("HTTP token path must be a regular file".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err("HTTP token file must not be accessible by group or other users".into());
            }
        }
        let mut bytes = Vec::new();
        file.take(257).read_to_end(&mut bytes).map_err(|e| format!("cannot read HTTP token file: {e}"))?;
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

/// Bound HTTP endpoint. One request per connection, with a total input deadline.
pub struct Endpoint {
    transport: TcpListener,
    authority: String,
    token: String,
}

/// Limited HTTP/1.1 request; parsed headers are validated before body consumption.
struct HttpRequest {
    peer: SocketAddr,
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl HttpRequest {
    fn header(&self, name: &str) -> Result<Option<String>, Reply> {
        let mut matches = self.headers.iter().filter(|(key, _)| key.eq_ignore_ascii_case(name));
        let first = matches.next().map(|(_, value)| value.clone());
        if matches.next().is_some() {
            return Err(Reply::error(400, format!("duplicate {name} header")));
        }
        Ok(first)
    }
}

/// A read always honors the *original* deadline; slow trickles cannot extend it.
fn bounded_read(socket: &mut TcpStream, dest: &mut [u8], deadline: Instant) -> Result<usize, Reply> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(Reply::error(408, "HTTP request deadline exceeded"));
    }
    socket.set_read_timeout(Some(remaining)).map_err(|_| Reply::error(400, "cannot read request"))?;
    match socket.read(dest) {
        Ok(n) => Ok(n),
        Err(e) if matches!(e.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock) => {
            Err(Reply::error(408, "HTTP request deadline exceeded"))
        }
        Err(_) => Err(Reply::error(400, "cannot read request")),
    }
}

/// Read one HTTP request line and a bounded header section, without consuming its body.
/// Rejecting authentication or Host/Origin now never tries to drain a stalled upload.
fn read_head(socket: &mut TcpStream, peer: SocketAddr, deadline: Instant) -> Result<HttpRequest, Reply> {
    let mut buffer = Vec::new();
    let boundary = loop {
        if let Some(i) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
        if buffer.len() >= MAX_HEADERS {
            return Err(Reply::error(431, "HTTP headers too large"));
        }
        let mut chunk = [0u8; 4096];
        let capacity = (MAX_HEADERS - buffer.len()).min(chunk.len());
        let count = bounded_read(socket, &mut chunk[..capacity], deadline)?;
        if count == 0 {
            return Err(Reply::error(400, "incomplete HTTP request headers"));
        }
        buffer.extend_from_slice(&chunk[..count]);
    };
    let header_text = std::str::from_utf8(&buffer[..boundary]).map_err(|_| Reply::error(400, "invalid HTTP headers"))?;
    let mut lines = header_text.split("\r\n");
    let start = lines.next().ok_or_else(|| Reply::error(400, "invalid HTTP request line"))?;
    let mut tokens = start.split(' ');
    let (Some(method), Some(path), Some(version), None) = (tokens.next(), tokens.next(), tokens.next(), tokens.next()) else {
        return Err(Reply::error(400, "invalid HTTP request line"));
    };
    if version != "HTTP/1.1" || !method.bytes().all(|b| b.is_ascii_uppercase()) || !path.starts_with('/') {
        return Err(Reply::error(400, "invalid HTTP request line"));
    }
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if headers.len() >= 64 {
            return Err(Reply::error(431, "too many HTTP headers"));
        }
        let Some((key, value)) = line.split_once(':') else {
            return Err(Reply::error(400, "invalid HTTP header"));
        };
        if key.is_empty() || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') || value.bytes().any(|b| b < 32 && b != b'\t' || b == 127) {
            return Err(Reply::error(400, "invalid HTTP header"));
        }
        headers.push((key.to_owned(), value.trim_matches([' ', '\t']).to_owned()));
    }
    Ok(HttpRequest {
        peer,
        method: method.to_owned(),
        path: path.to_owned(),
        headers,
        // Some of the body may already have arrived in the header read.
        body: buffer[boundary..].to_vec(),
    })
}

/// Receive an exactly declared bounded request body. Unsupported transfer framing is
/// rejected rather than interpreted ambiguously (request smuggling); connections close.
fn read_body(socket: &mut TcpStream, request: &mut HttpRequest, deadline: Instant) -> Result<(), Reply> {
    if request.header("Transfer-Encoding")?.is_some() {
        return Err(Reply::error(400, "Transfer-Encoding is not supported by this local endpoint"));
    }
    if request.header("Expect")?.is_some() {
        return Err(Reply::error(417, "Expect headers are not supported by this local endpoint"));
    }
    let declared = match request.header("Content-Length")? {
        Some(n) => n.parse::<usize>().map_err(|_| Reply::error(400, "invalid Content-Length"))?,
        None if request.method == "POST" => return Err(Reply::error(411, "Content-Length required")),
        None => 0,
    };
    if declared > MAX_BODY {
        return Err(Reply::error(413, "request body too large"));
    }
    request.body.truncate(declared);
    while request.body.len() < declared {
        let mut chunk = [0u8; 16384];
        let count = bounded_read(socket, &mut chunk[..(declared - request.body.len()).min(16384)], deadline)?;
        if count == 0 {
            return Err(Reply::error(400, "incomplete HTTP request body"));
        }
        request.body.extend_from_slice(&chunk[..count]);
    }
    Ok(())
}

impl Endpoint {
    /// Bind the validated local HTTP endpoint and retain its actual listening address.
    pub fn bind(options: Options) -> Result<Self, String> {
        let transport = TcpListener::bind(options.address).map_err(|e| format!("HTTP bind failed: {e}"))?;
        let address = transport.local_addr().map_err(|e| format!("HTTP listener address unavailable: {e}"))?;
        Ok(Self { transport, authority: address.to_string(), token: options.token })
    }

    /// Return the bound authority, including the assigned port when a test requested port zero.
    pub fn address(&self) -> &str {
        &self.authority
    }

    /// Dispatch requests serially. A stalled or malformed peer cannot retain this
    /// thread beyond the fixed per-request deadline.
    pub fn serve(self, backend: Box<dyn Backend>) -> Result<(), String> {
        let mut server = Server::new(backend);
        loop {
            let (socket, peer) = self.transport.accept().map_err(|e| format!("HTTP accept failed: {e}"))?;
            self.handle_socket(socket, peer, &mut server);
        }
    }

    fn handle_socket(&self, mut socket: TcpStream, peer: SocketAddr, server: &mut Server) {
        let deadline = Instant::now() + REQUEST_DEADLINE;
        let _ = socket.set_write_timeout(Some(WRITE_TIMEOUT));
        let reply = match read_head(&mut socket, peer, deadline) {
            Ok(mut request) => match self.validate(&request) {
                Err(reply) => reply,
                Ok(()) => match read_body(&mut socket, &mut request, deadline) {
                    Ok(()) => self.dispatch(&request, server),
                    Err(reply) => reply,
                },
            },
            Err(reply) => reply,
        };
        // All connections are close-after-response; rejected requests are *never*
        // synchronously drained. No edit is retried if the response is lost.
        if let Err(e) = socket.write_all(&reply.wire()) {
            log::debug!("HTTP client did not receive its response: {e}; request was not replayed");
        }
        // Close the sending half only: Windows may reset unread rejected bodies.
        let _ = socket.shutdown(std::net::Shutdown::Write);
    }

    /// Check the real loopback peer, Host, Origin, bearer token, route and protocol revision.
    /// Reject duplicate security headers before application dispatch.
    fn validate(&self, request: &HttpRequest) -> Result<(), Reply> {
        if !request.peer.ip().is_loopback() {
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
        if request.path != "/mcp" && request.path != "/health" {
            return Err(Reply::error(404, "not found"));
        }
        if let Some(version) = header(request, "MCP-Protocol-Version")?
            && !VERSIONS.contains(&version.as_str())
        {
            return Err(Reply::error(400, "unsupported MCP-Protocol-Version"));
        }
        Ok(())
    }

    /// Validate bounded HTTP/JSON-RPC input and dispatch one authenticated MCP message.
    /// Notifications receive an empty response without running edits; protocol state is per request.
    fn dispatch(&self, request: &HttpRequest, server: &mut Server) -> Reply {
        if request.path == "/health" {
            return if request.method == "GET" {
                Reply::json(
                    200,
                    json!({"service":"vectorcraft-native-http-prototype",
                    "version":env!("CARGO_PKG_VERSION"), "scope":"loopback-only"}),
                )
            } else {
                Reply::error(405, "use GET for health")
            };
        }
        if request.method != "POST" {
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
        let bytes = &request.body;
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

/// Read at most one header value, rejecting duplicates rather than choosing one.
fn header(request: &HttpRequest, name: &str) -> Result<Option<String>, Reply> {
    request.header(name)
}

struct Reply {
    status: u16,
    body: String,
}

impl Reply {
    /// Construct an HTTP reply with no response body, such as an accepted notification.
    fn empty(status: u16) -> Self {
        Self { status, body: String::new() }
    }
    /// Encode a JSON response using the requested HTTP status.
    fn json(status: u16, body: Value) -> Self {
        Self { status, body: body.to_string() }
    }
    /// Construct a transport error response rather than a successful tool result.
    fn error(status: u16, message: impl Into<String>) -> Self {
        Self::json(status, json!({"error":message.into()}))
    }
    /// Construct a JSON-RPC error with a null request ID for invalid incoming messages.
    fn rpc_error(status: u16, code: i64, message: &str) -> Self {
        Self::json(status, json!({"jsonrpc":"2.0","id":null,"error":{"code":code,"message":message}}))
    }
    /// Send a non-persistent HTTP/1.1 response with explicit framing. No unread
    /// request body is drained, even on authentication or protocol errors.
    fn wire(self) -> Vec<u8> {
        let reason = match self.status {
            200 => "OK",
            202 => "Accepted",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            408 => "Request Timeout",
            411 => "Length Required",
            413 => "Content Too Large",
            415 => "Unsupported Media Type",
            417 => "Expectation Failed",
            431 => "Request Header Fields Too Large",
            _ => "Error",
        };
        let mut head = format!(
            "HTTP/1.1 {} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n",
            self.status,
            self.body.len()
        );
        if self.status == 401 {
            head.push_str("WWW-Authenticate: Bearer realm=\"vectorcraft-local-prototype\"\r\n");
        }
        if self.status == 405 {
            head.push_str("Allow: POST\r\n");
        }
        head.push_str("\r\n");
        let mut bytes = head.into_bytes();
        bytes.extend_from_slice(self.body.as_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests;
