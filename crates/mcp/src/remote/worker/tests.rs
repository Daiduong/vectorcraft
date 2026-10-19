use super::*;

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

#[test]
fn explicit_file_targets_are_checked_before_desktop_dispatch() {
    let (_root, settings) = fixture();
    let roots = vectorcraft_engine::file_access::AutomationRoots::new(Some(&settings.artwork_root), Some(&settings.artwork_root)).unwrap().unwrap();
    let mut backend =
        Guarded { remote: Remote::deferred(&format!("127.0.0.1:{}", settings.control_port)), uncertain: Arc::new(AtomicBool::new(false)), roots };
    for method in ["app.open", "app.save", "app.export", "ui.render", "ui.screenshot"] {
        let error = backend.call(method, json!({"path":settings.profile_dir.join("private.json")})).unwrap_err();
        assert!(error.contains("outside the"), "{method}: {error}");
    }
    assert!(!backend.uncertain.load(Ordering::SeqCst));
}

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
