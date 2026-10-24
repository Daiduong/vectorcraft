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
//!   async write(path, bytes /* Uint8Array */) { … },
//! };
//! ```
//!
//! Host paths have a scheme of their own, such as `cloud:home/art/logo.svg`
//! ([`vectorcraft_ui_egui::io::is_host_path`]): a document opened from one keeps it, so File › Save
//! writes back there without asking. Without `window.craftStorage` nothing changes.

use std::cell::RefCell;
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

fn write(path: String, bytes: &[u8]) {
    let args = [JsValue::from(path), js_sys::Uint8Array::from(bytes).into()];
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = call("write", &args).await {
            log::error!("vectorcraft-web: craftStorage.write failed: {e:?}");
        }
    });
}

/// Wrap the async open and the download in `services` so they offer the host's storage when the
/// page has one: a file picked there arrives in `inbox` under its host path. The device's own
/// picker and download stay one choice away.
pub fn install(services: &mut Services, inbox: Inbox, ctx: egui::Context) {
    if let Some(device_open) = services.open_async.take() {
        let device_open = Rc::new(RefCell::new(device_open));
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
                    // "device", nothing, or a failed host: the browser's own picker.
                    _ => (device_open.borrow_mut())(),
                }
            });
        }));
    }
    if let Some(device_download) = services.download.take() {
        let device_download = Rc::new(RefCell::new(device_download));
        services.download = Some(Box::new(move |name: &str, bytes: &[u8]| {
            // File › Save of a document from the host: straight back to its path.
            if is_host_path(name) && storage().is_some() {
                write(name.to_string(), bytes);
                return Ok(());
            }
            if storage().is_none() {
                return (device_download.borrow_mut())(name, bytes);
            }
            // Anything else the app would download (Save As, exports): ask where it goes.
            let (device_download, name, bytes) = (device_download.clone(), name.to_string(), bytes.to_vec());
            wasm_bindgen_futures::spawn_local(async move {
                match call("chooseSave", &[name.clone().into()]).await {
                    Ok(c) if c.is_null() => {}
                    Ok(c) if c.as_string().is_some_and(|p| is_host_path(&p)) => write(c.as_string().unwrap_or_default(), &bytes),
                    _ => {
                        if let Err(e) = (device_download.borrow_mut())(&name, &bytes) {
                            log::error!("vectorcraft-web: download of {name} failed: {e}");
                        }
                    }
                }
            });
            Ok(())
        }));
    }
}
