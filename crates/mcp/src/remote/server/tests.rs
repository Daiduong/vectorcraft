use super::*;
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
