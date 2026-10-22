//! Linux file reads must not consume pipes/devices or allocate without a byte limit.

use rustix::fs::{Mode, OFlags};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Match the native format's 512 MiB unpacked limit for desktop input bytes.
const MAX_BYTES: u64 = 512 << 20;

/// Read a bounded regular file, following ordinary document symlinks.
pub fn read(path: &str) -> Result<Vec<u8>, String> {
    read_capped(Path::new(path), MAX_BYTES)
}

/// Check before opening to avoid device opens; check the descriptor again to close path races.
fn read_capped(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    read_open_file(open_regular_file(path)?, max)
}

/// Open without blocking on path replacements and validate the actual descriptor.
fn open_regular_file(path: &Path) -> Result<File, String> {
    if !std::fs::metadata(path).map_err(|e| e.to_string())?.is_file() {
        return Err("not a file".into());
    }
    // NONBLOCK prevents a regular-file-to-FIFO replacement from blocking open; NOCTTY avoids
    // acquiring a controlling terminal if a path is replaced with a terminal device.
    let fd =
        rustix::fs::open(path, OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOCTTY | OFlags::CLOEXEC, Mode::empty()).map_err(|e| e.to_string())?;
    let file = File::from(fd);
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("not a file".into());
    }
    Ok(file)
}

/// Read a bounded prefix for script and shortcut inspection, preserving truncation semantics.
pub fn read_head(path: &str, max: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    open_regular_file(Path::new(path))?.take(max as u64).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}

/// Validate the actual opened object and cap reads even when it grows or reports length zero.
fn read_open_file(file: File, max: u64) -> Result<Vec<u8>, String> {
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("not a file".into());
    }
    // Keep the existing localized engine diagnostics for invalid or oversized inputs.
    let too_large = || "the drawing is too large to open".to_string();
    if meta.len() > max {
        return Err(too_large());
    }
    let mut bytes = Vec::new();
    file.take(max.saturating_add(1)).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > max {
        return Err(too_large());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    /// A private fixture directory, removed after each test without touching user files.
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        /// Allocate a test-specific directory under the OS temporary folder.
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("vectorcraft-read-{}-{name}", std::process::id()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Fixture {
        /// Remove only the files created inside this fixture directory.
        fn drop(&mut self) {
            for entry in std::fs::read_dir(&self.0).unwrap().flatten() {
                let _ = std::fs::remove_file(entry.path());
            }
            let _ = std::fs::remove_dir(&self.0);
        }
    }

    /// Reject devices directly and through ordinary filesystem links.
    #[test]
    fn desktop_service_rejects_devices_and_directories() {
        let dir = Fixture::new("devices");
        let link = dir.0.join("device.svg");
        symlink("/dev/zero", &link).unwrap();
        for path in [Path::new("/dev/null"), Path::new("/dev/zero"), &dir.0, &link] {
            assert!(read(path.to_str().unwrap()).unwrap_err().contains("not a file"));
            assert!(read_head(path.to_str().unwrap(), 16).unwrap_err().contains("not a file"));
        }
    }

    /// FIFOs and a descriptor swapped after the path check cannot enter a blocking read.
    #[test]
    fn fifo_and_opened_special_descriptor_are_rejected() {
        let dir = Fixture::new("fifo");
        let path = dir.0.join("pipe.svg");
        rustix::fs::mkfifoat(rustix::fs::CWD, &path, Mode::RUSR | Mode::WUSR).unwrap();
        assert!(read(path.to_str().unwrap()).unwrap_err().contains("not a file"));
        let fd = rustix::fs::open(&path, OFlags::RDONLY | OFlags::NONBLOCK, Mode::empty()).unwrap();
        assert!(read_open_file(File::from(fd), 16).unwrap_err().contains("not a file"));
    }

    /// Existing-canvas placement reaches the same bounded reader before decoding or mutation.
    #[test]
    fn placement_rejects_special_and_oversized_paths() {
        let dir = Fixture::new("place");
        let fifo = dir.0.join("pipe.png");
        rustix::fs::mkfifoat(rustix::fs::CWD, &fifo, Mode::RUSR | Mode::WUSR).unwrap();
        let device = dir.0.join("device.png");
        symlink("/dev/zero", &device).unwrap();
        let large = dir.0.join("large.png");
        File::create(&large).unwrap().set_len(MAX_BYTES + 1).unwrap();
        let mut session = crate::Session::new();
        session.execute("file.new", &serde_json::json!({"width": 400, "height": 300})).unwrap();
        for path in [&fifo, &device, &large] {
            let error = session.execute("file.place", &serde_json::json!({"path": path})).unwrap_err().to_string();
            assert!(error.contains("not a file") || error.contains("too large"), "{error}");
        }
    }

    /// Normal symlinks still work; sparse oversized files are refused before allocation.
    #[test]
    fn regular_files_symlinks_and_byte_limits() {
        let dir = Fixture::new("regular");
        let path = dir.0.join("日本語.svg");
        std::fs::write(&path, b"test").unwrap();
        let link = dir.0.join("linked.svg");
        symlink(&path, &link).unwrap();
        assert_eq!(read(link.to_str().unwrap()).unwrap(), b"test");
        assert_eq!(read_capped(&path, 4).unwrap(), b"test");
        assert_eq!(read_head(path.to_str().unwrap(), 2).unwrap(), b"te");
        assert!(read_capped(&path, 3).unwrap_err().contains("too large"));
        File::options().write(true).open(&path).unwrap().set_len(MAX_BYTES + 1).unwrap();
        assert!(read(path.to_str().unwrap()).unwrap_err().contains("too large"));
        // procfs reports zero length: the streaming cap must still apply.
        assert!(read_capped(Path::new("/proc/self/status"), 16).unwrap_err().contains("too large"));
    }
}
