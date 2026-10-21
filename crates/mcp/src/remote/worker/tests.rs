use super::*;

/// Create isolated artwork folders and settings for worker file-access and reply-loss checks.
fn fixture() -> (tempfile::TempDir, Settings) {
    let root = tempfile::tempdir().unwrap();
    let profile = root.path().join("profile");
    let artwork = root.path().join("artwork");
    std::fs::create_dir_all(&profile).unwrap();
    std::fs::create_dir_all(profile.join("Remote Access")).unwrap();
    std::fs::create_dir_all(&artwork).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let settings = Settings {
        version: 1,
        enabled: true,
        public_origin: "https://native.example.test".into(),
        base_path: "/vectorcraft-native".into(),
        bind: "127.0.0.1:3002".into(),
        trusted_proxy: None,
        control_port: port,
        app_path: std::env::current_exe().unwrap(),
        profile_dir: profile,
        artwork_root: artwork,
        additional_artwork_roots: Vec::new(),
        owner_hash: String::new(), // Not used by the MCP worker, which has no consent endpoint.
        management_token: String::new(),
        allow_test_callbacks: false,
    };
    (root, settings)
}

/// Verify out-of-folder file targets are refused before attempting a desktop connection.
#[test]
fn explicit_file_targets_are_checked_before_desktop_dispatch() {
    let (_root, settings) = fixture();
    let roots = vectorcraft_engine::file_access::AutomationRoots::new(Some(&settings.artwork_root), Some(&settings.artwork_root)).unwrap().unwrap();
    let mut backend = Guarded {
        remote: Remote::deferred(&format!("127.0.0.1:{}", settings.control_port)),
        capability: settings.desktop_capability().unwrap(),
        uncertain: Arc::new(AtomicBool::new(false)),
        roots,
    };
    for method in ["app.open", "app.save", "app.export", "ui.render", "ui.screenshot"] {
        let error = backend.call(method, json!({"path":settings.profile_dir.join("private.json")})).unwrap_err();
        assert!(error.contains("outside the"), "{method}: {error}");
    }
    assert!(!backend.uncertain.load(Ordering::SeqCst));
}
/// Reconnection and readiness probes always use the bound envelope, never a bare desktop method.
#[test]
fn reconnection_and_open_refuse_a_different_workspace_on_the_same_port() {
    use std::io::{BufRead as _, Write as _};
    let (_root, settings) = fixture();
    let listener = std::net::TcpListener::bind(("127.0.0.1", settings.control_port)).unwrap();
    let capability = settings.desktop_capability().unwrap();
    let expected = capability.clone();
    let responder = std::thread::spawn(move || {
        for index in 0..3 {
            let (mut socket, _) = listener.accept().unwrap();
            socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            let mut line = String::new();
            std::io::BufReader::new(socket.try_clone().unwrap()).read_line(&mut line).unwrap();
            let request: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["method"], "remote.execute");
            assert_eq!(request["params"]["capability"], expected);
            let reply = if index == 0 {
                json!({"id":request["id"],"ok":true,"result":{"verified":true}})
            } else {
                json!({"id":request["id"],"ok":false,"error":"Workspace mismatch; no operation dispatched"})
            };
            writeln!(socket, "{reply}").unwrap();
            socket.shutdown(std::net::Shutdown::Both).unwrap();
        }
    });
    let mut backend = Guarded {
        remote: Remote::deferred(&format!("127.0.0.1:{}", settings.control_port)),
        capability,
        uncertain: Arc::new(AtomicBool::new(false)),
        roots: settings.automation_roots().unwrap().unwrap(),
    };
    assert_eq!(backend.call("ui.inspect", json!({})).unwrap()["verified"], true);
    // The first server closed its socket. The replacement must authorize the next operation anew.
    let probe = backend.remote.connection_probe();
    probe.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    assert_eq!(probe.peek(&mut [0]).unwrap(), 0, "wait for the old endpoint's FIN before reconnecting");
    assert!(
        backend
            .call("engine.execute", json!({"command":"file.list","params":{"path":settings.profile_dir}}))
            .unwrap_err()
            .contains("Workspace mismatch")
    );
    assert!(open(&settings).unwrap_err().contains("Workspace mismatch"));
    assert!(!backend.uncertain.load(Ordering::SeqCst));
    responder.join().unwrap();
}

/// Verify a lost mutation reply pauses later edits while allowing discovery and inspection metadata.
#[test]
fn dropped_mutation_reply_pauses_edits_but_keeps_metadata_available() {
    let (_root, settings) = fixture();
    let worker = Worker::start(settings).unwrap();
    let (reply, response) = tokio::sync::oneshot::channel();
    drop(response);
    worker.pending.fetch_add(1, Ordering::SeqCst);
    worker
        .sender
        .send(Job {
            message: json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"undo","arguments":{}}}),
            version: crate::PROTOCOL_VERSION.into(),
            deadline: Instant::now() + Duration::from_secs(10),
            reply,
        })
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !worker.uncertain.load(Ordering::SeqCst) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(worker.uncertain.load(Ordering::SeqCst));
    for (message, allowed) in [
        (json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"undo","arguments":{}}}), false),
        (json!({"jsonrpc":"2.0","id":3,"method":"tools/list"}), true),
    ] {
        let (reply, response) = tokio::sync::oneshot::channel();
        worker.pending.fetch_add(1, Ordering::SeqCst);
        worker
            .sender
            .send(Job { message, version: crate::PROTOCOL_VERSION.into(), deadline: Instant::now() + Duration::from_secs(5), reply })
            .unwrap();
        let result = response.blocking_recv().unwrap();
        assert_eq!(result.is_ok(), allowed);
        if !allowed {
            assert!(result.unwrap_err().contains("unknown outcome"));
        }
    }
}
