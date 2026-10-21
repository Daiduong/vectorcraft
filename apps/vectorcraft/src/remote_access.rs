//! Native host side of Remote AI connections. File/process operations stay out of the UI crate.
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::Duration;
use vectorcraft_mcp::remote::config::{Settings, hash_phrase, protect_directory, secret};

/// Find the selected desktop profile folder, or report that preferences are unavailable.
pub fn profile() -> Result<PathBuf, String> {
    super::prefs_path().and_then(|p| p.parent().map(std::path::Path::to_path_buf)).ok_or_else(|| "This session has no saved profile folder".into())
}
/// Locate connection settings inside this profile's private Remote Access folder.
fn settings_path() -> Result<PathBuf, String> {
    Ok(profile()?.join("Remote Access/settings.json"))
}
/// Authorize a delegated desktop call using this profile's current saved connection settings.
/// The UI supplies its installed grants; no remote caller can replace them or select a profile.
pub fn authorize_desktop(roots: &vectorcraft_engine::file_access::AutomationRoots, capability: &str) -> Result<(), String> {
    let settings = Settings::read(&settings_path()?)?;
    settings.authorize_desktop(roots, capability, &std::env::current_exe().map_err(|e| e.to_string())?)
}
pub type InitialControl = (Option<u16>, Option<std::sync::Arc<vectorcraft_engine::file_access::AutomationRoots>>);
/// Load saved artwork-folder grants and the enabled connection's control port.
/// Reject CLI folder overrides; Remote AI grants are managed in Preferences > Remote AI.
pub fn initial_control(explicit_roots: bool) -> Result<InitialControl, String> {
    if explicit_roots {
        return Err("Native Remote AI folders cannot be combined with command-line automation roots; manage grants in Preferences > Remote AI".into());
    }
    let profile = profile()?;
    initial_control_in(&profile)
}
/// Load and validate existing connection settings before granting automated file access.
/// A fresh profile creates no private folders and starts no control channel.
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
/// Build a short-deadline HTTP client that never follows redirects.
/// Use it for local connection management and the public gateway check.
fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())
}
/// Send an authenticated management request to this computer's connection service.
/// Return service errors without exposing the private management token to the wizard.
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
/// Check briefly whether the local HTTP port accepts a connection.
/// This is only a readiness probe; callers authenticate separately.
fn listening(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(&std::net::SocketAddr::from(([127, 0, 0, 1], port)), Duration::from_millis(200)).is_ok()
}
/// Build the wizard's connection status without returning passphrases or tokens.
/// Keep a running service visible for Stop even when saved settings need repair.
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
/// Start or authenticate the native connection service and wait for local readiness.
/// Record startup errors in the private profile log; do not open the editor here.
fn start(settings: &Settings, path: &std::path::Path) -> Result<(), String> {
    settings.validate()?;
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
/// Perform a local owner action from the Remote AI connections wizard.
/// Saving setup or resyncing folders returns status so the UI can install grants before Start.
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
            disable(&mut s, &path)?;
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
/// Persist the owner's Stop choice even when the service already exited.
/// A running service must finish its work successfully before its setup is disabled.
fn disable(settings: &mut Settings, path: &std::path::Path) -> Result<(), String> {
    stop_running(settings)?;
    settings.enabled = false;
    settings.save_disabled(path)
}
/// Validate replacement artwork folders while preserving all other connection settings.
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
/// Ask a running connection to stop and wait for its HTTP listener to close.
/// Refuse to interrupt pending work; return whether a service was running.
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
/// Save only the approved artwork-folder changes, keeping linked apps and the passphrase.
/// Refuse resync while work is still running; the UI installs saved grants before restarting.
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
/// Normalize the gateway address and reject paths, queries, fragments and credentials.
/// Full connection validation separately enforces the HTTPS policy.
fn url_origin(value: &str) -> Result<String, String> {
    // Config validation performs the HTTPS and origin checks; this normalizes scheme and host.
    let parsed = reqwest::Url::parse(value).map_err(|_| "Enter a complete HTTPS address")?;
    if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() || !parsed.username().is_empty() || parsed.password().is_some()
    {
        return Err("Enter the gateway origin without a path or query".into());
    }
    Ok(parsed.as_str().trim_end_matches('/').into())
}
/// Expose asynchronous local-owner operations to the wizard without blocking UI frames.
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
    /// Verify that private-settings failure leaves the editor usable with control disabled.
    /// An otherwise valid fresh session may still use an explicit CLI control port.
    #[test]
    fn optional_private_store_failure_keeps_startup_available_and_disables_even_explicit_control() {
        let (port, roots) = super::super::remote_control_startup(Some(33982), Err("private directory protection failed".into()));
        assert!(port.is_none());
        assert!(roots.is_none());
        let (port, _) = super::super::remote_control_startup(Some(33982), Ok((None, None)));
        assert_eq!(port, Some(33982), "explicit CLI control remains deliberate when no remote setup failed");
    }
    /// Verify that an unconfigured Remote AI build leaves a fresh profile untouched.
    #[test]
    fn fresh_profiles_do_not_create_private_folders_or_start_a_control_channel() {
        let root = std::env::temp_dir().join(format!("vectorcraft-unconfigured-{}", secret()));
        let (port, roots) = initial_control_in(&root).unwrap();
        assert!(port.is_none());
        assert!(roots.is_none());
        assert!(!root.exists());
    }
    /// Verify that saved Stop state prevents automatic control-channel startup.
    #[test]
    fn disabled_saved_connections_do_not_implicitly_open_control() {
        let (mut settings, path, root) = fixture();
        settings.enabled = false;
        settings.save(&path).unwrap();
        let (port, _) = initial_control_in(&settings.profile_dir).unwrap();
        assert!(port.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    /// Stop remains persistent after a crash, and repeating Stop keeps automatic control disabled.
    #[test]
    fn stopping_an_already_exited_service_persists_disabled_state() {
        let (mut settings, path, root) = fixture();
        settings.save(&path).unwrap();
        disable(&mut settings, &path).unwrap();
        assert!(!Settings::read(&path).unwrap().enabled);
        assert!(initial_control_in(&settings.profile_dir).unwrap().0.is_none());
        disable(&mut settings, &path).unwrap();
        assert!(!Settings::read(&path).unwrap().enabled);
        std::fs::remove_dir_all(root).unwrap();
    }
    /// A desktop capability cannot authorize another profile, broader grants or missing exclusions.
    #[test]
    fn desktop_delegation_requires_the_saved_profile_build_and_actual_grants() {
        use vectorcraft_engine::file_access::AutomationRoots;
        let (settings, path, root) = fixture();
        settings.save(&path).unwrap();
        let installed = settings.automation_roots().unwrap().unwrap();
        let capability = settings.desktop_capability().unwrap();
        assert!(!capability.contains(&settings.management_token));
        assert!(settings.authorize_desktop(&installed, &capability, &settings.app_path).is_ok());
        let broader = AutomationRoots::new(Some(&root), Some(&root)).unwrap().unwrap();
        assert!(settings.authorize_desktop(&broader, &capability, &settings.app_path).is_err());
        let no_exclusion = AutomationRoots::new(Some(&settings.artwork_root), Some(&settings.artwork_root)).unwrap().unwrap();
        assert!(settings.authorize_desktop(&no_exclusion, &capability, &settings.app_path).is_err());
        let mut other = settings.clone();
        other.profile_dir = root.join("other-profile");
        std::fs::create_dir_all(other.profile_dir.join("Remote Access")).unwrap();
        assert_ne!(other.desktop_capability().unwrap(), capability);
        assert!(other.authorize_desktop(&installed, &capability, &settings.app_path).is_err());
        let other_build = root.join("another-build.exe");
        std::fs::write(&other_build, b"not executable; identity test only").unwrap();
        assert!(settings.authorize_desktop(&installed, &capability, &other_build).is_err());
        other = settings.clone();
        other.enabled = false;
        assert!(other.authorize_desktop(&installed, &capability, &settings.app_path).is_err());
        other = settings.clone();
        other.management_token = secret();
        assert!(other.authorize_desktop(&installed, &capability, &settings.app_path).is_err());
        other = settings.clone();
        other.additional_artwork_roots.push(root.clone());
        assert_ne!(other.desktop_capability().unwrap(), capability);
        assert!(other.authorize_desktop(&installed, &capability, &settings.app_path).is_err());
        assert!(other.authorize_desktop(&other.automation_roots().unwrap().unwrap(), &capability, &settings.app_path).is_err());
        assert!(!public_status(Some(&settings)).unwrap().to_string().contains(&capability));
        std::fs::remove_dir_all(root).unwrap();
    }
    /// Keep Stop available when an older running connection lacks the new gateway setting.
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
    /// Reject CLI folder overrides that would compete with the wizard's approved folders.
    #[test]
    fn command_line_roots_cannot_override_private_exclusions_or_saved_remote_grants() {
        assert!(initial_control(true).unwrap_err().contains("cannot be combined"));
    }
    /// Reject an unreachable owner listener before readiness probes, log creation or process launch.
    #[test]
    fn invalid_owner_listener_is_rejected_before_startup_side_effects() {
        let (mut settings, _, root) = fixture();
        settings.trusted_proxy = Some("192.0.2.10".parse().unwrap());
        // A missing log parent also prevents a child launch if the validation guard regresses.
        let path = root.join("missing/settings.json");
        for host in ["192.0.2.20", "127.0.0.2"] {
            settings.bind = format!("{host}:3002");
            let error = start(&settings, &path).unwrap_err();
            assert!(error.contains("127.0.0.1"), "{host}: {error}");
            assert!(!root.join("missing").exists());
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    /// Create isolated profile and artwork folders with loopback-only test connection settings.
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
    /// Verify that Resync folders changes grants without replacing linked-app state or credentials.
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
    /// Verify that pending work blocks Resync folders before settings or credentials change.
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
    /// Verify that a missing artwork folder blocks automated access while local repair remains possible.
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
