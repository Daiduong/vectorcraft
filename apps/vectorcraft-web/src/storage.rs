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
//! ([`vectorcraft_ui_egui::io::is_host_path`]): a document opened from one keeps it, so File › Save
//! writes back there without asking. Without `window.craftStorage` nothing changes.
//!
//! Browsers open a file picker only during a click: when the user picks "This device" in the host's
//! own dialog, `chooseOpen` should resolve with "device" right from that click.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use vectorcraft_ui_egui::Services;
use vectorcraft_ui_egui::io::is_host_path;
use wasm_bindgen::JsCast as _;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::web::Inbox;

fn storage() -> Option<JsValue> {
    let window: JsValue = web_sys::window()?.into();
    let storage = js_sys::Reflect::get(&window, &"craftStorage".into()).ok()?;
    (!storage.is_undefined() && !storage.is_null()).then_some(storage)
}

async fn call(method: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    let storage = storage().ok_or(JsValue::NULL)?;
    let f: js_sys::Function = js_sys::Reflect::get(&storage, &method.into())?.dyn_into()?;
    let promise: js_sys::Promise = f.apply(&storage, &args.iter().collect::<js_sys::Array>())?.dyn_into()?;
    JsFuture::from(promise).await
}

/// A save the app counted done that the host still has to make: whether the file was stored, and
/// the host path the user chose for it in the host's Save dialog.
type Save = Pin<Box<dyn Future<Output = (bool, Option<String>)>>>;

thread_local! {
    /// Saves waiting for the host, made one at a time in the order the app asked, so a later
    /// save of a file is never overwritten by an earlier one: (id, the document whose save it is,
    /// name or host path, the save).
    static QUEUE: RefCell<VecDeque<(u64, Option<u64>, String, Save)>> = const { RefCell::new(VecDeque::new()) };
    /// A save is being made now.
    static SAVING: Cell<bool> = const { Cell::new(false) };
    /// The latest save of each document (or, for an export, each name): an earlier one that
    /// failed doesn't count once a later one is on its way, which answers for the document itself.
    static LATEST: RefCell<HashMap<(Option<u64>, String), u64>> = RefCell::new(HashMap::new());
    static LAST_ID: Cell<u64> = const { Cell::new(0) };
    /// Saves that finished since the web shell last looked. One that failed or that the user
    /// cancelled marks its document modified again ([`take_settled`]).
    static SETTLED: RefCell<Vec<Settled>> = const { RefCell::new(Vec::new()) };
}

fn in_flight() -> bool {
    SAVING.with(Cell::get) || QUEUE.with(|q| !q.borrow().is_empty())
}

/// A finished save, for [`vectorcraft_ui_egui::io::save_settled`]: (document, name or host path,
/// saved, the host path the user chose).
pub type Settled = (Option<u64>, String, bool, Option<String>);

/// What tells saves apart for [`LATEST`]: the document, or an export's name.
fn key(doc: Option<u64>, name: &str) -> (Option<u64>, String) {
    (doc, if doc.is_some() { String::new() } else { name.to_string() })
}

/// Queue a save of `name` for `doc`; the first one starts the saver, which works through the queue.
fn enqueue(doc: Option<u64>, name: String, save: Save, ctx: egui::Context) {
    let id = LAST_ID.with(|n| {
        n.set(n.get() + 1);
        n.get()
    });
    LATEST.with(|l| l.borrow_mut().insert(key(doc, &name), id));
    let start = !in_flight();
    QUEUE.with(|q| q.borrow_mut().push_back((id, doc, name, save)));
    if !start {
        return;
    }
    SAVING.with(|s| s.set(true));
    wasm_bindgen_futures::spawn_local(async move {
        while let Some((id, doc, name, save)) = QUEUE.with(|q| q.borrow_mut().pop_front()) {
            let (saved, host_path) = save.await;
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
            SETTLED.with(|s| s.borrow_mut().push((doc, name, saved || !latest, host_path.filter(|_| saved))));
            ctx.request_repaint();
        }
        SAVING.with(|s| s.set(false));
        ctx.request_repaint();
    });
}

