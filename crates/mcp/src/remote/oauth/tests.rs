use super::*;
use crate::remote::config::{check_phrase, hash_phrase, protect_directory};
const PHRASE: &str = "A private native test passphrase";
struct Fixture {
    root: tempfile::TempDir,
    settings: Settings,
    store: Store,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let profile = root.path().join("profile");
        let artwork = root.path().join("artwork");
        std::fs::create_dir_all(profile.join("Remote Access")).unwrap();
        std::fs::create_dir_all(&artwork).unwrap();
        protect_directory(&profile.join("Remote Access")).unwrap();
        static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        let settings = Settings {
            version: 1,
            enabled: true,
            public_origin: "https://native.example.test".into(),
            base_path: "/vectorcraft-native".into(),
            bind: "127.0.0.1:33981".into(),
            control_port: 33982,
            app_path: std::env::current_exe().unwrap(),
            profile_dir: profile.clone(),
            artwork_root: artwork,
            additional_artwork_roots: Vec::new(),
            owner_hash: HASH.get_or_init(|| hash_phrase(PHRASE).unwrap()).clone(),
            management_token: secret(),
            allow_test_callbacks: false,
        };
        settings.validate().unwrap();
        let store = Store::open(settings.clone(), &profile.join("Remote Access/oauth-state.json")).unwrap();
        Self { root, settings, store }
    }
    fn client(&mut self, method: &str, callback: &str) -> (Client, Value) {
        let value=self.store.register(json!({"client_name":"Test <unsafe>","redirect_uris":[callback],"token_endpoint_auth_method":method,"grant_types":["authorization_code","refresh_token"],"response_types":["code"]})).unwrap();
        (self.store.db.clients.get(value["client_id"].as_str().unwrap()).unwrap().clone(), value)
    }
    fn begin(&mut self, client: &Client) -> (String, String, String, Fields) {
        let verifier = secret();
        let mut fields = Fields::new();
        for (k, v) in [
            ("response_type", "code".to_string()),
            ("client_id", client.client_id.clone()),
            ("redirect_uri", client.redirect_uris[0].clone()),
            ("state", "a + / = & %".into()),
            ("code_challenge", digest(&verifier)),
            ("code_challenge_method", "S256".into()),
            ("resource", self.settings.resource()),
            ("scope", SCOPE.into()),
        ] {
            fields.insert(k.into(), v);
        }
        let (id, cookie, _) = self.store.authorize(&fields).unwrap();
        (id, cookie, verifier, fields)
    }
    fn grant(&mut self, client: &Client) -> (Value, Fields) {
        let (id, cookie, verifier, _) = self.begin(client);
        let destination = self.store.approve(&id, &cookie, true).unwrap();
        let url = Url::parse(&destination).unwrap();
        let query = fields(url.query().unwrap()).unwrap();
        assert_eq!(query.get("state").unwrap(), "a + / = & %");
        assert_eq!(query.get("iss").unwrap(), &self.settings.issuer());
        let mut form = Fields::new();
        for (k, v) in [
            ("grant_type", "authorization_code".into()),
            ("client_id", client.client_id.clone()),
            ("redirect_uri", client.redirect_uris[0].clone()),
            ("code", query["code"].clone()),
            ("code_verifier", verifier),
            ("resource", self.settings.resource()),
        ] {
            form.insert(k.into(), v);
        }
        (self.store.token(&form, client).unwrap(), form)
    }
}
#[test]
fn code_is_single_use_and_tokens_survive_restart_without_plaintext_secrets() {
    let mut f = Fixture::new();
    let (client, registration) = f.client("client_secret_post", "https://chatgpt.com/connector/oauth/test");
    let (tokens, form) = f.grant(&client);
    assert!(f.store.valid_access(tokens["access_token"].as_str().unwrap()));
    assert_eq!(f.store.token(&form, &client).unwrap_err().code, "invalid_grant");
    let bytes = std::fs::read_to_string(&f.store.path).unwrap();
    for value in
        [tokens["access_token"].as_str().unwrap(), tokens["refresh_token"].as_str().unwrap(), registration["client_secret"].as_str().unwrap(), PHRASE]
    {
        assert!(!bytes.contains(value));
    }
    let loaded = Store::open(f.settings.clone(), &f.store.path).unwrap();
    assert!(loaded.valid_access(tokens["access_token"].as_str().unwrap()));
    assert!(f.root.path().is_dir());
}
#[test]
fn pkce_callback_resource_scope_cookie_and_expiry_are_enforced() {
    let mut f = Fixture::new();
    let (client, _) = f.client("none", "https://chatgpt.com/connector/oauth/test");
    let (id, cookie, verifier, mut start) = f.begin(&client);
    assert!(f.store.pending(&id, "wrong").is_err());
    start.insert("redirect_uri".into(), "https://chatgpt.com/other".into());
    assert!(f.store.authorize(&start).is_err());
    let destination = f.store.approve(&id, &cookie, true).unwrap();
    let parsed = Url::parse(&destination).unwrap();
    let code = fields(parsed.query().unwrap()).unwrap()["code"].clone();
    let mut form = fields(&format!("grant_type=authorization_code&code={code}&code_verifier={verifier}")).unwrap();
    form.insert("redirect_uri".into(), client.redirect_uris[0].clone());
    for (key, bad) in [
        ("code_verifier", "x".repeat(43)),
        ("redirect_uri", "https://evil.example".into()),
        ("resource", "https://other.example/mcp".into()),
        ("scope", "other:scope".into()),
    ] {
        let mut bad_form = form.clone();
        bad_form.insert(key.into(), bad);
        assert!(f.store.token(&bad_form, &client).is_err(), "{key}");
    }
    f.store.codes.get_mut(&digest(&code)).unwrap().grant.expires = now() - 1;
    assert!(f.store.token(&form, &client).is_err());
    let (id, cookie, _, _) = f.begin(&client);
    f.store.pending.get_mut(&digest(&id)).unwrap().expires = now() - 1;
    assert!(f.store.pending(&id, &cookie).is_err());
}
#[test]
fn refresh_rotation_replay_revokes_only_its_connection_family() {
    let mut f = Fixture::new();
    let (client, _) = f.client("none", "https://chatgpt.com/connector/oauth/test");
    let (first, _) = f.grant(&client);
    let (other, _) = f.grant(&client);
    let mut refresh = Fields::new();
    refresh.insert("grant_type".into(), "refresh_token".into());
    refresh.insert("refresh_token".into(), first["refresh_token"].as_str().unwrap().into());
    let next = f.store.token(&refresh, &client).unwrap();
    assert_ne!(next["refresh_token"], first["refresh_token"]);
    assert!(f.store.valid_access(next["access_token"].as_str().unwrap()));
    assert!(f.store.token(&refresh, &client).is_err());
    assert!(!f.store.valid_access(first["access_token"].as_str().unwrap()));
    assert!(!f.store.valid_access(next["access_token"].as_str().unwrap()));
    assert!(f.store.valid_access(other["access_token"].as_str().unwrap()));
}
#[test]
fn public_post_and_basic_client_authentication_are_distinct() {
    let mut f = Fixture::new();
    for method in ["none", "client_secret_post", "client_secret_basic"] {
        let (client, value) = f.client(method, "https://oauth-redirect.googleusercontent.com/r/user_bound_custom-mcp-test");
        let mut form = Fields::new();
        form.insert("client_id".into(), client.client_id.clone());
        match method {
            "none" => {
                assert!(f.store.authenticate(&form, None).is_ok());
                form.insert("client_secret".into(), "unexpected".into());
                assert!(f.store.authenticate(&form, None).is_err());
            }
            "client_secret_post" => {
                assert!(f.store.authenticate(&form, None).is_err());
                form.insert("client_secret".into(), value["client_secret"].as_str().unwrap().into());
                assert!(f.store.authenticate(&form, None).is_ok());
                assert!(f.store.authenticate(&form, Some((&client.client_id, value["client_secret"].as_str().unwrap()))).is_err());
            }
            _ => {
                assert!(f.store.authenticate(&form, None).is_err());
                assert!(f.store.authenticate(&form, Some((&client.client_id, value["client_secret"].as_str().unwrap()))).is_ok());
            }
        }
    }
}
#[test]
fn revocation_removes_access_refresh_and_pending_client_grants() {
    let mut f = Fixture::new();
    let (client, _) = f.client("none", "https://chatgpt.com/connector/oauth/test");
    let (tokens, _) = f.grant(&client);
    f.store.revoke(tokens["access_token"].as_str().unwrap(), &client).unwrap();
    assert!(!f.store.valid_access(tokens["access_token"].as_str().unwrap()));
    let (tokens, _) = f.grant(&client);
    let (id, cookie, _, _) = f.begin(&client);
    f.store.remove_client(&client.client_id).unwrap();
    assert!(!f.store.valid_access(tokens["access_token"].as_str().unwrap()));
    assert!(f.store.pending(&id, &cookie).is_err());
}
#[test]
fn authorization_cancellation_escapes_the_page_and_preserves_state() {
    let mut f = Fixture::new();
    let (client, _) = f.client("none", "https://oauth-redirect.googleusercontent.com/r/user_bound_custom-mcp-test");
    let (id, cookie, _, _) = f.begin(&client);
    let pending = f.store.pending(&id, &cookie).unwrap();
    let page = crate::remote::pages::consent(&f.settings, &id, &pending, None);
    assert!(page.contains("Test &lt;unsafe&gt;"));
    assert!(!page.contains("Test <unsafe>"));
    let csp = crate::remote::pages::csp(&pending.redirect);
    assert!(csp.contains("https://oauth-redirect.googleusercontent.com"));
    assert!(csp.contains("https://gemini.google.com"));
    let cancel = f.store.approve(&id, &cookie, false).unwrap();
    let parsed = Url::parse(&cancel).unwrap();
    let params = fields(parsed.query().unwrap()).unwrap();
    assert_eq!(params["error"], "access_denied");
    assert_eq!(params["state"], "a + / = & %");
    assert!(f.store.approve(&id, &cookie, true).is_err());
}
#[test]
fn owner_password_hash_and_artwork_boundary_are_validated() {
    let mut f = Fixture::new();
    assert!(check_phrase(PHRASE, &f.settings.owner_hash));
    assert!(!check_phrase("not the phrase", &f.settings.owner_hash));
    f.settings.artwork_root = f.settings.profile_dir.join("Remote Access");
    assert!(f.settings.validate().is_err());
    f.settings.artwork_root = f.root.path().to_path_buf();
    assert!(f.settings.validate().is_ok());
    let roots = f.settings.automation_roots().unwrap().unwrap();
    assert!(
        roots
            .check(&f.settings.profile_dir.join("Remote Access/settings.json").to_string_lossy(), vectorcraft_engine::file_access::Access::Read)
            .is_err()
    );
}
#[test]
fn owner_phrase_limits_count_characters_rather_than_utf8_bytes() {
    assert!(hash_phrase(&"界".repeat(4)).is_err());
    assert!(hash_phrase(&"界".repeat(11)).is_err());
    for length in [12, 256] {
        let phrase = "界".repeat(length);
        let hash = hash_phrase(&phrase).unwrap();
        assert!(check_phrase(&phrase, &hash));
        assert!(!check_phrase(&(phrase + "x"), &hash));
    }
    assert!(hash_phrase(&"界".repeat(257)).is_err());
    let phrase = "🔒".repeat(256);
    assert!(check_phrase(&phrase, &hash_phrase(&phrase).unwrap()));
    let f = Fixture::new();
    let pending = Pending {
        client_id: String::new(),
        client_name: String::new(),
        redirect: "https://chatgpt.com/callback".into(),
        state: None,
        challenge: String::new(),
        cookie_hash: String::new(),
        expires: now() + 60,
    };
    // HTML counts UTF-16 units: allow a surrogate pair for each of the 256 characters.
    assert!(crate::remote::pages::consent(&f.settings, "test", &pending, None).contains("maxlength=\"512\""));
}
#[test]
fn settings_cannot_be_saved_or_served_outside_the_excluded_private_folder() {
    let f = Fixture::new();
    let private = f.settings.profile_dir.join("Remote Access/settings.json");
    f.settings.save(&private).unwrap();
    assert!(Settings::read(&private).is_ok());
    let original = std::fs::read(&private).unwrap();
    let mut oversized = f.settings.clone();
    oversized.management_token = "m".repeat(64 * 1024);
    assert!(oversized.save(&private).is_err());
    assert_eq!(std::fs::read(&private).unwrap(), original);
    let public = f.settings.artwork_root.join("settings.json");
    assert!(f.settings.save(&public).is_err());
    assert!(!public.exists());
    std::fs::write(&public, serde_json::to_vec(&f.settings).unwrap()).unwrap();
    assert!(Settings::read(&public).is_err());
}
#[cfg(unix)]
#[test]
fn private_settings_and_state_refuse_symbolic_link_targets() {
    let f = Fixture::new();
    let outside = f.settings.artwork_root.join("outside.json");
    let original = serde_json::to_vec(&f.settings).unwrap();
    std::fs::write(&outside, &original).unwrap();
    let private = f.settings.profile_dir.join("Remote Access/settings.json");
    std::os::unix::fs::symlink(&outside, &private).unwrap();
    assert!(Settings::read(&private).is_err());
    assert!(f.settings.save(&private).is_err());
    assert!(write_private(&private, b"must not be written").is_err());
    assert_eq!(std::fs::read(&outside).unwrap(), original);
}
#[cfg(windows)]
#[test]
fn protecting_private_storage_removes_old_explicit_grants_from_folder_and_file() {
    use std::os::windows::process::CommandExt as _;
    fn entries(path: &Path) -> usize {
        let export = path.parent().unwrap().join(format!(".test-permissions-{}.acl", secret()));
        let utility = PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/icacls.exe");
        assert!(
            std::process::Command::new(utility).arg(path).arg("/save").arg(&export).creation_flags(0x08000000).output().unwrap().status.success()
        );
        let bytes = std::fs::read(&export).unwrap();
        std::fs::remove_file(export).unwrap();
        let (pairs, trailing) = bytes.as_chunks::<2>();
        assert!(trailing.is_empty());
        let units: Vec<u16> = pairs.iter().copied().map(u16::from_le_bytes).collect();
        String::from_utf16(&units).unwrap().matches("(").count()
    }
    let f = Fixture::new();
    let directory = f.settings.profile_dir.join("Remote Access");
    let file = directory.join("settings.json");
    f.settings.save(&file).unwrap();
    let original = std::fs::read(&file).unwrap();
    let utility = PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/icacls.exe");
    for target in [&directory, &file] {
        assert!(
            std::process::Command::new(&utility)
                .arg(target)
                .args(["/grant", "*S-1-1-0:R"])
                .creation_flags(0x08000000)
                .output()
                .unwrap()
                .status
                .success()
        );
        assert!(entries(target) > 2);
    }
    protect_directory(&directory).unwrap();
    assert_eq!(entries(&directory), 2, "only the owner and SYSTEM");
    assert_eq!(entries(&file), 2, "old explicit file grants must be replaced too");
    assert_eq!(std::fs::read(file).unwrap(), original);
}
#[test]
fn single_folder_settings_migrate_and_extra_folders_are_validated() {
    let mut f = Fixture::new();
    let mut legacy = serde_json::to_value(&f.settings).unwrap();
    legacy.as_object_mut().unwrap().remove("additionalArtworkRoots");
    let migrated: Settings = serde_json::from_value(legacy).unwrap();
    assert_eq!(migrated.artwork_roots(), vec![f.settings.artwork_root.clone()]);
    let second = f.root.path().join("second");
    std::fs::create_dir(&second).unwrap();
    f.settings.additional_artwork_roots.push(second.clone());
    f.settings.validate().unwrap();
    assert_eq!(f.settings.artwork_roots(), vec![f.settings.artwork_root.clone(), second.clone()]);
    f.settings.additional_artwork_roots.push(f.root.path().join("missing"));
    assert!(f.settings.validate().is_err());
    f.settings.additional_artwork_roots = vec![second; 32];
    assert!(f.settings.validate().is_err());
}
#[test]
fn expired_access_and_refresh_cannot_authorize_or_issue_tokens() {
    let mut f = Fixture::new();
    let (client, _) = f.client("none", "https://chatgpt.com/connector/oauth/expiry");
    let (tokens, _) = f.grant(&client);
    let access = tokens["access_token"].as_str().unwrap();
    let refresh = tokens["refresh_token"].as_str().unwrap();
    f.store.db.access.get_mut(&digest(access)).unwrap().expires = now() - 1;
    f.store.db.refresh.get_mut(&digest(refresh)).unwrap().expires = now() - 1;
    assert!(!f.store.valid_access(access));
    let mut form = Fields::new();
    form.insert("grant_type".into(), "refresh_token".into());
    form.insert("refresh_token".into(), refresh.into());
    assert_eq!(f.store.token(&form, &client).unwrap_err().code, "invalid_grant");
}

#[test]
fn persistence_failure_does_not_issue_a_usable_token() {
    let mut f = Fixture::new();
    let (client, _) = f.client("none", "https://chatgpt.com/connector/oauth/test");
    let (id, cookie, verifier, _) = f.begin(&client);
    let destination = f.store.approve(&id, &cookie, true).unwrap();
    let parsed = Url::parse(&destination).unwrap();
    let params = fields(parsed.query().unwrap()).unwrap();
    let mut form = fields("grant_type=authorization_code").unwrap();
    form.insert("code".into(), params["code"].clone());
    form.insert("redirect_uri".into(), client.redirect_uris[0].clone());
    form.insert("code_verifier".into(), verifier);
    f.store.path = f.root.path().join("missing-parent/state.json");
    assert!(f.store.token(&form, &client).is_err());
    assert!(f.store.db.access.is_empty());
}
