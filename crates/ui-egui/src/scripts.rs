//! File › Scripts › Other Script…: run a JavaScript file over the commands
//! ([`vectorcraft_script`]): one undo step per document it changes; a failure changes nothing and
//! is shown in a dialog with the script's log.

use serde_json::Value;

use crate::{FilePick, VectorcraftApp, dialogs, io};

/// The files the script picker offers.
pub const EXTS: &[&str] = &["js"];

/// `ui.runScript {path?}`: run the script at `path`, else a picked one (on the web the browser's
/// picker hands it to [`drain`]).
pub fn run(app: &mut VectorcraftApp, path: Option<String>) -> Result<Value, String> {
    let path = match path {
        Some(p) => p,
        None => {
            if let Some(start) = app.services.script_async.as_mut() {
                start();
                return Ok(Value::Null);
            }
            io::pick_open(app, &FilePick { filters: vec![("JavaScript", EXTS)], ..Default::default() })?
        }
    };
    let bytes = io::read(app, &path)?;
    let name = vectorcraft_engine::cmd::fileio::file_name(&path);
    run_bytes(app, &name, &bytes)
}

/// Run the script `name` (its file name) with text `bytes` → its value. Alerts and the log are
/// shown in a dialog after the run; an error in one too.
pub fn run_bytes(app: &mut VectorcraftApp, name: &str, bytes: &[u8]) -> Result<Value, String> {
    let source = match vectorcraft_script::read_source(bytes) {
        Ok(s) => s,
        Err(e) => {
            report(app, name, &format!("{name}: {e}"), &[]);
            return Err(e);
        }
    };
    let r = vectorcraft_script::run_script(&mut app.session, &source, name, &vectorcraft_script::ScriptOptions::default());
    app.sync_views();
    match r {
        Ok(out) => {
            app.status(format!("Ran script {name} ({} commands)", out.commands));
            if !out.alerts.is_empty() || !out.log.is_empty() {
                let detail = [out.alerts.join("\n"), out.log.join("\n")].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n\n");
                let message = crate::i18n::fmt(tl!("Script “{name}” finished."), &[("name", name)]);
                dialogs::confirm::tell(app, &message, &detail);
            }
            Ok(out.value)
        }
        Err(e) => {
            let text = e.to_string();
            report(app, name, &text, &e.log);
            Err(text)
        }
    }
}

fn report(app: &mut VectorcraftApp, name: &str, error: &str, log: &[String]) {
    app.status(format!("Script {name} failed: {error}"));
    let mut detail = error.to_string();
    if !log.is_empty() {
        detail.push_str("\n\n");
        detail.push_str(&log.join("\n"));
    }
    let message = crate::i18n::fmt(tl!("Script “{name}” failed."), &[("name", name)]);
    dialogs::confirm::tell(app, &message, &detail);
}

/// Scripts picked in the browser (web): each runs as it arrives.
pub fn drain(app: &mut VectorcraftApp) {
    let arrived: Vec<(String, Vec<u8>)> =
        app.services.script_inbox.as_ref().map(|q| std::mem::take(&mut *q.lock().unwrap_or_else(|e| e.into_inner()))).unwrap_or_default();
    for (name, bytes) in arrived {
        // A failure is shown in its dialog.
        let _ = run_bytes(app, &name, &bytes);
    }
}
