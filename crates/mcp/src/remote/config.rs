use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use url::Url;

#[cfg(windows)]
mod windows_permissions;

pub fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}
pub fn secret() -> String {
    let bytes: [u8; 32] = rand::random();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
pub fn digest(value: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(value.as_bytes()))
}
pub fn hash_phrase(phrase: &str) -> Result<String, String> {
    if phrase.len() > 1024 || !(12..=256).contains(&phrase.chars().count()) {
        return Err("Choose an owner passphrase of 12 to 256 characters".into());
    }
    let bytes: [u8; 16] = rand::random();
    let salt = argon2::password_hash::SaltString::encode_b64(&bytes).map_err(|e| e.to_string())?;
    Argon2::default().hash_password(phrase.as_bytes(), &salt).map(|h| h.to_string()).map_err(|e| e.to_string())
}
pub fn check_phrase(phrase: &str, hash: &str) -> bool {
    phrase.len() <= 1024
        && phrase.chars().count() <= 256
        && PasswordHash::new(hash).is_ok_and(|h| Argon2::default().verify_password(phrase.as_bytes(), &h).is_ok())
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub version: u32,
    pub enabled: bool,
    pub public_origin: String,
    pub base_path: String,
    pub bind: String,
    pub control_port: u16,
    pub app_path: PathBuf,
    pub profile_dir: PathBuf,
    pub artwork_root: PathBuf,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_artwork_roots: Vec<PathBuf>,
    pub owner_hash: String,
    pub management_token: String,
    #[serde(default)]
    pub allow_test_callbacks: bool,
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        let origin = Url::parse(&self.public_origin).map_err(|_| "Public address is not a URL")?;
        let loopback = matches!(origin.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
        if (origin.scheme() != "https" && !(origin.scheme() == "http" && loopback))
            || origin.path() != "/"
            || origin.query().is_some()
            || origin.fragment().is_some()
            || !origin.username().is_empty()
            || origin.password().is_some()
        {
            return Err("Use a public HTTPS origin without a path, query or credentials".into());
        }
        if self.base_path.len() > 80
            || !self.base_path.starts_with('/')
            || self.base_path.len() < 2
            || !self.base_path.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'/' || b == b'-' || b == b'_')
            || self.base_path.ends_with('/')
            || self.base_path.contains("//")
        {
            return Err("Invalid endpoint path".into());
        }
        let address: std::net::SocketAddr = self.bind.parse().map_err(|_| "Invalid listener address")?;
        if address.port() == 0 || self.control_port == 0 || address.port() == self.control_port {
            return Err("Choose distinct nonzero HTTP and control ports".into());
        }
        if self.version != 1
            || !self.app_path.is_absolute()
            || !self.app_path.is_file()
            || !self.profile_dir.is_absolute()
            || !self.artwork_root.is_absolute()
            || !self.artwork_root.is_dir()
        {
            return Err("Invalid executable, profile or artwork folder".into());
        }
        if self.management_token.len() < 40 || PasswordHash::new(&self.owner_hash).is_err() {
            return Err("Invalid private authorization configuration".into());
        }
        if self.allow_test_callbacks && !loopback {
            return Err("Test callbacks require a loopback public origin".into());
        }
        let folders = self.artwork_roots();
        if folders.is_empty() || folders.len() > 32 {
            return Err("Choose between 1 and 32 artwork folders".into());
        }
        let private = self.profile_dir.join("Remote Access");
        // The private directory is created before a configuration is saved. An absent one is
        // still compared by the real parent so new configurations can be validated safely.
        let private = if private.exists() { private.canonicalize() } else { self.profile_dir.canonicalize().map(|p| p.join("Remote Access")) }
            .map_err(|e| e.to_string())?;
        for folder in folders {
            if !folder.is_absolute() || !folder.is_dir() {
                return Err("Every artwork folder must be an existing absolute directory".into());
            }
            let folder = folder.canonicalize().map_err(|e| e.to_string())?;
            let within_private = if cfg!(windows) {
                folder.to_string_lossy().to_lowercase().starts_with(&(private.to_string_lossy().to_lowercase() + "\\"))
                    || folder.to_string_lossy().eq_ignore_ascii_case(&private.to_string_lossy())
            } else {
                folder.starts_with(&private)
            };
            if within_private {
                return Err("Private connection settings cannot be shared as artwork".into());
            }
        }
        Ok(())
    }
    pub fn artwork_roots(&self) -> Vec<PathBuf> {
        std::iter::once(self.artwork_root.clone()).chain(self.additional_artwork_roots.iter().cloned()).collect()
    }
    pub fn automation_roots(&self) -> Result<Option<std::sync::Arc<vectorcraft_engine::file_access::AutomationRoots>>, String> {
        let roots = self.artwork_roots();
        vectorcraft_engine::file_access::AutomationRoots::new_many(&roots, &roots, &[self.profile_dir.join("Remote Access")])
    }
    pub fn origin(&self) -> Result<Url, String> {
        Url::parse(&self.public_origin).map_err(|e| e.to_string())
    }
    pub fn issuer(&self) -> String {
        format!("{}{}", self.public_origin.trim_end_matches('/'), self.base_path)
    }
    pub fn resource(&self) -> String {
        format!("{}/mcp", self.issuer())
    }
    pub fn directory(path: &Path) -> Result<&Path, String> {
        path.parent().ok_or_else(|| "Settings need a parent folder".into())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let result = Self::read_for_owner_repair(path)?;
        result.validate()?;
        result.validate_storage(path)?;
        Ok(result)
    }
    /// Decode saved settings so the local owner can repair an unavailable folder. These settings
    /// MUST still be validated before starting a service or installing automation authority.
    pub fn read_for_owner_repair(path: &Path) -> Result<Self, String> {
        let metadata = std::fs::metadata(path).map_err(|e| format!("Cannot read remote settings: {e}"))?;
        if metadata.len() > 64 * 1024 {
            return Err("Remote settings are too large".into());
        }
        serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?).map_err(|e| format!("Invalid remote settings: {e}"))
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if bytes.len() > 64 * 1024 {
            return Err("Remote settings are too large".into());
        }
        protect_directory(&self.profile_dir.join("Remote Access"))?;
        self.validate_storage(path)?;
        write_private(path, &bytes)
    }
    fn validate_storage(&self, path: &Path) -> Result<(), String> {
        let private = self.profile_dir.join("Remote Access").canonicalize().map_err(|e| e.to_string())?;
        let directory = Self::directory(path)?.canonicalize().map_err(|e| e.to_string())?;
        if directory != private {
            return Err("Remote settings must stay in this profile's private Remote Access folder".into());
        }
        reject_link(path)
    }
}

