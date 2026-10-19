//! Replace the private-store DACL through built-in Windows PowerShell/.NET, retaining the
//! crate's unsafe-code prohibition. No script files, new runtime installs or credential arguments.
//! https://learn.microsoft.com/dotnet/api/system.security.accesscontrol.objectsecurity.setsecuritydescriptorsddlform
use std::os::windows::process::CommandExt as _;
use std::path::{Path, PathBuf};

pub(super) fn protect(path: &Path, owner: &str, directory: bool) -> Result<(), String> {
    let inheritance = if directory { "OICI" } else { "" };
    // owner is a validated numeric SID from whoami. Pass metadata through environment entries,
    // never interpolate a path or other caller-controlled text into the fixed command.
    let acl = format!("D:P(A;{inheritance};FA;;;{owner})(A;{inheritance};FA;;;SY)");
    let utility = PathBuf::from(std::env::var_os("SystemRoot").ok_or("Windows system folder is unavailable")?)
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let script = r#"$ErrorActionPreference = 'Stop'
if ($env:VECTORCRAFT_PRIVATE_KIND -eq 'directory') {
    $acl = New-Object System.Security.AccessControl.DirectorySecurity
    $acl.SetSecurityDescriptorSddlForm($env:VECTORCRAFT_PRIVATE_DACL, [System.Security.AccessControl.AccessControlSections]::Access)
    [System.IO.Directory]::SetAccessControl($env:VECTORCRAFT_PRIVATE_PATH, $acl)
} else {
    $acl = New-Object System.Security.AccessControl.FileSecurity
    $acl.SetSecurityDescriptorSddlForm($env:VECTORCRAFT_PRIVATE_DACL, [System.Security.AccessControl.AccessControlSections]::Access)
    [System.IO.File]::SetAccessControl($env:VECTORCRAFT_PRIVATE_PATH, $acl)
}"#;
    let status = std::process::Command::new(utility)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("VECTORCRAFT_PRIVATE_KIND", if directory { "directory" } else { "file" })
        .env("VECTORCRAFT_PRIVATE_PATH", path)
        .env("VECTORCRAFT_PRIVATE_DACL", acl)
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| e.to_string())?;
    if !status.status.success() {
        return Err("Cannot replace permissions on the private connection store".into());
    }
    Ok(())
}
