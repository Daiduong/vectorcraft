use super::{
    SCOPE,
    config::{Settings, now},
    oauth::{self, Failure, Fields, Store, equal},
    pages,
    worker::Worker,
};
use axum::{
    Json, Router,
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
};
use base64::Engine as _;
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct Context {
    settings: Settings,
    store: Mutex<Store>,
    worker: Arc<Worker>,
    stop: tokio::sync::Notify,
    admission: Arc<tokio::sync::Semaphore>,
    rates: Mutex<HashMap<&'static str, (u64, u32)>>,
}
// This transport implements the initialization-based revisions. Upstream's partial modern
// result support is not a complete 2026 transport (server/discover, metadata validation, MRTR).
const HTTP_VERSIONS: &[&str] = &["2025-03-26", "2025-06-18"];
fn supported_http_version(version: &str) -> Result<(), Box<Response>> {
    if HTTP_VERSIONS.contains(&version) {
        return Ok(());
    }
    let mut response = bad("Unsupported MCP protocol version; this HTTP transport supports 2025-06-18 and 2025-03-26");
    // Reject unsupported probes before reading their body. Do not reuse a connection whose
    // unread bytes might be mistaken for another request. A legacy 400 enables client fallback.
    put(&mut response, "connection", "close");
    Err(Box::new(response))
}
fn json_response(status: u16, value: Value) -> Response {
    (StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(value)).into_response()
}
fn error(error: Failure) -> Response {
    json_response(error.status, json!({"error":error.code,"error_description":error.message}))
}
fn bad(message: &str) -> Response {
    error(Failure::new("invalid_request", message))
}
fn header(headers: &HeaderMap, name: &str) -> Result<Option<String>, Failure> {
    let values: Vec<_> = headers.get_all(name).iter().collect();
    if values.len() > 1 {
        return Err(Failure::new("invalid_request", format!("Duplicate {name} header")));
    }
    values.first().map(|v| v.to_str().map(str::to_owned).map_err(|_| Failure::new("invalid_request", "Invalid header"))).transpose()
}
fn put(response: &mut Response, name: &'static str, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        response.headers_mut().insert(name, value);
    }
}
impl Context {
    fn rate(&self, key: &'static str, limit: u32, window: u64) -> bool {
        let Ok(mut rates) = self.rates.lock() else { return false };
        let slot = rates.entry(key).or_insert((now(), 0));
        if now().saturating_sub(slot.0) >= window {
            *slot = (now(), 0);
        }
        if slot.1 >= limit {
            return false;
        }
        slot.1 += 1;
        true
    }
    fn validate(&self, request: &Request, peer: SocketAddr) -> Result<(), Failure> {
        let host = header(request.headers(), "host")?.unwrap_or_default();
        let url = self.settings.origin().map_err(|e| Failure::new("server_error", e))?;
        let authority = &url[url::Position::BeforeHost..url::Position::AfterPort];
        let address: SocketAddr = self.settings.bind.parse().map_err(|_| Failure::new("server_error", "Invalid bind"))?;
        let local = [format!("127.0.0.1:{}", address.port()), format!("localhost:{}", address.port()), format!("[::1]:{}", address.port())];
        if host != authority && !(peer.ip().is_loopback() && local.contains(&host)) {
            return Err(Failure { status: 403, code: "access_denied", message: "Host not allowed".into() });
        }
        if let Some(origin) = header(request.headers(), "origin")?
            && origin != url.origin().ascii_serialization()
        {
            return Err(Failure { status: 403, code: "access_denied", message: "Origin not allowed".into() });
        }
        header(request.headers(), "authorization")?;
        header(request.headers(), "cookie")?;
        Ok(())
    }
    fn store(&self) -> Result<std::sync::MutexGuard<'_, Store>, Failure> {
        self.store.lock().map_err(|_| Failure::new("server_error", "Authorization state is unavailable"))
    }
}
async fn body(request: Request, limit: usize) -> Result<Vec<u8>, Box<Response>> {
    receive_body(request, limit, Duration::from_secs(10)).await
}
async fn receive_body(request: Request, limit: usize, timeout: Duration) -> Result<Vec<u8>, Box<Response>> {
    let declared = request.headers().get("content-length").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<usize>().ok());
    if declared.is_some_and(|n| n > limit) {
        return Err(Box::new(json_response(413, json!({"error":"request_too_large"}))));
    }
    let chunked = request.headers().get("transfer-encoding").is_some_and(|value| value == "chunked");
    let expects_continue = request.headers().get("expect").is_some_and(|value| value == "100-continue");
    let encoding = match request.headers().get("content-encoding").and_then(|value| value.to_str().ok()) {
        None | Some("identity") => "identity",
        Some("gzip") => "gzip",
        Some("br") => "br",
        Some("deflate") => "deflate",
        _ => "other",
    };
    let mut incoming = request.into_body();
    let mut bytes = Vec::new();
    let read = async {
        while let Some(frame) = incoming.frame().await {
            let frame = frame.map_err(|_| 400_u16)?;
            if let Ok(data) = frame.into_data() {
                if data.len() > limit.saturating_sub(bytes.len()) {
                    return Err(413);
                }
                bytes.extend_from_slice(&data);
            }
        }
        Ok::<(), u16>(())
    };
    match tokio::time::timeout(timeout, read).await {
        Ok(Ok(())) => Ok(bytes),
        Ok(Err(status)) => {
            Err(Box::new(json_response(status, json!({"error":if status == 413 {"request_too_large"} else {"invalid_request_body"}}))))
        }
        Err(_) => {
            // Framing diagnostics only: never log Authorization, cookies, URLs, or body content.
            let parsed = serde_json::from_slice::<Value>(&bytes);
            let complete_json = parsed.is_ok();
            let json_eof = parsed.as_ref().err().is_some_and(serde_json::Error::is_eof);
            let start = if bytes.starts_with(&[0x1f, 0x8b]) {
                "gzip"
            } else if bytes.iter().find(|byte| !byte.is_ascii_whitespace()) == Some(&b'{') {
                "json-object"
            } else {
                "other"
            };
            eprintln!(
                "native-mcp request_body_timeout: received_bytes={} declared_bytes={declared:?} complete_json={complete_json} json_eof={json_eof} body_start={start} encoding={encoding} chunked={chunked} expect_continue={expects_continue}",
                bytes.len()
            );
            Err(Box::new(json_response(
                408,
                json!({"error":"request_body_timeout","error_description":"The HTTP request body did not finish before the deadline; no tool was dispatched. Check the declared Content-Length and the bytes sent by the client and gateway."}),
            )))
        }
    }
}
async fn form(request: Request) -> Result<Fields, Box<Response>> {
    if request.headers().get("content-type").and_then(|v| v.to_str().ok()).and_then(|v| v.split(';').next())
        != Some("application/x-www-form-urlencoded")
    {
        return Err(Box::new(json_response(415, json!({"error":"invalid_request","error_description":"Use form encoding"}))));
    }
    let bytes = body(request, 16 * 1024).await?;
    let text = std::str::from_utf8(&bytes).map_err(|_| Box::new(bad("Form is not UTF-8")))?;
    oauth::fields(text).map_err(|e| Box::new(error(e)))
}
async fn json_body(request: Request) -> Result<Value, Box<Response>> {
    if request.headers().get("content-type").and_then(|v| v.to_str().ok()).and_then(|v| v.split(';').next()) != Some("application/json") {
        return Err(Box::new(json_response(415, json!({"error":"invalid_request","error_description":"Use JSON"}))));
    }
    serde_json::from_slice(&body(request, 16 * 1024).await?).map_err(|_| Box::new(bad("Invalid JSON")))
}
async fn endpoint(State(context): State<Arc<Context>>, ConnectInfo(peer): ConnectInfo<SocketAddr>, request: Request) -> Response {
    let permit = context.admission.clone().try_acquire_owned();
    let mut response = if permit.is_err() {
        json_response(503, json!({"error":"server_busy"}))
    } else if let Err(e) = context.validate(&request, peer) {
        error(e)
    } else {
        dispatch(&context, peer, request).await
    };
    put(&mut response, "cache-control", "no-store");
    put(&mut response, "x-content-type-options", "nosniff");
    put(&mut response, "x-robots-tag", "noindex, nofollow");
    put(&mut response, "referrer-policy", "strict-origin");
    response
}
async fn dispatch(context: &Arc<Context>, peer: SocketAddr, request: Request) -> Response {
    let settings = &context.settings;
    let path = request.uri().path().trim_end_matches('/').to_string();
    let method = request.method().clone();
    let prefix = &settings.base_path;
    let issuer = settings.issuer();
    let auth_meta = format!("/.well-known/oauth-authorization-server{prefix}");
    let resource_meta = format!("/.well-known/oauth-protected-resource{prefix}/mcp");
    if method == Method::GET && (path == auth_meta || path == format!("{prefix}/.well-known/oauth-authorization-server")) {
        return json_response(
            200,
            json!({"issuer":issuer,"authorization_endpoint":format!("{issuer}/authorize"),"token_endpoint":format!("{issuer}/token"),
            "registration_endpoint":format!("{issuer}/register"),"revocation_endpoint":format!("{issuer}/revoke"),"response_types_supported":["code"],
            "grant_types_supported":["authorization_code","refresh_token"],"code_challenge_methods_supported":["S256"],
            "token_endpoint_auth_methods_supported":["none","client_secret_post","client_secret_basic"],"revocation_endpoint_auth_methods_supported":["none","client_secret_post","client_secret_basic"],
            "scopes_supported":[SCOPE],"authorization_response_iss_parameter_supported":true}),
        );
    }
    if method == Method::GET && (path == resource_meta || path == format!("{prefix}/.well-known/oauth-protected-resource")) {
        return json_response(
            200,
            json!({"resource":settings.resource(),"authorization_servers":[issuer],"scopes_supported":[SCOPE],"resource_name":"VectorCraft native desktop"}),
        );
    }
    if path == format!("{prefix}/health") && method == Method::GET {
        return json_response(
            200,
            json!({"status":"ok","service":"vectorcraft-native","version":env!("CARGO_PKG_VERSION"),"transport":"streamable-http","authentication":"oauth","tools":crate::tool_definitions().len()+2}),
        );
    }
    if path == format!("{prefix}/_owner") {
        let authorization = header(request.headers(), "authorization").ok().flatten().unwrap_or_default();
        if method != Method::POST
            || !peer.ip().is_loopback()
            || !equal(authorization.strip_prefix("Bearer ").unwrap_or(""), &settings.management_token)
        {
            return json_response(403, json!({"error":"access_denied"}));
        }
        let input = match json_body(request).await {
            Ok(value) => value,
            Err(response) => return *response,
        };
        let mut store = match context.store() {
            Ok(store) => store,
            Err(e) => return error(e),
        };
        let action = input.get("action").and_then(Value::as_str).unwrap_or("");
        match action {
            "status" => json_response(
                200,
                json!({"running":true,"activeArtworkRoots":settings.artwork_roots(),"endpoint":settings.resource(),"clients":store.clients(),"pending":context.worker.pending.load(Ordering::SeqCst),"uncertain":context.worker.uncertain.load(Ordering::SeqCst)}),
            ),
            "register" => match store.register(input.get("client").cloned().unwrap_or(Value::Null)) {
                Ok(value) => json_response(201, value),
                Err(e) => error(e),
            },
            "revoke_client" => match store.remove_client(input.get("id").and_then(Value::as_str).unwrap_or("")) {
                Ok(()) => json_response(200, json!({"revoked":true})),
                Err(e) => error(e),
            },
            "revoke_all" => match store.revoke_all() {
                Ok(()) => json_response(200, json!({"revoked":true})),
                Err(e) => error(e),
            },
            "resume" => {
                if context.worker.pending.load(Ordering::SeqCst) > 0 {
                    return json_response(409, json!({"error":"Work is still running"}));
                }
                context.worker.uncertain.store(false, Ordering::SeqCst);
                json_response(200, json!({"resumed":true}))
            }
            "stop" => {
                if context.worker.pending.load(Ordering::SeqCst) > 0 {
                    return json_response(409, json!({"error":"Work is still running; finish it before stopping the connection"}));
                }
                context.stop.notify_one();
                json_response(202, json!({"stopping":true}))
            }
            _ => bad("Unknown owner action"),
        }
    } else if path == format!("{prefix}/register") && method == Method::POST {
        if !context.rate("register", 24, 60) {
            return json_response(429, json!({"error":"slow_down"}));
        }
        let input = match json_body(request).await {
            Ok(value) => value,
            Err(response) => return *response,
        };
        match context.store().and_then(|mut store| store.register(input)) {
            Ok(value) => json_response(201, value),
            Err(e) => error(e),
        }
    } else if path == format!("{prefix}/authorize") && method == Method::GET {
        if !context.rate("authorize", 60, 60) {
            return json_response(429, json!({"error":"slow_down"}));
        }
        let parameters = match oauth::fields(request.uri().query().unwrap_or("")) {
            Ok(value) => value,
            Err(e) => return error(e),
        };
        match context.store().and_then(|mut store| store.authorize(&parameters)) {
            Ok((id, cookie, pending)) => {
                let mut response =
                    (StatusCode::OK, [("content-type", "text/html; charset=utf-8")], pages::consent(settings, &id, &pending, None)).into_response();
                let secure = if settings.public_origin.starts_with("https:") { "; Secure" } else { "" };
                put(
                    &mut response,
                    "set-cookie",
                    &format!("vectorcraft_native_oauth={cookie}; HttpOnly; SameSite=Lax; Max-Age=600; Path={prefix}/approve{secure}"),
                );
                put(&mut response, "content-security-policy", &pages::csp(&pending.redirect));
                put(&mut response, "x-frame-options", "DENY");
                response
            }
            Err(e) => error(e),
        }
    } else if path == format!("{prefix}/approve") && method == Method::POST {
        if !context.rate("approve", 10, 900) {
            return json_response(429, json!({"error":"slow_down","error_description":"Too many sign-in attempts. Try later"}));
        }
        let origin = header(request.headers(), "origin").ok().flatten().unwrap_or_default();
        if settings.origin().map(|u| u.origin().ascii_serialization()).unwrap_or_default() != origin {
            return json_response(403, json!({"error":"access_denied","error_description":"Origin not allowed"}));
        }
        let cookies = header(request.headers(), "cookie").ok().flatten().unwrap_or_default();
        let cookie = cookies.split(';').map(str::trim).find_map(|c| c.strip_prefix("vectorcraft_native_oauth=")).unwrap_or("").to_string();
        let parameters = match form(request).await {
            Ok(value) => value,
            Err(response) => return *response,
        };
        let id = parameters.get("request").cloned().unwrap_or_default();
        let pending = match context.store().and_then(|store| store.pending(&id, &cookie)) {
            Ok(value) => value,
            Err(e) => return error(e),
        };
        let allow = parameters.get("decision").map(String::as_str) == Some("allow");
        if !allow && parameters.get("decision").map(String::as_str) != Some("deny") {
            return bad("Invalid decision");
        }
        if allow {
            let phrase = parameters.get("passphrase").cloned().unwrap_or_default();
            let hash = settings.owner_hash.clone();
            let valid = tokio::task::spawn_blocking(move || super::config::check_phrase(&phrase, &hash)).await.unwrap_or(false);
            if !valid {
                let mut response = (
                    StatusCode::FORBIDDEN,
                    [("content-type", "text/html; charset=utf-8")],
                    pages::consent(settings, &id, &pending, Some("Incorrect passphrase. You can try again.")),
                )
                    .into_response();
                put(&mut response, "content-security-policy", &pages::csp(&pending.redirect));
                put(&mut response, "x-frame-options", "DENY");
                return response;
            }
        }
        match context.store().and_then(|mut store| store.approve(&id, &cookie, allow)) {
            Ok(destination) => {
                let mut response = StatusCode::SEE_OTHER.into_response();
                put(&mut response, "location", &destination);
                put(&mut response, "set-cookie", &format!("vectorcraft_native_oauth=; HttpOnly; SameSite=Lax; Max-Age=0; Path={prefix}/approve"));
                put(&mut response, "content-security-policy", &pages::csp(&pending.redirect));
                response
            }
            Err(e) => error(e),
        }
    } else if (path == format!("{prefix}/token") || path == format!("{prefix}/revoke")) && method == Method::POST {
        if !context.rate("tokens", 120, 60) {
            return json_response(429, json!({"error":"slow_down"}));
        }
        let authorization = header(request.headers(), "authorization").ok().flatten();
        let basic = authorization
            .as_deref()
            .map(|a| {
                let value = a.strip_prefix("Basic ").ok_or_else(Failure::unauthorized)?;
                let bytes = base64::engine::general_purpose::STANDARD.decode(value).map_err(|_| Failure::unauthorized())?;
                let text = String::from_utf8(bytes).map_err(|_| Failure::unauthorized())?;
                let (id, secret) = text.split_once(':').ok_or_else(Failure::unauthorized)?;
                Ok::<_, Failure>((id.to_string(), secret.to_string()))
            })
            .transpose();
        let basic = match basic {
            Ok(value) => value,
            Err(e) => return error(e),
        };
        let parameters = match form(request).await {
            Ok(value) => value,
            Err(response) => return *response,
        };
        let mut store = match context.store() {
            Ok(store) => store,
            Err(e) => return error(e),
        };
        let client = match store.authenticate(&parameters, basic.as_ref().map(|(id, secret)| (id.as_str(), secret.as_str()))) {
            Ok(value) => value,
            Err(e) => return error(e),
        };
        if path.ends_with("/token") {
            match store.token(&parameters, &client) {
                Ok(value) => json_response(200, value),
                Err(e) => error(e),
            }
        } else {
            match store.revoke(parameters.get("token").map(String::as_str).unwrap_or(""), &client) {
                Ok(()) => StatusCode::OK.into_response(),
                Err(e) => error(e),
            }
        }
    } else if path == format!("{prefix}/mcp") {
        let authorization = header(request.headers(), "authorization").ok().flatten().unwrap_or_default();
        let token = authorization.strip_prefix("Bearer ").unwrap_or("");
        let valid = context.store().is_ok_and(|store| store.valid_access(token));
        if !valid {
            let mut response =
                json_response(401, json!({"error":"invalid_token","error_description":"Authorize this connection before using the tools"}));
            put(
                &mut response,
                "www-authenticate",
                &format!(
                    "Bearer error=\"invalid_token\", scope=\"{SCOPE}\", resource_metadata=\"{}{}\"",
                    settings.public_origin.trim_end_matches('/'),
                    resource_meta
                ),
            );
            return response;
        }
        if method != Method::POST {
            let mut response = StatusCode::METHOD_NOT_ALLOWED.into_response();
            put(&mut response, "allow", "POST");
            return response;
        }
        let version = match header(request.headers(), "mcp-protocol-version") {
            Ok(Some(value)) => value,
            Ok(None) => "2025-03-26".into(),
            Err(e) => return error(e),
        };
        if let Err(response) = supported_http_version(&version) {
            return *response;
        }
        let accept = header(request.headers(), "accept").ok().flatten().unwrap_or_default();
        let types: Vec<_> = accept.split(',').filter_map(|v| v.split(';').next().map(str::trim)).collect();
        if !types.contains(&"application/json") || !types.contains(&"text/event-stream") {
            return json_response(406, json!({"error":"Accept must include application/json and text/event-stream"}));
        }
        if header(request.headers(), "content-type").ok().flatten().and_then(|s| s.split(';').next().map(str::to_owned)).as_deref()
            != Some("application/json")
        {
            return json_response(415, json!({"error":"Use application/json"}));
        }
        let bytes = match body(request, 12 * 1024 * 1024).await {
            Ok(value) => value,
            Err(response) => return *response,
        };
        let mut message: Value = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(_) => return json_response(400, json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Invalid JSON"}})),
        };
        if !message.is_object()
            || message.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
            || message.get("params").is_some_and(|p| !p.is_object())
            || message.get("id").is_some_and(|v| !v.is_string() && !v.is_number())
        {
            return bad("Invalid JSON-RPC envelope");
        }
        let rpc_method = message.get("method").and_then(Value::as_str).unwrap_or("").to_string();
        if rpc_method.is_empty() {
            return if message.get("id").is_some() && (message.get("result").is_some() ^ message.get("error").is_some()) {
                StatusCode::ACCEPTED.into_response()
            } else {
                bad("Missing method")
            };
        }
        if message.get("id").is_none() {
            return StatusCode::ACCEPTED.into_response();
        }
        let id = message.get("id").cloned().unwrap_or(Value::Null);
        if rpc_method == "logging/setLevel" {
            return json_response(
                200,
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Logging streams are not offered over this transport"}}),
            );
        }
        if rpc_method == "initialize" {
            let asked = message.pointer("/params/protocolVersion").and_then(Value::as_str).unwrap_or(crate::PROTOCOL_VERSION);
            if !HTTP_VERSIONS.contains(&asked) {
                message["params"]["protocolVersion"] = json!(crate::PROTOCOL_VERSION);
            }
        }
        match context.worker.call(message, version).await {
            Ok(value) => json_response(200, value),
            Err(message) if rpc_method == "tools/call" => {
                json_response(200, json!({"jsonrpc":"2.0","id":id,"result":crate::ToolResult::error(message).to_value()}))
            }
            Err(message) => json_response(200, json!({"jsonrpc":"2.0","id":id,"error":{"code":-32003,"message":message}})),
        }
    } else {
        json_response(404, json!({"error":"not_found"}))
    }
}

pub fn run(path: &Path) -> Result<(), String> {
    let settings = Settings::read(path)?;
    if !settings.enabled {
        return Err("Remote access is disabled".into());
    }
    let directory = Settings::directory(path)?;
    super::config::protect_directory(directory)?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(directory.join("service.lock"))
        .map_err(|e| e.to_string())?;
    fs2::FileExt::try_lock_exclusive(&lock).map_err(|_| "This profile's remote service is already running")?;
    let store = Store::open(settings.clone(), &directory.join("oauth-state.json"))?;
    let worker = Worker::start(settings.clone())?;
    let context = Arc::new(Context {
        settings: settings.clone(),
        store: Mutex::new(store),
        worker,
        stop: tokio::sync::Notify::new(),
        admission: Arc::new(tokio::sync::Semaphore::new(32)),
        rates: Mutex::new(HashMap::new()),
    });
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().map_err(|e| e.to_string())?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(&settings.bind).await.map_err(|e| format!("Cannot listen on {}: {e}", settings.bind))?;
        let stop = context.clone();
        let router = Router::new().fallback(endpoint).with_state(context);
        axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async move { stop.stop.notified().await })
            .await
            .map_err(|e| e.to_string())
    })
}

#[cfg(test)]
mod tests;
