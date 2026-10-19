use super::{
    SCOPE,
    config::{Settings, digest, now, secret, write_private},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;
use url::Url;

pub type Fields = BTreeMap<String, String>;
pub type AuthResult<T> = Result<T, Failure>;
#[derive(Debug)]
pub struct Failure {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
}

#[cfg(test)]
mod tests;
impl Failure {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { status: 400, code, message: message.into() }
    }
    pub fn unauthorized() -> Self {
        Self { status: 401, code: "invalid_client", message: "Client authentication failed".into() }
    }
}
pub fn equal(a: &str, b: &str) -> bool {
    bool::from(a.as_bytes().ct_eq(b.as_bytes()))
}
pub fn fields(text: &str) -> AuthResult<Fields> {
    let mut result = Fields::new();
    for (k, v) in url::form_urlencoded::parse(text.as_bytes()) {
        if result.len() >= 32 || k.len() > 100 || v.len() > 8192 || result.insert(k.into_owned(), v.into_owned()).is_some() {
            return Err(Failure::new("invalid_request", "Invalid or repeated parameter"));
        }
    }
    Ok(result)
}
pub fn allowed_callback(text: &str, test: bool) -> bool {
    if text.len() > 2048 {
        return false;
    }
    let Ok(url) = Url::parse(text) else { return false };
    if url.fragment().is_some() || !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    if test && url.scheme() == "http" && matches!(url.host_str(), Some("127.0.0.1" | "localhost")) {
        return true;
    }
    if url.scheme() != "https" || url.port().is_some() {
        return false;
    }
    match url.host_str() {
        Some("chatgpt.com" | "chat.openai.com") => true,
        Some("oauth-redirect.googleusercontent.com" | "oauth-redirect-sandbox.googleusercontent.com") => {
            url.path().strip_prefix("/r/").is_some_and(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_alphanumeric() || b"._~-".contains(&b)))
                && url.query().is_none()
        }
        _ => false,
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Client {
    pub client_id: String,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub token_endpoint_auth_method: String,
    pub client_id_issued_at: u64,
    pub secret_hash: Option<String>,
    /// Public registrations expire unless the owner approves them. Legacy and locally
    /// registered clients retain their existing lifetime.
    #[serde(default)]
    pub registration_expires: Option<u64>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Grant {
    pub client_id: String,
    pub resource: String,
    pub family: String,
    pub expires: u64,
    #[serde(default)]
    pub used: bool,
}
#[derive(Clone, Serialize, Deserialize, Default)]
struct Database {
    resource: String,
    clients: BTreeMap<String, Client>,
    access: BTreeMap<String, Grant>,
    refresh: BTreeMap<String, Grant>,
}
#[derive(Clone)]
pub struct Pending {
    pub client_id: String,
    pub client_name: String,
    pub redirect: String,
    pub state: Option<String>,
    pub challenge: String,
    pub cookie_hash: String,
    pub expires: u64,
}
struct Code {
    grant: Grant,
    redirect: String,
    challenge: String,
}
pub struct Store {
    pub settings: Settings,
    path: PathBuf,
    db: Database,
    pending: HashMap<String, Pending>,
    codes: HashMap<String, Code>,
    attempts: HashMap<String, u8>,
}
impl Store {
    pub fn open(settings: Settings, path: &Path) -> Result<Self, String> {
        let mut db = if path.exists() {
            if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 8 * 1024 * 1024 {
                return Err("Authorization store is too large".into());
            }
            serde_json::from_slice::<Database>(&std::fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|_| "Authorization store is invalid; it was left unchanged")?
        } else {
            Database { resource: settings.resource(), ..Default::default() }
        };
        if db.resource != settings.resource() {
            if db.clients.is_empty() && db.access.is_empty() && db.refresh.is_empty() {
                db.resource = settings.resource();
            } else {
                return Err("The public address changed. Reset connections locally before using the new address".into());
            }
        }
        Self::prune_db(&mut db);
        Ok(Self { settings, path: path.into(), db, pending: HashMap::new(), codes: HashMap::new(), attempts: HashMap::new() })
    }
    fn prune_db(db: &mut Database) {
        let time = now();
        db.access.retain(|_, g| g.expires > time);
        db.refresh.retain(|_, g| g.expires > time);
        db.clients.retain(|_, c| c.registration_expires.is_none_or(|expiry| expiry > time));
    }
    fn commit(&mut self, mut db: Database) -> AuthResult<()> {
        Self::prune_db(&mut db);
        if db.access.len() > 4096 || db.refresh.len() > 4096 {
            return Err(Failure::new("temporarily_unavailable", "Connection limit reached; revoke unused connections locally"));
        }
        let bytes = serde_json::to_vec(&db).map_err(|_| Failure::new("server_error", "Cannot encode authorization state"))?;
        write_private(&self.path, &bytes).map_err(|_| Failure::new("server_error", "Cannot save authorization state"))?;
        self.db = db;
        Ok(())
    }
    pub fn register(&mut self, input: Value) -> AuthResult<Value> {
        self.register_client(input, false)
    }
    pub fn register_public(&mut self, input: Value) -> AuthResult<Value> {
        self.register_client(input, true)
    }
    fn register_client(&mut self, input: Value, public: bool) -> AuthResult<Value> {
        let redirects: Vec<String> = serde_json::from_value(input.get("redirect_uris").cloned().unwrap_or(Value::Null))
            .map_err(|_| Failure::new("invalid_client_metadata", "redirect_uris must be an array"))?;
        if redirects.is_empty() || redirects.len() > 16 || redirects.iter().any(|r| !allowed_callback(r, self.settings.allow_test_callbacks)) {
            return Err(Failure::new("invalid_client_metadata", "Callback not allowed. Use the exact connection callback shown by your app"));
        }
        let method = input.get("token_endpoint_auth_method").and_then(Value::as_str).unwrap_or("client_secret_basic");
        if !["none", "client_secret_post", "client_secret_basic"].contains(&method) {
            return Err(Failure::new("invalid_client_metadata", "Unsupported token authentication method"));
        }
        if let Some(types) = input.get("grant_types") {
            let Some(types) = types.as_array() else { return Err(Failure::new("invalid_client_metadata", "Invalid grant types")) };
            if types.iter().any(|t| !matches!(t.as_str(), Some("authorization_code" | "refresh_token"))) {
                return Err(Failure::new("invalid_client_metadata", "Unsupported grant type"));
            }
        }
        if let Some(types) = input.get("response_types")
            && types != &json!(["code"])
        {
            return Err(Failure::new("invalid_client_metadata", "Only code responses are supported"));
        }
        let name = input.get("client_name").and_then(Value::as_str).unwrap_or("Remote design client");
        if name.len() > 120 {
            return Err(Failure::new("invalid_client_metadata", "Client name is too long"));
        }
        Self::prune_db(&mut self.db);
        let count = self.db.clients.values().filter(|c| c.registration_expires.is_some() == public).count();
        if count >= if public { 32 } else { 128 } {
            return Err(Failure::new(
                "temporarily_unavailable",
                if public {
                    "Public registration is busy. Try again in ten minutes or create a client in Preferences > Remote AI"
                } else {
                    "Client limit reached; remove unused registrations locally"
                },
            ));
        }
        let client_secret = (method != "none").then(secret);
        let client = Client {
            client_id: secret(),
            client_name: name.into(),
            redirect_uris: redirects,
            token_endpoint_auth_method: method.into(),
            client_id_issued_at: now(),
            secret_hash: client_secret.as_deref().map(digest),
            registration_expires: public.then(|| now() + 600),
        };
        let mut db = self.db.clone();
        db.clients.insert(client.client_id.clone(), client.clone());
        self.commit(db)?;
        let mut result = json!({"client_id":client.client_id,"client_name":client.client_name,"redirect_uris":client.redirect_uris,
            "token_endpoint_auth_method":method,"client_id_issued_at":client.client_id_issued_at,
            "grant_types":["authorization_code","refresh_token"],"response_types":["code"],"scope":SCOPE});
        if let Some(value) = client_secret {
            result["client_secret"] = json!(value);
            result["client_secret_expires_at"] = json!(0);
        }
        Ok(result)
    }
    fn target(&self, parameters: &Fields) -> AuthResult<()> {
        if parameters.get("resource").is_some_and(|r| r.trim_end_matches('/') != self.settings.resource()) {
            return Err(Failure::new("invalid_target", "Token belongs to a different MCP resource"));
        }
        if parameters.get("scope").is_some_and(|s| s.split_whitespace().any(|v| v != SCOPE)) {
            return Err(Failure::new("invalid_scope", "Unknown scope"));
        }
        Ok(())
    }
    pub fn authorize(&mut self, parameters: &Fields) -> AuthResult<(String, String, Pending)> {
        self.target(parameters)?;
        if parameters.get("response_type").map(String::as_str) != Some("code") {
            return Err(Failure::new("unsupported_response_type", "Use the code flow"));
        }
        let client = self
            .db
            .clients
            .get(parameters.get("client_id").map(String::as_str).unwrap_or(""))
            .filter(|c| c.registration_expires.is_none_or(|expiry| expiry > now()))
            .ok_or_else(Failure::unauthorized)?;
        let redirect = parameters
            .get("redirect_uri")
            .filter(|r| client.redirect_uris.contains(r))
            .ok_or_else(|| Failure::new("invalid_request", "Callback does not match the registered client"))?;
        let challenge = parameters
            .get("code_challenge")
            .filter(|v| v.len() == 43 && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
            .ok_or_else(|| Failure::new("invalid_request", "S256 PKCE is required"))?;
        if parameters.get("code_challenge_method").map(String::as_str) != Some("S256") {
            return Err(Failure::new("invalid_request", "S256 PKCE is required"));
        }
        self.pending.retain(|_, p| {
            p.expires > now() && self.db.clients.get(&p.client_id).is_some_and(|c| c.registration_expires.is_none_or(|expiry| expiry > now()))
        });
        self.attempts.retain(|id, _| self.pending.contains_key(id));
        self.codes.retain(|_, c| c.grant.expires > now());
        let public = client.registration_expires.is_some();
        let pending_class =
            self.pending.values().filter(|p| self.db.clients.get(&p.client_id).is_some_and(|c| c.registration_expires.is_some() == public)).count();
        if self.pending.values().filter(|p| p.client_id == client.client_id).count() >= 8
            || pending_class >= if public { 32 } else { 64 }
            || self.codes.len() >= 128
        {
            return Err(Failure::new("temporarily_unavailable", "Too many pending sign-ins"));
        }
        let id = secret();
        let cookie = secret();
        let pending = Pending {
            client_id: client.client_id.clone(),
            client_name: client.client_name.clone(),
            redirect: redirect.clone(),
            state: parameters.get("state").cloned(),
            challenge: challenge.clone(),
            cookie_hash: digest(&cookie),
            expires: now() + 600,
        };
        self.pending.insert(digest(&id), pending.clone());
        Ok((id, cookie, pending))
    }
    pub fn pending(&self, id: &str, cookie: &str) -> AuthResult<Pending> {
        self.pending
            .get(&digest(id))
            .filter(|p| {
                p.expires > now()
                    && equal(&digest(cookie), &p.cookie_hash)
                    && self.db.clients.get(&p.client_id).is_some_and(|c| c.registration_expires.is_none_or(|expiry| expiry > now()))
            })
            .cloned()
            .ok_or_else(|| Failure::new("invalid_request", "Sign-in expired or invalid. Start again from the app connecting to VectorCraft"))
    }
    pub fn approve(&mut self, id: &str, cookie: &str, allow: bool) -> AuthResult<String> {
        let pending = self.pending(id, cookie)?;
        if !self.db.clients.get(&pending.client_id).is_some_and(|c| c.registration_expires.is_none_or(|expiry| expiry > now())) {
            return Err(Failure::unauthorized());
        }
        if allow && self.db.clients.get(&pending.client_id).is_some_and(|c| c.registration_expires.is_some()) {
            if self.db.clients.values().filter(|c| c.registration_expires.is_none()).count() >= 128 {
                return Err(Failure::new("temporarily_unavailable", "Remove an unused saved client locally before approving another"));
            }
            let mut db = self.db.clone();
            let client = db.clients.get_mut(&pending.client_id).ok_or_else(Failure::unauthorized)?;
            client.registration_expires = None;
            self.commit(db)?;
        }
        let mut destination = Url::parse(&pending.redirect).map_err(|_| Failure::new("server_error", "Invalid saved callback"))?;
        {
            let mut query = destination.query_pairs_mut();
            if let Some(state) = pending.state {
                query.append_pair("state", &state);
            }
            query.append_pair("iss", &self.settings.issuer());
            if allow {
                let code = secret();
                query.append_pair("code", &code);
                self.codes.insert(
                    digest(&code),
                    Code {
                        grant: Grant {
                            client_id: pending.client_id,
                            resource: self.settings.resource(),
                            family: secret(),
                            expires: now() + 120,
                            used: false,
                        },
                        redirect: pending.redirect,
                        challenge: pending.challenge,
                    },
                );
            } else {
                query.append_pair("error", "access_denied");
            }
        }
        self.pending.remove(&digest(id));
        self.attempts.remove(&digest(id));
        Ok(destination.to_string())
    }
    /// A retry budget belongs to a validated browser sign-in, never to an anonymous route.
    pub fn password_attempt(&mut self, id: &str, cookie: &str) -> AuthResult<()> {
        self.pending(id, cookie)?;
        let attempts = self.attempts.entry(digest(id)).or_default();
        if *attempts >= 10 {
            return Err(Failure {
                status: 429,
                code: "slow_down",
                message: "Too many attempts for this sign-in. Start a new sign-in from your AI app".into(),
            });
        }
        *attempts += 1;
        Ok(())
    }
    pub fn authenticate(&self, parameters: &Fields, basic: Option<(&str, &str)>) -> AuthResult<Client> {
        let id = basic.map(|v| v.0).or_else(|| parameters.get("client_id").map(String::as_str)).unwrap_or("");
        let client =
            self.db.clients.get(id).filter(|c| c.registration_expires.is_none_or(|expiry| expiry > now())).ok_or_else(Failure::unauthorized)?;
        if basic.is_some() && (parameters.get("client_id").is_some_and(|value| value != id) || parameters.contains_key("client_secret")) {
            return Err(Failure::unauthorized());
        }
        match client.token_endpoint_auth_method.as_str() {
            "none" if basic.is_none() && !parameters.contains_key("client_secret") => {}
            "client_secret_post" if basic.is_none() => {
                let supplied = parameters.get("client_secret").map(|s| digest(s)).unwrap_or_default();
                if !equal(&supplied, client.secret_hash.as_deref().unwrap_or("!")) {
                    return Err(Failure::unauthorized());
                }
            }
            "client_secret_basic" if basic.is_some() => {
                let supplied = basic.map(|v| digest(v.1)).unwrap_or_default();
                if !equal(&supplied, client.secret_hash.as_deref().unwrap_or("!")) {
                    return Err(Failure::unauthorized());
                }
            }
            _ => return Err(Failure::unauthorized()),
        }
        Ok(client.clone())
    }
    fn issue(&mut self, grant: &Grant, mut db: Database) -> AuthResult<Value> {
        let access = secret();
        let refresh = secret();
        let mut a = grant.clone();
        a.expires = now() + 3600;
        a.used = false;
        let mut r = grant.clone();
        r.expires = now() + 14 * 86400;
        r.used = false;
        db.access.insert(digest(&access), a);
        db.refresh.insert(digest(&refresh), r);
        self.commit(db)?;
        Ok(json!({"access_token":access,"refresh_token":refresh,"token_type":"Bearer","expires_in":3600,"scope":SCOPE}))
    }
    pub fn token(&mut self, parameters: &Fields, client: &Client) -> AuthResult<Value> {
        self.target(parameters)?;
        match parameters.get("grant_type").map(String::as_str) {
            Some("authorization_code") => {
                let key = digest(parameters.get("code").map(String::as_str).unwrap_or(""));
                let code = self.codes.get(&key).ok_or_else(|| Failure::new("invalid_grant", "Code invalid or expired"))?;
                let verifier = parameters.get("code_verifier").map(String::as_str).unwrap_or("");
                if code.grant.expires <= now()
                    || code.grant.client_id != client.client_id
                    || parameters.get("redirect_uri") != Some(&code.redirect)
                    || !(43..=128).contains(&verifier.len())
                    || !verifier.bytes().all(|b| b.is_ascii_alphanumeric() || b"-._~".contains(&b))
                    || !equal(&digest(verifier), &code.challenge)
                {
                    return Err(Failure::new("invalid_grant", "Code, callback or PKCE verifier is invalid"));
                }
                let grant = code.grant.clone();
                let result = self.issue(&grant, self.db.clone())?;
                self.codes.remove(&key);
                Ok(result)
            }
            Some("refresh_token") => {
                let key = digest(parameters.get("refresh_token").map(String::as_str).unwrap_or(""));
                let grant = self
                    .db
                    .refresh
                    .get(&key)
                    .filter(|g| g.expires > now() && g.client_id == client.client_id)
                    .cloned()
                    .ok_or_else(|| Failure::new("invalid_grant", "Refresh token invalid or expired"))?;
                if grant.used {
                    let mut db = self.db.clone();
                    Self::remove_family(&mut db, &grant.family);
                    self.commit(db)?;
                    return Err(Failure::new("invalid_grant", "Refresh token reused; this connection was revoked"));
                }
                let mut db = self.db.clone();
                if let Some(old) = db.refresh.get_mut(&key) {
                    old.used = true;
                }
                self.issue(&grant, db)
            }
            _ => Err(Failure::new("unsupported_grant_type", "Use authorization_code or refresh_token")),
        }
    }
    pub fn valid_access(&self, token: &str) -> bool {
        token.len() <= 512
            && self
                .db
                .access
                .get(&digest(token))
                .is_some_and(|g| g.expires > now() && g.resource == self.settings.resource() && self.db.clients.contains_key(&g.client_id))
    }
    fn remove_family(db: &mut Database, family: &str) {
        db.access.retain(|_, g| g.family != family);
        db.refresh.retain(|_, g| g.family != family);
    }
    pub fn revoke(&mut self, token: &str, client: &Client) -> AuthResult<()> {
        let key = digest(token);
        if let Some(grant) = self.db.access.get(&key).or_else(|| self.db.refresh.get(&key)).filter(|g| g.client_id == client.client_id).cloned() {
            let mut db = self.db.clone();
            Self::remove_family(&mut db, &grant.family);
            self.commit(db)?;
        }
        Ok(())
    }
    pub fn clients(&self) -> Value {
        json!(
            self.db
                .clients
                .values()
                .filter(|c| c.registration_expires.is_none_or(|expiry| expiry > now()))
                .map(|c| json!({"id":c.client_id,"name":c.client_name,"callbacks":c.redirect_uris,
        "connections":self.db.refresh.values().filter(|g|g.client_id==c.client_id&&!g.used&&g.expires>now()).count()}))
                .collect::<Vec<_>>()
        )
    }
    pub fn remove_client(&mut self, id: &str) -> AuthResult<()> {
        let mut db = self.db.clone();
        db.clients.remove(id);
        db.access.retain(|_, g| g.client_id != id);
        db.refresh.retain(|_, g| g.client_id != id);
        self.commit(db)?;
        self.pending.retain(|_, p| p.client_id != id);
        self.attempts.retain(|id, _| self.pending.contains_key(id));
        self.codes.retain(|_, c| c.grant.client_id != id);
        Ok(())
    }
    pub fn revoke_all(&mut self) -> AuthResult<()> {
        self.commit(Database { resource: self.settings.resource(), ..Default::default() })?;
        self.pending.clear();
        self.attempts.clear();
        self.codes.clear();
        Ok(())
    }
}
