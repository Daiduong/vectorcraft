//! Linux file-manager launches hand their paths to the running desktop app over the session bus.
//! The bus owns the name atomically and releases it when the process exits; no stale lock files.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

const NAME: &str = "ai.storyteller.vectorcraft.OpenDocuments";
const PATH: &str = "/ai/storyteller/vectorcraft/OpenDocuments";

#[derive(Default)]
struct Pending {
    files: Vec<String>,
    activate: bool,
    ctx: Option<egui::Context>,
}

#[derive(Clone, Default)]
struct OpenDocuments(Arc<Mutex<Pending>>);

#[zbus::interface(name = "ai.storyteller.vectorcraft.OpenDocuments")]
impl OpenDocuments {
    /// Validate and queue absolute file paths, then wake the UI to dispatch them.
    fn open(&self, files: Vec<String>) -> zbus::fdo::Result<()> {
        let mut pending = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        // Bound queued work, including requests that arrive while a file dialog is open.
        if files.len().saturating_add(pending.files.len()) > 256 || files.iter().any(|p| p.len() > 32768 || !std::path::Path::new(p).is_absolute()) {
            return Err(zbus::fdo::Error::InvalidArgs("too many files or invalid absolute paths".into()));
        }
        pending.files.extend(files);
        pending.activate = true;
        if let Some(ctx) = &pending.ctx {
            ctx.request_repaint();
        }
        Ok(())
    }
}

/// Keep the connection alive for the lifetime of the window.
pub struct Instance {
    _connection: zbus::blocking::Connection,
    documents: OpenDocuments,
}

/// Whether this process owns the desktop window or handed its request to the owner.
pub enum Launch {
    Primary(Instance),
    Forwarded,
}

/// Claim the session-bus name or forward this launch to its existing owner.
pub fn start(files: &[String]) -> Result<Launch, String> {
    let documents = OpenDocuments::default();
    let builder = zbus::blocking::connection::Builder::session().map_err(|e| e.to_string())?;
    start_on(builder, files, documents)
}

/// Start on a supplied connection builder so transport tests can isolate their bus.
fn start_on(builder: zbus::blocking::connection::Builder<'_>, files: &[String], documents: OpenDocuments) -> Result<Launch, String> {
    let connection = builder
        .method_timeout(Duration::from_secs(5))
        .serve_at(PATH, documents.clone())
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;
    match connection.request_name_with_flags(NAME, zbus::fdo::RequestNameFlags::DoNotQueue.into()) {
        Ok(_) => Ok(Launch::Primary(Instance { _connection: connection, documents })),
        Err(zbus::Error::NameTaken) => {
            // Relative paths belong to the launching file manager's working directory.
            let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
            let paths: Vec<String> = files.iter().map(|p| cwd.join(p).to_string_lossy().into_owned()).collect();
            let proxy = zbus::blocking::Proxy::new(&connection, NAME, PATH, NAME).map_err(|e| e.to_string())?;
            let _: () = proxy.call("Open", &(paths,)).map_err(|e| e.to_string())?;
            Ok(Launch::Forwarded)
        }
        Err(e) => Err(e.to_string()),
    }
}

impl Instance {
    /// Register the repaint context after the primary window has been created.
    pub fn set_ui(&self, ctx: &egui::Context) {
        self.documents.0.lock().unwrap_or_else(PoisonError::into_inner).ctx = Some(ctx.clone());
    }

