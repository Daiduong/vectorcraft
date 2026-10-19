//! Native host side of Remote AI connections. File/process operations stay out of the UI crate.
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::Duration;
use vectorcraft_mcp::remote::config::{Settings, hash_phrase, protect_directory, secret};

pub fn profile() -> Result<PathBuf, String> {
    super::prefs_path().and_then(|p| p.parent().map(std::path::Path::to_path_buf)).ok_or_else(|| "This session has no saved profile folder".into())
}
fn settings_path() -> Result<PathBuf, String> {
    Ok(profile()?.join("Remote Access/settings.json"))
}
pub type InitialControl = (Option<u16>, Option<std::sync::Arc<vectorcraft_engine::file_access::AutomationRoots>>);
pub fn initial_control(explicit_roots: bool) -> Result<InitialControl, String> {
    if explicit_roots {
        return Err("Native Remote AI folders cannot be combined with command-line automation roots; manage grants in Preferences > Remote AI".into());
    }
    let profile = profile()?;
    initial_control_in(&profile)
}
fn initial_control_in(profile: &std::path::Path) -> Result<InitialControl, String> {
    let private = profile.join("Remote Access");
    let path = private.join("settings.json");
    if path.exists() {
        protect_directory(&private)?;
        let settings = Settings::read(&path)?;
        Ok((settings.enabled.then_some(settings.control_port), settings.automation_roots()?))
    } else {
        // A feature-enabled build is still an ordinary editor until the owner saves setup.
        Ok((None, None))
    }
}
fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())
}
fn owner(settings: &Settings, value: Value) -> Result<Value, String> {
    let address: std::net::SocketAddr = settings.bind.parse().map_err(|_| "Invalid HTTP bind")?;
    let response = client()?
        .post(format!("http://127.0.0.1:{}{}/_owner", address.port(), settings.base_path))
        .bearer_auth(&settings.management_token)
        .json(&value)
        .send()
        .map_err(|e| format!("Remote service is not available: {e}"))?;
    let status = response.status();
    let result: Value = response.json().map_err(|_| "Unexpected response from the remote service")?;
    if !status.is_success() {
        return Err(result
            .get("error_description")
            .or_else(|| result.get("error"))
            .and_then(Value::as_str)
            .unwrap_or("Remote service request failed")
            .into());
    }
    Ok(result)
}
fn listening(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(&std::net::SocketAddr::from(([127, 0, 0, 1], port)), Duration::from_millis(200)).is_ok()
}
fn public_status(settings: Option<&Settings>) -> Result<Value, String> {
    let profile = match settings {
        Some(settings) => settings.profile_dir.clone(),
        None => profile()?,
    };
    let mut result = json!({"configured":settings.is_some(),"running":false,"publicOrigin":"","basePath":"/vectorcraft-native","httpPort":3002,
        "artworkRoots":[],"activeArtworkRoots":[],"privateFolder":profile.join("Remote Access"),"clients":[],"pending":0,"uncertain":false,"trustedProxy":"","controlPort":33982});
    if let Some(settings) = settings {
        result["publicOrigin"] = json!(settings.public_origin);
        result["basePath"] = json!(settings.base_path);
        result["trustedProxy"] = json!(settings.trusted_proxy.map(|ip| ip.to_string()).unwrap_or_default());
        result["controlPort"] = json!(settings.control_port);
        result["artworkRoots"] = json!(settings.artwork_roots());
        result["endpoint"] = json!(settings.resource());
        if let Err(error) = settings.validate() {
            result["configurationError"] = json!(error);
        }
        // A readable legacy service may still be running. Keep authenticated local Stop
        // available during repair, even when the new policy refuses to start that setup.
        let Ok(address) = settings.bind.parse::<std::net::SocketAddr>() else { return Ok(result) };
        result["httpPort"] = json!(address.port());
        if listening(address.port()) {
            let status = owner(settings, json!({"action":"status"}))?;
            if let (Some(target), Some(source)) = (result.as_object_mut(), status.as_object()) {
                target.extend(source.clone());
            }
        }
    }
    Ok(result)
}
fn start(settings: &Settings, path: &std::path::Path) -> Result<(), String> {
    let address: std::net::SocketAddr = settings.bind.parse().map_err(|_| "Invalid bind")?;
    if listening(address.port()) {
        owner(settings, json!({"action":"status"}))?;
        return Ok(());
    }
    let log = path.parent().ok_or("Invalid settings path")?.join("service-stderr.log");
    if std::fs::metadata(&log).is_ok_and(|m| m.len() > 2 * 1024 * 1024) {
        std::fs::rename(&log, log.with_extension("previous.log")).map_err(|e| e.to_string())?;
    }
    let output = std::fs::OpenOptions::new().create(true).append(true).open(log).map_err(|e| e.to_string())?;
    let mut command = std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?);
    command.arg("--remote-service").arg(path).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(output);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        command.creation_flags(0x08000000);
    }
    let child = command.spawn().map_err(|e| format!("Cannot start the native remote service: {e}"))?;
    std::thread::spawn(move || {
        let mut child = child;
        if let Err(e) = child.wait() {
            log::debug!("Remote service wait failed: {e}");
        }
    });
    for _ in 0..50 {
        if listening(address.port()) {
            owner(settings, json!({"action":"status"}))?;
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("The native service did not become ready. See the service log in this profile's Remote Access folder".into())
}
fn handle(input: Value, control_port: Option<u16>) -> Result<Value, String> {
    let path = settings_path()?;
    let mut settings = if path.exists() { Some(Settings::read_for_owner_repair(&path)?) } else { None };
    let action = input.get("action").and_then(Value::as_str).unwrap_or("");
    match action {
        "status" => public_status(settings.as_ref()),
        "resync" => {
            let previous = settings.ok_or("Set up the connection first")?;
            let next = resync_folders(&previous, &path, &input)?;
            // The UI installs the saved authority before requesting start, exactly as setup does.
            public_status(Some(&next))
        }
        "configure" => {
            if settings.as_ref().is_some_and(|s| s.bind.parse::<std::net::SocketAddr>().is_ok_and(|a| listening(a.port()))) {
                return Err("Stop the connection before changing its setup".into());
            }
            let origin = input.get("publicOrigin").and_then(Value::as_str).unwrap_or("");
            let url = url_origin(origin)?;
            let phrase = input.get("passphrase").and_then(Value::as_str).unwrap_or("");
            let hash = if phrase.is_empty() {
                settings.as_ref().map(|s| s.owner_hash.clone()).ok_or("Choose an owner passphrase first")?
            } else {
                hash_phrase(phrase)?
            };
            let roots: Vec<PathBuf> = serde_json::from_value(input.get("artworkRoots").cloned().unwrap_or(Value::Null))
                .map_err(|_| "Choose one or more artwork folders")?;
            let root = roots.first().cloned().ok_or("Choose at least one artwork folder")?;
            let profile = profile()?;
            std::fs::create_dir_all(&profile).map_err(|e| e.to_string())?;
            protect_directory(path.parent().ok_or("Invalid settings folder")?)?;
            let next = Settings {
                version: 1,
                enabled: true,
                public_origin: url,
                base_path: "/vectorcraft-native".into(),
                bind: if input.get("lanAccess").and_then(Value::as_bool) == Some(true) { "0.0.0.0:3002" } else { "127.0.0.1:3002" }.into(),
                trusted_proxy: if input.get("lanAccess").and_then(Value::as_bool) == Some(true) {
                    Some(
                        input
                            .get("trustedProxy")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .parse()
                            .map_err(|_| "Enter the trusted gateway IP address")?,
                    )
                } else {
                    None
                },
                control_port: control_port.or_else(|| settings.as_ref().map(|s| s.control_port)).unwrap_or(33982),
                app_path: std::env::current_exe().map_err(|e| e.to_string())?,
                profile_dir: profile,
                artwork_root: root,
                additional_artwork_roots: roots.into_iter().skip(1).collect(),
                owner_hash: hash,
                management_token: settings.as_ref().map(|s| s.management_token.clone()).unwrap_or_else(secret),
                allow_test_callbacks: false,
            };
            next.validate()?;
            // Never silently replace live authorizations when the address changes.
            let state = path.parent().ok_or("Invalid settings folder")?.join("oauth-state.json");
            if settings.as_ref().is_some_and(|old| old.resource() != next.resource()) && state.exists() {
                let store = vectorcraft_mcp::remote::oauth::Store::open(next.clone(), &state)?;
                drop(store);
            }
            next.save(&path)?;
            settings = Some(next);
            // The UI installs the new file authority before starting the service. A closed
            // wizard or delayed frame must never expose the previous, broader folder grant.
            public_status(settings.as_ref())
        }
        "start" => {
            let mut s = settings.ok_or("Set up the connection first")?;
            s.enabled = true;
            s.save(&path)?;
            start(&s, &path)?;
            public_status(Some(&s))
        }
        "stop" => {
            let mut s = settings.ok_or("No connection is configured")?;
            owner(&s, json!({"action":"stop"}))?;
            let address: std::net::SocketAddr = s.bind.parse().map_err(|_| "Invalid bind")?;
            for _ in 0..50 {
                if !listening(address.port()) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            if listening(address.port()) {
                return Err("The connection is finishing requests. Wait before trying again".into());
            }
            s.enabled = false;
            s.save_disabled(&path)?;
            public_status(Some(&s))
        }
        "register" => {
            let s = settings.ok_or("Set up the connection first")?;
            owner(
                &s,
                json!({"action":"register","client":{"client_name":input.get("name").and_then(Value::as_str).unwrap_or("My AI app"),
                "redirect_uris":[input.get("callback").and_then(Value::as_str).unwrap_or("")],"token_endpoint_auth_method":"client_secret_post","grant_types":["authorization_code","refresh_token"],"response_types":["code"]}}),
            )
        }
        "revoke_client" | "revoke_all" | "resume" => {
            let s = settings.ok_or("No connection is configured")?;
            owner(&s, input)?;
            public_status(Some(&s))
        }
        "check_public" => {
            let s = settings.ok_or("Set up the connection first")?;
            s.validate()?;
            let response = client()?
                .get(format!("{}/health", s.issuer()))
                .send()
                .map_err(|_| "Cannot reach the public address. Check the gateway route and firewall")?;
            if !response.status().is_success() {
                return Err(format!("Public health check returned HTTP {}. Check the gateway route", response.status().as_u16()));
            }
            let health: Value = response.json().map_err(|_| "The public address returned a web page instead of the native service")?;
            if health.get("service").and_then(Value::as_str) != Some("vectorcraft-native") {
                return Err("The public route reaches a different service".into());
            }
            let mut status = public_status(Some(&s))?;
            status["publicReady"] = json!(true);
            Ok(status)
        }
        _ => Err("Unknown local connection action".into()),
    }
}
fn folder_settings(previous: &Settings, input: &Value) -> Result<Settings, String> {
    let roots: Vec<PathBuf> =
        serde_json::from_value(input.get("artworkRoots").cloned().unwrap_or(Value::Null)).map_err(|_| "Choose one or more artwork folders")?;
    let mut next = previous.clone();
    next.artwork_root = roots.first().cloned().ok_or("Choose at least one artwork folder")?;
    next.additional_artwork_roots = roots.into_iter().skip(1).collect();
    next.enabled = true;
    next.validate()?;
    Ok(next)
}
fn stop_running(settings: &Settings) -> Result<bool, String> {
    let address: std::net::SocketAddr = settings.bind.parse().map_err(|_| "Invalid bind")?;
    if !listening(address.port()) {
        return Ok(false);
    }
    owner(settings, json!({"action":"stop"}))?;
    for _ in 0..50 {
        if !listening(address.port()) {
            return Ok(true);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("The connection is finishing requests. Wait before trying again".into())
}
fn resync_folders(previous: &Settings, path: &std::path::Path, input: &Value) -> Result<Settings, String> {
    // Validate before stopping anything. Only folders change; OAuth clients, tokens and owner
    // credentials remain in place. A pending edit causes stop to refuse, rather than interrupt it.
    let next = folder_settings(previous, input)?;
    let was_running = stop_running(previous)?;
    if let Err(error) = next.save(path) {
        if was_running && let Err(restart) = start(previous, path) {
            return Err(format!("Folders were not saved: {error}. The previous connection could not restart: {restart}"));
        }
        return Err(error);
    }
    Ok(next)
}
fn url_origin(value: &str) -> Result<String, String> {
    // Config validation performs the HTTPS and origin checks; this normalizes scheme and host.
    let parsed = reqwest::Url::parse(value).map_err(|_| "Enter a complete HTTPS address")?;
    if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() || !parsed.username().is_empty() || parsed.password().is_some()
    {
        return Err("Enter the gateway origin without a path or query".into());
    }
    Ok(parsed.as_str().trim_end_matches('/').into())
}
pub fn service(control_port: Option<u16>) -> vectorcraft_ui_egui::RemoteAccessFn {
    Box::new(move |input| {
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let result = handle(input, control_port);
            let _ = sender.send(result);
        });
        receiver
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optional_private_store_failure_keeps_startup_available_and_disables_even_explicit_control() {
        let (port, roots) = super::super::remote_control_startup(Some(33982), Err("private directory protection failed".into()));
        assert!(port.is_none());
        assert!(roots.is_none());
        let (port, _) = super::super::remote_control_startup(Some(33982), Ok((None, None)));
        assert_eq!(port, Some(33982), "explicit CLI control remains deliberate when no remote setup failed");
    }
    #[test]
    fn fresh_profiles_do_not_create_private_folders_or_start_a_control_channel() {
        let root = std::env::temp_dir().join(format!("vectorcraft-unconfigured-{}", secret()));
        let (port, roots) = initial_control_in(&root).unwrap();
        assert!(port.is_none());
        assert!(roots.is_none());
        assert!(!root.exists());
    }
    #[test]
    fn disabled_saved_connections_do_not_implicitly_open_control() {
        let (mut settings, path, root) = fixture();
        settings.enabled = false;
        settings.save(&path).unwrap();
        let (port, _) = initial_control_in(&settings.profile_dir).unwrap();
        assert!(port.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn a_running_legacy_service_stays_visible_to_the_owner_while_network_policy_needs_repair() {
        use std::io::{Read as _, Write as _};
        let (mut settings, _, root) = fixture();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        settings.bind = format!("0.0.0.0:{}", listener.local_addr().unwrap().port());
        assert!(settings.validate().is_err());
        let responder = std::thread::spawn(move || {
            for socket in listener.incoming() {
                let mut socket = socket.unwrap();
                socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                let mut request = [0; 4096];
                if socket.read(&mut request).unwrap_or(0) == 0 {
                    continue;
                }
                let body = r#"{"running":true,"clients":[],"pending":0}"#;
                write!(
                    socket,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
                break;
            }
        });
        let status = public_status(Some(&settings)).unwrap();
        assert_eq!(status["running"], true, "the wizard must show Stop for a running legacy service");
        assert!(status["configurationError"].is_string());
        responder.join().unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn command_line_roots_cannot_override_private_exclusions_or_saved_remote_grants() {
        assert!(initial_control(true).unwrap_err().contains("cannot be combined"));
    }
    fn fixture() -> (Settings, PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("vectorcraft-host-resync-{}", secret()));
        let profile = root.join("profile");
        let artwork = root.join("artwork");
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::create_dir_all(&artwork).unwrap();
        protect_directory(&profile.join("Remote Access")).unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let settings = Settings {
            version: 1,
            enabled: true,
            public_origin: format!("http://127.0.0.1:{port}"),
            base_path: "/vectorcraft-native".into(),
            bind: format!("127.0.0.1:{port}"),
            trusted_proxy: None,
            control_port: 33982,
            app_path: std::env::current_exe().unwrap(),
            profile_dir: profile.clone(),
            artwork_root: artwork,
            additional_artwork_roots: vec![],
            owner_hash: hash_phrase("test fixture passphrase").unwrap(),
            management_token: secret(),
            allow_test_callbacks: true,
        };
        (settings, profile.join("Remote Access/settings.json"), root)
    }
    #[test]
    fn resync_changes_only_folders_and_preserves_oauth_state_and_credentials() {
        let (settings, path, root) = fixture();
        let second = root.join("second");
        std::fs::create_dir_all(&second).unwrap();
        settings.save(&path).unwrap();
        let state = path.with_file_name("oauth-state.json");
        std::fs::write(&state, "existing opaque OAuth state").unwrap();
        let next =
            resync_folders(&settings, &path, &json!({"artworkRoots":[second],"passphrase":"ignored","publicOrigin":"https://ignored.example"}))
                .unwrap();
        let mut expected = settings.clone();
        expected.artwork_root = second;
        assert_eq!(serde_json::to_value(&next).unwrap(), serde_json::to_value(expected).unwrap());
        assert_eq!(std::fs::read_to_string(&state).unwrap(), "existing opaque OAuth state");
        assert_eq!(Settings::read(&path).unwrap().artwork_root, next.artwork_root);
        assert!(folder_settings(&next, &json!({"artworkRoots":[]})).is_err());
        assert!(folder_settings(&next, &json!({"artworkRoots":[root.join("missing")]})).is_err());
        assert!(folder_settings(&next, &json!({"artworkRoots":[path.parent().unwrap()]})).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn a_pending_edit_prevents_resync_without_saving_or_revoking_anything() {
        use std::io::{Read as _, Write as _};
        let (mut settings, path, root) = fixture();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        settings.bind = listener.local_addr().unwrap().to_string();
        settings.save(&path).unwrap();
        let original = std::fs::read(&path).unwrap();
        let responder = std::thread::spawn(move || {
            for incoming in listener.incoming() {
                let mut socket = incoming.unwrap();
                socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                let mut request = [0; 4096];
                if socket.read(&mut request).unwrap_or(0) == 0 {
                    continue;
                }
                let body = r#"{"error":"Work is still running; finish it before stopping the connection"}"#;
                write!(
                    socket,
                    "HTTP/1.1 409 Conflict\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
                break;
            }
        });
        let outcome = resync_folders(&settings, &path, &json!({"artworkRoots":[settings.artwork_root]}));
        assert!(outcome.err().unwrap().contains("still running"));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        responder.join().unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn an_unavailable_artwork_folder_keeps_the_editor_confined_and_the_setup_repairable() {
        let (settings, path, root) = fixture();
        settings.save(&path).unwrap();
        std::fs::remove_dir(&settings.artwork_root).unwrap();
        assert!(Settings::read(&path).is_err());
        let saved = Settings::read_for_owner_repair(&path).unwrap();
        assert!(initial_control_in(&settings.profile_dir).is_err(), "invalid setup never activates control");
        let status = public_status(Some(&saved)).unwrap();
        assert_eq!(status["configured"], true);
        assert_eq!(status["running"], false);
        assert!(status["configurationError"].is_string());
        let replacement = root.join("replacement");
        std::fs::create_dir(&replacement).unwrap();
        let repaired = resync_folders(&saved, &path, &json!({"artworkRoots":[replacement]})).unwrap();
        assert_eq!(Settings::read(&path).unwrap().artwork_root, repaired.artwork_root);
        assert!(!repaired.artwork_root.as_os_str().is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}
