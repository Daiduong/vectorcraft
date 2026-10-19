use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::*;

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct Fake(Arc<AtomicUsize>);
impl Backend for Fake {
    fn call(&mut self, _method: &str, _params: Value) -> Result<Value, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(json!({"ok":true}))
    }
    fn has_ui(&self) -> bool {
        true
    }
    fn describe(&self) -> String {
        "test desktop".into()
    }
}

fn fixture(count: usize) -> (String, Arc<AtomicUsize>, std::thread::JoinHandle<()>) {
    let endpoint = Endpoint::bind(Options { address: "127.0.0.1:0".parse().unwrap(), token: TOKEN.into() }).unwrap();
    let address = endpoint.address().to_string();
    let calls = Arc::new(AtomicUsize::new(0));
    let backend = calls.clone();
    let worker = std::thread::spawn(move || {
        let mut mcp = Server::new(Box::new(Fake(backend)));
        for _ in 0..count {
            let request = endpoint.transport.recv_timeout(Duration::from_secs(10)).unwrap().expect("test request");
            endpoint.handle(request, &mut mcp);
        }
    });
    (address, calls, worker)
}

fn raw(address: &str, method: &str, path: &str, headers: &str, body: &str) -> (u16, String) {
    let mut socket = TcpStream::connect(address).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(socket, "{method} {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\nContent-Length: {}\r\n{headers}\r\n{body}", body.len())
        .unwrap();
    socket.shutdown(Shutdown::Write).unwrap();
    let mut response = String::new();
    socket.read_to_string(&mut response).unwrap();
    let status = response.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, response)
}

fn headers(extra: &str) -> String {
    format!("Authorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\n{extra}")
}

fn rpc(address: &str, method: &str, params: Value, extra: &str) -> (u16, Value) {
    let body = json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}).to_string();
    let (status, response) = raw(address, "POST", "/mcp", &headers(extra), &body);
    let (head, body) = response.split_once("\r\n\r\n").unwrap();
    let bytes = if head.to_ascii_lowercase().contains("transfer-encoding: chunked") {
        // The native catalogue exceeds the HTTP library's chunk threshold. Test the real wire
        // framing instead of assuming every valid JSON response uses Content-Length.
        let mut remaining = body.as_bytes();
        let mut decoded = Vec::new();
        loop {
            let end = remaining.windows(2).position(|w| w == b"\r\n").unwrap();
            let length = usize::from_str_radix(std::str::from_utf8(&remaining[..end]).unwrap(), 16).unwrap();
            remaining = &remaining[end + 2..];
            if length == 0 {
                break;
            }
            assert!(remaining.len() >= length + 2);
            decoded.extend_from_slice(&remaining[..length]);
            remaining = &remaining[length + 2..];
        }
        decoded
    } else {
        body.as_bytes().to_vec()
    };
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[test]
fn authentication_is_required_for_every_request() {
    let (address, calls, worker) = fixture(4);
    assert_eq!(raw(&address, "GET", "/health", "", "").0, 401);
    assert_eq!(raw(&address, "POST", "/mcp", "Authorization: Bearer wrong\r\n", "{}").0, 401);
    assert_eq!(rpc(&address, "tools/list", json!({}), "").0, 200);
    assert_eq!(raw(&address, "POST", "/mcp", "", "{}").0, 401);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    worker.join().unwrap();
}

#[test]
fn host_origin_and_duplicate_auth_are_rejected() {
    let (address, calls, worker) = fixture(4);
    assert_eq!(rpc(&address, "tools/list", json!({}), "Origin: https://untrusted.example\r\n").0, 403);
    assert_eq!(rpc(&address, "tools/list", json!({}), "Host: untrusted.example\r\n").0, 400);
    assert_eq!(rpc(&address, "tools/list", json!({}), "Authorization: Bearer wrong\r\n").0, 400);
    assert_eq!(rpc(&address, "tools/list", json!({}), &format!("Origin: http://{address}\r\n")).0, 200);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    worker.join().unwrap();
}