/// The saves that finished since the last call, for [`vectorcraft_ui_egui::io::save_settled`].
pub fn take_settled() -> Vec<Settled> {
    SETTLED.with(|s| std::mem::take(&mut *s.borrow_mut()))
}

async fn write(path: &str, bytes: &[u8]) -> bool {
    let args = [JsValue::from(path), js_sys::Uint8Array::from(bytes).into()];
    match call("write", &args).await {
        Ok(_) => true,
        Err(e) => {
            log::error!("vectorcraft-web: craftStorage.write failed: {e:?}");
            false
        }
    }
}

/// Wrap the async open and the download in `services` so they offer the host's storage when the
/// page has one: a file picked there arrives in `inbox` under its host path. The device's own
/// picker and download stay one choice away.
pub fn install(services: &mut Services, inbox: Inbox, ctx: egui::Context) {
    if let Some(device_open) = services.open_async.take() {
        let device_open = Rc::new(RefCell::new(device_open));
        let (inbox, ctx) = (inbox.clone(), ctx.clone());
        services.open_async = Some(Box::new(move || {
            if storage().is_none() {
                return (device_open.borrow_mut())();
            }
            let (device_open, inbox, ctx) = (device_open.clone(), inbox.clone(), ctx.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match call("chooseOpen", &[JsValue::NULL]).await {
                    Ok(c) if c.is_null() => {}
                    Ok(c) if c.is_object() => {
                        let path = js_sys::Reflect::get(&c, &"path".into()).ok().and_then(|p| p.as_string()).unwrap_or_default();
                        let bytes = js_sys::Reflect::get(&c, &"bytes".into()).map(|b| js_sys::Uint8Array::new(&b).to_vec()).unwrap_or_default();
                        inbox.lock().unwrap_or_else(|e| e.into_inner()).push((path, bytes));
                        ctx.request_repaint();
                    }
                    // Only the user's own choice opens the browser's picker.
                    Ok(c) if c.as_string().as_deref() == Some("device") => (device_open.borrow_mut())(),
                    // A host that failed or answered something else: nothing was chosen.
                    other => log::error!("vectorcraft-web: craftStorage.chooseOpen didn't answer with a choice: {other:?}"),
                }
            });
        }));
    }
    if let Some(device_download) = services.download.take() {
        services.save_in_flight = Some(Box::new(in_flight));
        let device_download = Rc::new(RefCell::new(device_download));
        services.download = Some(Box::new(move |name: &str, bytes: &[u8]| {
            if storage().is_none() {
                return (device_download.borrow_mut())(name, bytes);
            }
            // The save counts as done now; one that doesn't happen marks the document modified again.
            let doc = vectorcraft_ui_egui::io::saving_document();
            let (device_download, owned, bytes) = (device_download.clone(), name.to_string(), bytes.to_vec());
            let save: Save = Box::pin(async move {
                let name = owned;
                if is_host_path(&name) {
                    // File › Save of a document from the host: straight back to its path.
                    (write(&name, &bytes).await, None)
                } else {
                    // Anything else the app would download (Save As, exports): ask where it goes.
                    match call("chooseSave", &[name.clone().into()]).await {
                        Ok(c) if c.is_null() => (false, None),
                        Ok(c) if let Some(path) = c.as_string().filter(|p| is_host_path(p)) => (write(&path, &bytes).await, Some(path)),
                        // Only the user's own choice downloads to the device.
                        Ok(c) if c.as_string().as_deref() == Some("device") => {
                            let downloaded = (device_download.borrow_mut())(&name, &bytes)
                                .inspect_err(|e| log::error!("vectorcraft-web: download of {name} failed: {e}"))
                                .is_ok();
                            (downloaded, None)
                        }
                        // A host that failed or answered something else: not saved, so the
                        // document stays modified.
                        other => {
                            log::error!("vectorcraft-web: craftStorage.chooseSave didn't answer with a choice: {other:?}");
                            (false, None)
                        }
                    }
                }
            });
            enqueue(doc, name.to_string(), save, ctx.clone());
            Ok(())
        }));
    }
}
