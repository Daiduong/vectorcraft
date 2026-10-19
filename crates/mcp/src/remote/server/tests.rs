use super::*;

fn context() -> (tempfile::TempDir, Arc<Context>) {
    let root = tempfile::tempdir().unwrap();
    let profile = root.path().join("profile");
    let artwork = root.path().join("artwork");
    super::super::config::protect_directory(&profile.join("Remote Access")).unwrap();
    std::fs::create_dir(&artwork).unwrap();
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let settings = Settings {
        version: 1,
        enabled: true,
        public_origin: "https://native.example.test".into(),
        base_path: "/vectorcraft-native".into(),
        bind: "127.0.0.1:3002".into(),
        trusted_proxy: None,
        control_port: 33982,
        app_path: std::env::current_exe().unwrap(),
        profile_dir: profile.clone(),
        artwork_root: artwork,
        additional_artwork_roots: vec![],
        owner_hash: HASH.get_or_init(|| super::super::config::hash_phrase("A private server test passphrase").unwrap()).clone(),
        management_token: super::super::config::secret(),
        allow_test_callbacks: false,
    };
    let store = Store::open(settings.clone(), &profile.join("Remote Access/oauth-state.json")).unwrap();
    let worker = Worker::start(settings.clone()).unwrap();
    (
        root,
        Arc::new(Context {
            settings,
            store: Mutex::new(store),
            worker,
            stop: tokio::sync::Notify::new(),
            admission: Arc::new(tokio::sync::Semaphore::new(32)),
            owner_admission: Arc::new(tokio::sync::Semaphore::new(4)),
            password_checks: Arc::new(tokio::sync::Semaphore::new(2)),
        }),
    )
}

#[test]
fn only_loopback_and_the_explicit_gateway_are_accepted_regardless_of_forwarding_headers() {
    let (_root, mut context) = context();
    Arc::get_mut(&mut context).unwrap().settings.trusted_proxy = Some("192.0.2.10".parse().unwrap());
    let request = Request::builder()
        .uri("/vectorcraft-native/health")
        .header("host", "native.example.test")
        .header("x-forwarded-for", "127.0.0.1")
        .header("forwarded", "for=192.0.2.10")
        .body(axum::body::Body::empty())
        .unwrap();
    for ip in ["127.0.0.1:1234", "192.0.2.10:1234"] {
        context.validate(&request, ip.parse().unwrap()).unwrap();
    }
    assert_eq!(context.validate(&request, "198.51.100.10:1234".parse().unwrap()).unwrap_err().status, 403);
}