#[test]
fn discovery_reuses_native_definitions_without_contacting_the_desktop() {
    let (address, calls, worker) = fixture(5);
    let (status, initialized) = rpc(&address, "initialize", json!({"protocolVersion":"2025-06-18"}), "");
    assert_eq!(status, 200);
    assert_eq!(initialized["result"]["serverInfo"]["name"], "vectorcraft");
    assert!(initialized["result"]["capabilities"].get("logging").is_none());
    let (_, tools) = rpc(&address, "tools/list", json!({}), "");
    assert_eq!(tools["result"]["tools"], json!(crate::tool_definitions()));
    for method in ["resources/list", "resources/templates/list", "prompts/list"] {
        let (status, response) = rpc(&address, method, json!({}), "");
        assert_eq!(status, 200);
        assert!(response.get("error").is_none(), "{response}");
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    worker.join().unwrap();
}

#[test]
fn invalid_http_and_json_do_not_dispatch() {
    let (address, calls, worker) = fixture(8);
    assert_eq!(raw(&address, "GET", "/mcp", &headers(""), "").0, 405);
    assert_eq!(raw(&address, "DELETE", "/mcp", &headers(""), "").0, 405);
    assert_eq!(raw(&address, "POST", "/wrong", &headers(""), "{}").0, 404);
    assert_eq!(raw(&address, "POST", "/mcp", &headers(""), "{").0, 400);
    assert_eq!(raw(&address, "POST", "/mcp", &headers(""), "[]").0, 400);
    let no_accept = format!("Authorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\n");
    assert_eq!(raw(&address, "POST", "/mcp", &no_accept, "{}").0, 406);
    let wrong_type = format!("Authorization: Bearer {TOKEN}\r\nContent-Type: text/plain\r\n");
    assert_eq!(raw(&address, "POST", "/mcp", &wrong_type, "{}").0, 415);
    assert_eq!(rpc(&address, "tools/list", json!({}), "MCP-Protocol-Version: 2099-01-01\r\n").0, 400);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    worker.join().unwrap();
}

#[test]
fn notifications_have_empty_accepted_responses_and_never_edit() {
    let (address, calls, worker) = fixture(2);
    for method in ["notifications/initialized", "tools/call"] {
        let body = json!({"jsonrpc":"2.0","method":method,"params":{}}).to_string();
        let (status, response) = raw(&address, "POST", "/mcp", &headers(""), &body);
        assert_eq!(status, 202);
        assert!(response.split_once("\r\n\r\n").unwrap().1.is_empty());
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    worker.join().unwrap();
}

#[test]
fn per_request_protocol_revisions_do_not_leak_between_clients() {
    let (address, _, worker) = fixture(3);
    let (_, init) = rpc(&address, "initialize", json!({"protocolVersion":"2026-07-28"}), "");
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    let (_, old) = rpc(&address, "tools/list", json!({}), "MCP-Protocol-Version: 2025-06-18\r\n");
    let (status, _) = rpc(&address, "tools/list", json!({}), "MCP-Protocol-Version: 2026-07-28\r\n");
    assert_eq!(old["result"]["tools"], json!(crate::tool_definitions()));
    assert_eq!(status, 400, "Do not claim a modern transport without discovery/metadata support");
    worker.join().unwrap();
}

#[test]
fn public_bind_and_invalid_tokens_are_refused() {
    let missing = Path::new("does-not-exist-http-token");
    let err = Options::from_file("0.0.0.0:7979", missing).err().unwrap();
    assert!(err.contains("loopback"));
    assert!(Options::from_file("127.0.0.1:0", missing).is_err());
    let path = std::env::temp_dir().join(format!("vectorcraft-http-token-test-{}", std::process::id()));
    for text in ["too-short", &"x".repeat(64), &"0".repeat(300)] {
        std::fs::write(&path, text).unwrap();
        assert!(Options::from_file("127.0.0.1:0", &path).is_err());
    }
    std::fs::write(&path, format!("{TOKEN}\n")).unwrap();
    assert!(Options::from_file("127.0.0.1:0", &path).is_ok());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn deferred_remote_discovery_works_while_desktop_is_closed() {
    let endpoint = Endpoint::bind(Options { address: "127.0.0.1:0".parse().unwrap(), token: TOKEN.into() }).unwrap();
    let address = endpoint.address().to_string();
    let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let unused = socket.local_addr().unwrap().to_string();
    drop(socket);
    let worker = std::thread::spawn(move || {
        let mut server = Server::new(Box::new(crate::Remote::deferred(&unused)));
        for _ in 0..2 {
            let request = endpoint.transport.recv_timeout(Duration::from_secs(10)).unwrap().unwrap();
            endpoint.handle(request, &mut server);
        }
    });
    assert_eq!(rpc(&address, "tools/list", json!({}), "").0, 200);
    let (_, response) = rpc(&address, "tools/call", json!({"name":"ui_inspect","arguments":{}}), "");
    assert_eq!(response["result"]["isError"], true, "{response}");
    worker.join().unwrap();
}

#[test]
fn native_tool_calls_are_dispatched_once_and_http_disconnect_does_not_replay() {
    let (address, calls, worker) = fixture(2);
    let body = json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
        "params":{"name":"command_run","arguments":{"id":"file.new","params":{}}}})
    .to_string();
    let mut socket = TcpStream::connect(&address).unwrap();
    write!(socket, "POST /mcp HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\nContent-Length: {}\r\n{}\r\n{body}", body.len(), headers(""))
        .unwrap();
    socket.flush().unwrap();
    for _ in 0..100 {
        if calls.load(Ordering::SeqCst) == 1 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    drop(socket);
    let (_, response) = rpc(&address, "tools/list", json!({}), "");
    assert!(response["result"]["tools"].is_array());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    worker.join().unwrap();
}

#[test]
fn oversized_requests_are_rejected_before_reading_or_dispatch() {
    let (address, calls, worker) = fixture(1);
    let mut socket = TcpStream::connect(&address).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(socket, "POST /mcp HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\nContent-Length: {}\r\n{}\r\n", MAX_BODY + 1, headers("")).unwrap();
    socket.shutdown(Shutdown::Write).unwrap();
    let mut response = String::new();
    socket.read_to_string(&mut response).unwrap();
    assert_eq!(response.split_whitespace().nth(1), Some("413"));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    worker.join().unwrap();
}
