//! Host storage: a page that serves VectorCraft can offer its own file storage (a cloud drive, a
//! team folder) next to this device, through `window.craftStorage`:
//!
//! ```js
//! window.craftStorage = {
//!   // File › Open: {path, bytes} for a file in the host's storage, "device" for the browser's
//!   // own file picker, or null when the user cancelled.
//!   async chooseOpen(extensions /* ["svg", …] or null */) { … },
//!   // A save or export to download: a path in the host's storage, "device" for a download, or null.
//!   async chooseSave(suggested /* the file name */) { … },
//!   // Writes a file the user saved to a host path; the host reports progress and errors itself.
//!   // The save counts once it resolves; a rejection leaves the document modified. Saves come
//!   // one at a time, in the order the user made them, so each must settle: a host whose storage
//!   // can hang should reject after its own timeout (only the host can tell a slow upload from a
//!   // stuck one).
//!   async write(path, bytes /* Uint8Array */) { … },
//! };
//! ```
//!
//! Host paths have a scheme of their own, such as `cloud:home/art/logo.svg`
//! ([`craft_web_host::is_host_path`]): a document opened from one keeps it, so File › Save
//! writes back there without asking. Without `window.craftStorage` nothing changes.
//!
//! Browsers open a file picker only during a click: when the user picks "This device" in the host's
//! own dialog, `chooseOpen` should resolve with "device" right from that click.
//!
//! The contract and its rules are the same for every Craft app and live in `craft-web-host`; this
//! module connects them to VectorCraft's open and download services.

use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::rc::Rc;

use craft_web_host::is_host_path;
use craft_web_host::storage::{self, OpenChoice, SaveChoice, SaveQueue};
use vectorcraft_ui_egui::Services;

use crate::web::Inbox;

/// What a save to the host came to: whether the file was stored, and the host path the user chose
/// for it in the host's Save dialog.
type Outcome = (bool, Option<String>);

/// A finished save, for [`vectorcraft_ui_egui::io::save_settled`]: (document, name or host path,
/// saved, the host path the user chose).
pub type Settled = (Option<u64>, String, bool, Option<String>);

thread_local! {
    /// Saves to the host, one at a time in the order the user made them (`craft_web_host`).
    static SAVES: SaveQueue<Outcome> = SaveQueue::new();
    /// Each save in [`SAVES`] by its id: the document whose save it is, and its name or host path.
    static QUEUED: RefCell<HashMap<u64, (Option<u64>, String)>> = RefCell::new(HashMap::new());
    /// The latest save of each document (or, for an export, each name): an earlier one that
    /// failed doesn't count once a later one is on its way, which answers for the document itself.
    static LATEST: RefCell<HashMap<(Option<u64>, String), u64>> = RefCell::new(HashMap::new());
}

fn in_flight() -> bool {
    SAVES.with(SaveQueue::busy)
}

/// What tells saves apart for [`LATEST`]: the document, or an export's name.
fn key(doc: Option<u64>, name: &str) -> (Option<u64>, String) {
    (doc, if doc.is_some() { String::new() } else { name.to_string() })
}

/// Queue a save of `name` for `doc`; it runs once the saves before it have finished.
fn enqueue(doc: Option<u64>, name: String, save: impl Future<Output = Outcome> + 'static) {
    let id = SAVES.with(|q| q.push(save));
    LATEST.with(|l| l.borrow_mut().insert(key(doc, &name), id));
    QUEUED.with(|q| q.borrow_mut().insert(id, (doc, name)));
}

/// The saves that finished since the last call, for [`vectorcraft_ui_egui::io::save_settled`]. One
/// that failed or that the user cancelled marks its document modified again.
pub fn take_settled() -> Vec<Settled> {
    SAVES
        .with(SaveQueue::take_done)
        .into_iter()
        .filter_map(|(id, (saved, host_path))| {
            let (doc, name) = QUEUED.with(|q| q.borrow_mut().remove(&id))?;
            let key = key(doc, &name);
            let latest = LATEST.with(|l| {
                let mut l = l.borrow_mut();
                let latest = l.get(&key) == Some(&id);
                if latest {
                    l.remove(&key);
                }
                latest
            });
            // A failed save that a later one replaces leaves the document to that one.
            Some((doc, name, saved || !latest, host_path.filter(|_| saved)))
        })
        .collect()
}

/// Write to the host; true once the host has the file.
async fn write(path: &str, bytes: &[u8]) -> bool {
    storage::write(path, bytes).await.inspect_err(|e| log::error!("vectorcraft-web: craftStorage.write of {path} failed: {e}")).is_ok()
}

/// Wrap the async open and the download in `services` so they offer the host's storage when the
/// page has one: a file picked there arrives in `inbox` under its host path. The device's own
/// picker and download stay one choice away.
pub fn install(services: &mut Services, inbox: Inbox, ctx: egui::Context) {
    {
        let ctx = ctx.clone();
        SAVES.with(|q| q.on_done(move || ctx.request_repaint()));
    }
    if let Some(device_open) = services.open_async.take() {
        let device_open = Rc::new(RefCell::new(device_open));
        services.open_async = Some(Box::new(move || {
            if !storage::available() {
                return (device_open.borrow_mut())();
            }
            let (device_open, inbox, ctx) = (device_open.clone(), inbox.clone(), ctx.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match storage::choose_open(None).await {
                    Ok(OpenChoice::File { path, bytes }) => {
                        inbox.lock().unwrap_or_else(|e| e.into_inner()).push((path, bytes));
                        ctx.request_repaint();
                    }
                    // The user's own choice: the browser's picker.
                    Ok(OpenChoice::Device) => (device_open.borrow_mut())(),
                    Ok(OpenChoice::Cancelled) => {}
                    // A host that failed or answered something else: nothing was chosen.
                    Err(e) => log::error!("vectorcraft-web: {e}"),
                }
            });
        }));
    }
    if let Some(device_download) = services.download.take() {
        services.save_in_flight = Some(Box::new(in_flight));
        let device_download = Rc::new(RefCell::new(device_download));
        services.download = Some(Box::new(move |name: &str, bytes: &[u8]| {
            if !storage::available() {
                return (device_download.borrow_mut())(name, bytes);
            }
            // The save counts as done now; one that doesn't happen marks the document modified again.
            let doc = vectorcraft_ui_egui::io::saving_document();
            let (device_download, owned, bytes) = (device_download.clone(), name.to_string(), bytes.to_vec());
            enqueue(doc, name.to_string(), async move {
                let name = owned;
                if is_host_path(&name) {
                    // File › Save of a document from the host: straight back to its path.
                    return (write(&name, &bytes).await, None);
                }
                // Anything else the app would download (Save As, exports): ask where it goes.
                match storage::choose_save(&name).await {
                    Ok(SaveChoice::Host(path)) => (write(&path, &bytes).await, Some(path)),
                    // The user's own choice: a download to the device.
                    Ok(SaveChoice::Device) => {
                        let downloaded = (device_download.borrow_mut())(&name, &bytes)
                            .inspect_err(|e| log::error!("vectorcraft-web: download of {name} failed: {e}"))
                            .is_ok();
                        (downloaded, None)
                    }
                    Ok(SaveChoice::Cancelled) => (false, None),
                    // A host that failed or answered something else: not saved, so the document
                    // stays modified.
                    Err(e) => {
                        log::error!("vectorcraft-web: {e}");
                        (false, None)
                    }
                }
            });
            Ok(())
        }));
    }
}