    /// File-open requests must not change the document a dialog is acting on.
    pub fn dispatch(&self, app: &mut vectorcraft_ui_egui::VectorcraftApp, ctx: &egui::Context) {
        if !app.file_dialog_open()
            && app.ui.dialog.is_none()
            && let Some(files) = self.take()
        {
            for file in files {
                // IPC grants document opening only. File Open also installs plug-ins and
                // imports settings/libraries; those actions require an explicit local choice.
                if vectorcraft_ui_egui::io::opens_as(&file).is_some() {
                    vectorcraft_ui_egui::io::report_open_error(
                        app,
                        &file,
                        "Forwarded requests can only open documents. Open this file from within VectorCraft instead.",
                    );
                } else {
                    super::open_files(app, vec![file]);
                }
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
    }

    /// Drain an activation request once; an empty file list still requests focus.
    fn take(&self) -> Option<Vec<String>> {
        let mut pending = self.documents.0.lock().unwrap_or_else(PoisonError::into_inner);
        if !std::mem::take(&mut pending.activate) {
            return None;
        }
        Some(std::mem::take(&mut pending.files))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Invalid requests leave previously queued paths intact.
    #[test]
    fn queues_files_until_taken_and_rejects_invalid_requests() {
        let documents = OpenDocuments::default();
        documents.open(vec!["/tmp/a b.svg".into(), "/tmp/日本語.svg".into()]).unwrap();
        assert!(documents.open(vec!["relative.svg".into()]).is_err());
        assert!(documents.open(vec!["/tmp/a.svg".into(); 256]).is_err());
        let pending = documents.0.lock().unwrap();
        assert_eq!(pending.files, ["/tmp/a b.svg", "/tmp/日本語.svg"]);
        assert!(pending.activate);
    }

    /// Exercise real forwarding, dialog deferral, tab opening and ownership cleanup.
    #[test]
    fn second_launch_forwards_and_name_is_released_on_exit() {
        use std::io::BufRead as _;
        let Ok(mut daemon) = std::process::Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address=1"])
            .stdout(std::process::Stdio::piped())
            .spawn()
        else {
            eprintln!("dbus-daemon unavailable; skipping the private-bus transport regression");
            return;
        };
        struct Stop(std::process::Child);
        impl Drop for Stop {
            /// Stop the private test bus even when an assertion unwinds.
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let mut address = String::new();
        std::io::BufReader::new(daemon.stdout.take().unwrap()).read_line(&mut address).unwrap();
        let _stop = Stop(daemon);
        let builder = || zbus::blocking::connection::Builder::address(address.trim()).unwrap();
        let Launch::Primary(first) = start_on(builder(), &[], OpenDocuments::default()).unwrap() else { panic!("first launch") };
        first.set_ui(&egui::Context::default());
        assert!(matches!(
            start_on(builder(), &["a b.svg".into(), "/tmp/日本語.svg".into(), "/tmp/external.WASM".into()], OpenDocuments::default()).unwrap(),
            Launch::Forwarded
        ));
        let files = first.documents.0.lock().unwrap().files.clone();
        assert_eq!(
            files,
            [std::env::current_dir().unwrap().join("a b.svg").to_string_lossy(), "/tmp/日本語.svg".into(), "/tmp/external.WASM".into()]
        );
        // Exercise the actual desktop file-opening path with an existing unsaved document.
        let special_reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let reads = special_reads.clone();
        let services = vectorcraft_ui_egui::Services {
            read: Some(Box::new(move |path| {
                if vectorcraft_ui_egui::io::opens_as(path).is_some() {
                    reads.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                if vectorcraft_ui_egui::io::opens_as(path) == Some(vectorcraft_ui_egui::io::OpensAs::Plugin) {
                    return Ok(include_bytes!("../../../crates/plugins/tests/fixtures/desaturate.wasm").to_vec());
                }
                Ok(br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><rect width="10" height="10"/></svg>"#.to_vec())
            })),
            ..Default::default()
        };
        let mut app = vectorcraft_ui_egui::VectorcraftApp::new(vectorcraft_engine::Session::new(), services);
        let plugin = serde_json::json!({"id": "org.vectorcraft.example.desaturate"});
        assert!(app.run("plugin.info", plugin.clone()).is_err());
        app.run("file.new", serde_json::json!({"width": 100, "height": 100})).unwrap();
        app.run("shape.rectangle", serde_json::json!({"x": 1, "y": 1, "width": 5, "height": 5})).unwrap();
        let original = app.session.documents()[0].doc.clone();
        let ctx = egui::Context::default();
        app.run("file.newDialog", serde_json::Value::Null).unwrap();
        assert!(app.ui.dialog.is_some());
        first.dispatch(&mut app, &ctx);
        assert_eq!(app.session.documents().len(), 1, "dialog keeps its document active");
        assert_eq!(first.documents.0.lock().unwrap().files, files, "queued files wait for the dialog");
        app.ui.dialog = None;
        first.dispatch(&mut app, &ctx);
        assert!(app.run("plugin.info", plugin.clone()).is_err(), "forwarded files cannot install plug-ins in the running editor");
        assert_eq!(special_reads.load(std::sync::atomic::Ordering::Relaxed), 0, "reject before reading the module");
        assert!(app.ui.dialog.as_ref().unwrap().str("detail").contains("only open documents"));
        app.ui.dialog = None; // The user closes the refusal before queued documents finish.
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while !app.opening.pending.is_empty() && std::time::Instant::now() < deadline {
            vectorcraft_ui_egui::opening::poll(&mut app);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(app.opening.pending.is_empty(), "forwarded files finished opening");
        assert_eq!(app.session.documents().len(), 3, "both files become tabs in the same app");
        assert_eq!(app.session.documents()[0].doc, original, "unsaved artwork is preserved");
        assert!(app.session.documents()[0].is_dirty());
        assert_ne!(app.session.active_index(), Some(0));
        assert!(first.take().is_none());
        // Deliberate local installation still works, and IPC cannot replace that plug-in.
        vectorcraft_ui_egui::io::open_path(&mut app, "/tmp/manual.wasm").unwrap();
        let installed = app.run("plugin.info", plugin.clone()).unwrap();
        assert!(matches!(start_on(builder(), &["/tmp/external.WASM".into()], OpenDocuments::default()).unwrap(), Launch::Forwarded));
        first.dispatch(&mut app, &ctx);
        assert_eq!(app.run("plugin.info", plugin).unwrap(), installed, "forwarding must not replace an installed plug-in");
        assert_eq!(special_reads.load(std::sync::atomic::Ordering::Relaxed), 1, "only the explicit local install reads the module");
        app.ui.dialog = None;
        assert!(matches!(start_on(builder(), &[], OpenDocuments::default()).unwrap(), Launch::Forwarded));
        assert_eq!(first.take(), Some(vec![]));
        drop(first);
        assert!(matches!(start_on(builder(), &[], OpenDocuments::default()).unwrap(), Launch::Primary(_)));
    }
}