fn reject_link(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err("Private authorization files cannot be symbolic links".into()),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
pub fn write_private(path: &Path, bytes: &[u8]) -> Result<(), String> {
    reject_link(path)?;
    vectorcraft_format::write_atomic(path, bytes).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    }
    Ok(())
}
pub fn protect_directory(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        let system = std::env::var_os("SystemRoot").map(PathBuf::from).ok_or("Windows system folder is unavailable")?.join("System32");
        let identity = std::process::Command::new(system.join("whoami.exe"))
            .args(["/user", "/fo", "csv", "/nh"])
            .creation_flags(0x08000000)
            .output()
            .map_err(|e| e.to_string())?;
        if !identity.status.success() {
            return Err("Cannot identify the Windows owner".into());
        }
        let text = String::from_utf8_lossy(&identity.stdout);
        let sid = text.trim().rsplit(',').next().unwrap_or("").trim_matches('"');
        if !sid.starts_with("S-1-") || !sid.bytes().all(|b| b.is_ascii_digit() || b == b'S' || b == b'-') {
            return Err("Cannot identify the Windows owner SID".into());
        }
        windows_permissions::protect(path, sid, true)?;
        // Existing files can have explicit grants even after their parent stops inheriting.
        // Harden the service's flat private store as well; never follow a credential symlink.
        for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            reject_link(&entry.path())?;
            if entry.file_type().map_err(|e| e.to_string())?.is_file() {
                windows_permissions::protect(&entry.path(), sid, false)?;
            }
        }
    }
    Ok(())
}