#[tokio::test]
async fn anonymous_capacity_cannot_block_authenticated_local_recovery() {
    let (_root, context) = context();
    let _occupied = context.admission.clone().acquire_many_owned(32).await.unwrap();
    let request = Request::builder()
        .method("POST")
        .uri("/vectorcraft-native/_owner")
        .header("host", "localhost:3002")
        .header("authorization", format!("Bearer {}", context.settings.management_token))
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"action":"status"}"#))
        .unwrap();
    let response = endpoint(State(context.clone()), ConnectInfo("127.0.0.1:1234".parse().unwrap()), request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let anonymous = Request::builder().uri("/vectorcraft-native/health").body(axum::body::Body::empty()).unwrap();
    assert_eq!(endpoint(State(context), ConnectInfo("127.0.0.1:1234".parse().unwrap()), anonymous).await.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn anonymous_invalid_approval_posts_cannot_lock_out_a_real_browser_sign_in() {
    let (_root, context) = context();
    let (id, cookie, _) = {
        let mut store = context.store().unwrap();
        let client =
            store.register(json!({"redirect_uris":["https://chatgpt.com/connector/oauth/test"],"token_endpoint_auth_method":"none"})).unwrap();
        let fields = oauth::fields(
            &url::form_urlencoded::Serializer::new(String::new())
                .append_pair("response_type", "code")
                .append_pair("client_id", client["client_id"].as_str().unwrap())
                .append_pair("redirect_uri", "https://chatgpt.com/connector/oauth/test")
                .append_pair("code_challenge", &super::super::config::digest("server-test-verifier"))
                .append_pair("code_challenge_method", "S256")
                .finish(),
        )
        .unwrap();
        store.authorize(&fields).unwrap()
    };
    let request = |body: String, cookie: &str| {
        Request::builder()
            .method("POST")
            .uri("/vectorcraft-native/approve")
            .header("host", "native.example.test")
            .header("origin", "https://native.example.test")
            .header("cookie", format!("vectorcraft_native_oauth={cookie}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(axum::body::Body::from(body))
            .unwrap()
    };
    let peer = "127.0.0.1:1234".parse().unwrap();
    for _ in 0..25 {
        let response = dispatch(&context, peer, request("request=invented&decision=allow&passphrase=wrong".into(), "invented")).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("request", &id)
        .append_pair("decision", "allow")
        .append_pair("passphrase", "A private server test passphrase")
        .finish();
    assert_eq!(dispatch(&context, peer, request(body, &cookie)).await.status(), StatusCode::SEE_OTHER);
}
#[test]
fn modern_and_unknown_http_versions_are_rejected_for_legacy_fallback() {
    for version in HTTP_VERSIONS {
        assert!(supported_http_version(version).is_ok());
    }
    for version in ["2026-07-28", "1900-01-01"] {
        let response = supported_http_version(version).unwrap_err();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response.headers().get("connection").unwrap(), "close");
    }
}
#[test]
fn callback_policy_keeps_google_relays_separate_from_user_content() {
    assert!(oauth::allowed_callback("https://chatgpt.com/connector/oauth/test", false));
    assert!(oauth::allowed_callback("https://oauth-redirect.googleusercontent.com/r/user_bound_custom-mcp-test", false));
    for callback in [
        "https://evil.googleusercontent.com/r/test",
        "https://chatgpt.com.evil.example/callback",
        "https://oauth-redirect.googleusercontent.com/r/test/extra",
        "https://chatgpt.com/callback#fragment",
        "http://chatgpt.com/callback",
    ] {
        assert!(!oauth::allowed_callback(callback, false), "{callback}");
    }
}
#[test]
fn duplicate_form_fields_are_rejected() {
    assert!(oauth::fields("client_id=one&client_id=two").is_err());
    assert!(oauth::fields("client_id=one&code=two").is_ok());
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn complete_fixed_and_chunked_bodies_work_but_an_unfinished_body_never_dispatches() {
    use std::io::{Read as _, Write as _};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let router = Router::new().fallback(|request: Request| async {
        match receive_body(request, 256, Duration::from_millis(200)).await {
            Ok(bytes) => json_response(200, json!({"received":bytes.len()})),
            Err(response) => *response,
        }
    });
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    for (framing, body, expected) in [
        ("Content-Length: 2\r\n", "{}", "200 OK"),
        ("Transfer-Encoding: chunked\r\n", "2\r\n{}\r\n0\r\n\r\n", "200 OK"),
        ("Content-Length: 4\r\n", "{}", "408 Request Timeout"),
        ("Transfer-Encoding: chunked\r\n", "2\r\n{}\r\n", "408 Request Timeout"),
        ("Content-Length: 500\r\n", "", "413 Payload Too Large"),
    ] {
        let response = tokio::task::spawn_blocking(move || {
            let mut socket = std::net::TcpStream::connect(address).unwrap();
            socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            write!(socket, "POST /mcp HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n{framing}\r\n{body}").unwrap();
            let mut response = String::new();
            // Timeout replies may be followed by a reset because the peer never finished its body.
            let _ = socket.read_to_string(&mut response);
            response
        })
        .await
        .unwrap();
        assert!(response.contains(expected), "{response}");
    }
    server.abort();
}
