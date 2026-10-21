//! Documents whose own file changes on disk ([`crate::file_watch`]): a clean document reloads in
//! place, one with unsaved changes asks first, and VectorCraft's own saves are never taken for
//! another app's changes.

// The watcher is desktop only (`crate::file_watch`).
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use serde_json::{Value, json};
use vectorcraft_engine::Session;

use crate::file_watch::{Watched, Watcher};
use crate::state::Dialog;
use crate::{Services, VectorcraftApp};

/// A fresh folder for one test.
fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vc-filewatch-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn path(d: &std::path::Path, name: &str) -> String {
    d.join(name).to_string_lossy().into_owned()
}

/// An SVG with `n` rectangles.
fn svg(n: usize) -> String {
    let rects: String = (0..n).map(|i| format!(r#"<rect x="{}" y="10" width="5" height="5"/>"#, 10 + 10 * i)).collect();
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">{rects}</svg>"#)
}

/// Another app writes `file`, late enough that its stamp moves even on coarse file systems.
fn write_elsewhere(file: &str, bytes: impl AsRef<[u8]>) {
    std::thread::sleep(std::time::Duration::from_millis(30));
    std::fs::write(file, bytes).unwrap();
}

/// An app whose watcher looks on the UI thread, once per frame, with `file` open.
fn app_with(file: &str) -> VectorcraftApp {
    let mut app = VectorcraftApp::new(Session::new(), Services::default());
    app.file_watch.inline = Some(Watcher::default());
    app.session.execute("document.open", &json!({"path": file})).unwrap();
    frames(&mut app);
    app
}

/// Enough frames for a change to be seen, seen unchanged again, and read.
fn frames(app: &mut VectorcraftApp) {
    let ctx = egui::Context::default();
    for _ in 0..3 {
        app.tick_file_watch(&ctx);
    }
}

fn objects(app: &VectorcraftApp) -> usize {
    app.session.doc().unwrap().doc.node_count()
}

fn asking(app: &VectorcraftApp) -> bool {
    app.ui.dialog.as_ref().is_some_and(|d| d.kind == crate::dialogs::confirm::KIND && d.fields.get("__command") == Some(&json!("file.revert")))
}

#[test]
fn a_clean_document_reloads_in_place_once() {
    let d = dir("clean");
    let file = path(&d, "art.svg");
    std::fs::write(&file, svg(1)).unwrap();
    let mut app = app_with(&file);
    let (uid, one) = (app.session.doc().unwrap().uid, objects(&app));
    write_elsewhere(&file, svg(3));
    frames(&mut app);
    let st = app.session.doc().unwrap();
    assert_eq!((objects(&app), st.uid, st.path.as_deref()), (one + 2, uid, Some(file.as_str())));
    assert!(!st.is_dirty() && !asking(&app));
    assert_eq!(app.ui.status, "Reloaded art.svg: the file changed on disk");
    // Acted on once: the same version doesn't reload again.
    app.ui.status.clear();
    frames(&mut app);
    assert!(app.ui.status.is_empty());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn unsaved_changes_ask_first_and_cancel_keeps_them() {
    let d = dir("dirty");
    let file = path(&d, "art.svg");
    std::fs::write(&file, svg(1)).unwrap();
    let mut app = app_with(&file);
    app.run("shape.rectangle", json!({"x": 100, "y": 50, "width": 5, "height": 5})).unwrap();
    let edited = objects(&app);
    write_elsewhere(&file, svg(3));
    frames(&mut app);
    assert!(asking(&app), "{:?}", app.ui.dialog.as_ref().map(|d| &d.kind));
    assert_eq!(objects(&app), edited);
    // Cancel: the changes stay, and the same version isn't asked about again.
    crate::dialogs::cancel(&mut app);
    frames(&mut app);
    assert!(!asking(&app));
    assert_eq!(objects(&app), edited);
    // A newer version asks again; OK reads it.
    write_elsewhere(&file, svg(4));
    frames(&mut app);
    assert!(asking(&app));
    crate::dialogs::confirm(&mut app).unwrap();
    assert_eq!(objects(&app), edited + 2);
    assert!(!app.session.doc().unwrap().is_dirty());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_documents_own_saves_never_reload() {
    let d = dir("own");
    let file = path(&d, "art.svg");
    std::fs::write(&file, svg(1)).unwrap();
    let mut app = app_with(&file);
    app.run("shape.rectangle", json!({"x": 100, "y": 50, "width": 5, "height": 5})).unwrap();
    app.session.execute("document.save", &json!({})).unwrap();
    frames(&mut app);
    frames(&mut app);
    let st = app.session.doc().unwrap();
    assert!(!asking(&app) && app.ui.status.is_empty(), "{}", app.ui.status);
    assert!(!st.history.undo.is_empty(), "not read again: the history is still there");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_version_found_before_a_save_is_not_applied_over_it() {
    let d = dir("stale");
    let file = path(&d, "art.svg");
    std::fs::write(&file, svg(1)).unwrap();
    let mut app = app_with(&file);
    // Another app writes the file while a dialog is open: the new version waits.
    app.ui.dialog = Some(Dialog::new("message", Value::Null));
    write_elsewhere(&file, svg(3));
    frames(&mut app);
    // The document is saved over it before the dialog closes: that version is older now.
    app.ui.dialog = None;
    app.run("shape.rectangle", json!({"x": 100, "y": 50, "width": 5, "height": 5})).unwrap();
    let edited = objects(&app);
    app.session.execute("document.save", &json!({})).unwrap();
    frames(&mut app);
    assert!(!asking(&app) && app.ui.status.is_empty(), "{}", app.ui.status);
    assert_eq!(objects(&app), edited);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn save_as_watches_the_new_file() {
    let d = dir("saveas");
    let (a, b) = (path(&d, "a.svg"), path(&d, "b.svg"));
    std::fs::write(&a, svg(1)).unwrap();
    let mut app = app_with(&a);
    app.session.execute("file.saveAs", &json!({"path": b})).unwrap();
    frames(&mut app);
    let one = objects(&app);
    write_elsewhere(&a, svg(5));
    frames(&mut app);
    assert_eq!(objects(&app), one, "no longer the document's file");
    write_elsewhere(&b, svg(5));
    frames(&mut app);
    assert_eq!(objects(&app), one + 4);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_document_in_the_back_reloads_too_and_asks_when_in_front() {
    let d = dir("back");
    let (a, b) = (path(&d, "a.svg"), path(&d, "b.svg"));
    std::fs::write(&a, svg(1)).unwrap();
    std::fs::write(&b, svg(1)).unwrap();
    let mut app = app_with(&a);
    app.run("shape.rectangle", json!({"x": 100, "y": 50, "width": 5, "height": 5})).unwrap();
    app.session.execute("document.open", &json!({"path": b})).unwrap();
    frames(&mut app);
    let count = |app: &VectorcraftApp, i: usize| app.session.documents()[i].doc.node_count();
    let (a_edited, b_one) = (count(&app, 0), count(&app, 1));
    write_elsewhere(&a, svg(3));
    write_elsewhere(&b, svg(3));
    frames(&mut app);
    // b (in front, clean) reloads; a (in the back, modified) waits to be asked.
    assert_eq!((count(&app, 0), count(&app, 1)), (a_edited, b_one + 2));
    assert!(!asking(&app));
    app.session.set_active(0);
    frames(&mut app);
    assert!(asking(&app));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_file_that_cant_be_read_leaves_the_document_until_it_changes_again() {
    let d = dir("broken");
    let file = path(&d, "art.vectorcraft");
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 200, "height": 100})).unwrap();
    s.execute("document.save", &json!({"path": file})).unwrap();
    let good = std::fs::read(&file).unwrap();
    let mut app = app_with(&file);
    let one = objects(&app);
    write_elsewhere(&file, b"{ not a document");
    frames(&mut app);
    assert_eq!(objects(&app), one);
    assert!(!app.ui.status.is_empty(), "the error is shown");
    app.ui.status.clear();
    frames(&mut app);
    assert!(app.ui.status.is_empty(), "once");
    // A good version reloads.
    s.execute("shape.rectangle", &json!({"x": 10, "y": 10, "width": 5, "height": 5})).unwrap();
    s.execute("document.save", &json!({})).unwrap();
    assert_ne!(std::fs::read(&file).unwrap(), good);
    frames(&mut app);
    assert_eq!(objects(&app), one + 1);
    let _ = std::fs::remove_dir_all(d);
}

/// A half-written SVG is not read: the watcher waits for a version that ends as a whole SVG does.
#[test]
fn a_half_written_svg_waits_for_the_whole_file() {
    let d = dir("half");
    let file = path(&d, "art.svg");
    std::fs::write(&file, svg(1)).unwrap();
    let mut w = Watcher::default();
    w.set(vec![Watched { doc: 1, path: file.clone(), known: None, version: 0 }]);
    let looks = |w: &mut Watcher| (0..3).flat_map(|_| w.tick().0).collect::<Vec<_>>();
    assert!(looks(&mut w).is_empty(), "first seen: remembered");
    let whole = svg(3);
    write_elsewhere(&file, &whole[..whole.len() - 10]);
    assert!(looks(&mut w).is_empty(), "not a whole SVG");
    write_elsewhere(&file, &whole);
    let found = looks(&mut w);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].bytes.as_slice(), whole.as_bytes());
    // A version the UI thread knows (it wrote it) is not a change.
    write_elsewhere(&file, svg(4));
    w.set(vec![Watched { doc: 1, path: file.clone(), known: crate::file_watch::Stamp::at(&file), version: 1 }]);
    assert!(looks(&mut w).is_empty());
    let _ = std::fs::remove_dir_all(d);
}

/// A version whose metadata alone changed (rewritten with the same bytes, `touch`, `chmod`, a sync
/// tool's extended attributes) is not a change: only one whose contents differ is found.
#[test]
fn a_change_of_metadata_alone_is_not_a_change() {
    let d = dir("meta");
    let file = path(&d, "art.svg");
    std::fs::write(&file, svg(1)).unwrap();
    let mut w = Watcher::default();
    w.set(vec![Watched { doc: 1, path: file.clone(), known: crate::file_watch::Stamp::at(&file), version: 0 }]);
    let looks = |w: &mut Watcher| (0..3).flat_map(|_| w.tick().0).collect::<Vec<_>>();
    assert!(looks(&mut w).is_empty());
    // The same bytes written again, and a new modification time alone.
    write_elsewhere(&file, svg(1));
    assert!(looks(&mut w).is_empty(), "rewritten with the same bytes");
    let later = std::time::SystemTime::now() + std::time::Duration::from_secs(60);
    std::fs::File::options().write(true).open(&file).unwrap().set_modified(later).unwrap();
    assert!(looks(&mut w).is_empty(), "touched");
    write_elsewhere(&file, svg(2));
    assert_eq!(looks(&mut w).len(), 1, "new contents");
    let _ = std::fs::remove_dir_all(d);
}

/// In the app: a clean document whose file is rewritten with the same bytes stays as it is, its
/// history and selection with it.
#[test]
fn a_file_rewritten_with_the_same_bytes_doesnt_reload() {
    let d = dir("samebytes");
    let file = path(&d, "art.svg");
    std::fs::write(&file, svg(2)).unwrap();
    let mut app = app_with(&file);
    app.run("select.all", json!({})).unwrap();
    let revision = app.session.doc().unwrap().revision;
    write_elsewhere(&file, svg(2));
    frames(&mut app);
    frames(&mut app);
    assert!(app.ui.status.is_empty() && !asking(&app), "{}", app.ui.status);
    let st = app.session.doc().unwrap();
    assert_eq!(st.revision, revision, "not replaced");
    assert!(!st.selection.is_empty());
    let _ = std::fs::remove_dir_all(d);
}
