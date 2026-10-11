//! File › Scripts › Other Script…: the menu item, a picked script's run as one undo step, the
//! error and alert dialogs, and scripts picked in the browser.

use std::sync::{Arc, Mutex};

use serde_json::json;
use vectorcraft_engine::Session;

use crate::{VectorcraftApp, dialogs, menus, scripts, theme};

fn app() -> VectorcraftApp {
    let mut app = VectorcraftApp::new(Session::new(), Default::default());
    app.run("file.new", json!({"width": 200, "height": 200})).unwrap();
    app.services.read = Some(Box::new(|p: &str| std::fs::read(p).map_err(|e| e.to_string())));
    app
}

fn script(name: &str, src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("vectorcraft-ui-scripts-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(name);
    std::fs::write(&p, src).unwrap();
    p.to_string_lossy().to_string()
}

fn count(app: &VectorcraftApp) -> usize {
    app.session.doc().unwrap().doc.layers[0].children().unwrap().len()
}

/// One headless frame of the dialog layer.
fn frame(app: &mut VectorcraftApp) {
    let ctx = egui::Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, Default::default());
    let mut out = ctx.run_ui(Default::default(), |ui| dialogs::show(app, ui.ctx()));
    out.textures_delta.clear();
}

#[test]
fn other_script_is_in_the_file_menu_with_its_shortcut() {
    let app = app();
    let entries = menus::menu_entries(&app);
    let e = entries.iter().find(|e| e.path == ["File", "Scripts"] && e.label == "Other Script…").expect("listed");
    assert_eq!(e.command.as_deref(), Some("ui.runScript"));
    assert!(e.enabled);
    assert!(e.shortcut.contains("F12"), "{}", e.shortcut);
}

#[test]
fn a_picked_script_runs_as_one_undo_step() {
    let mut app = app();
    let path = script(
        "draw.js",
        "vc.run('shape.rectangle', {x: 10, y: 10, width: 50, height: 50});\nvc.run('shape.ellipse', {x: 80, y: 10, width: 50, height: 50});",
    );
    app.services.pick_open = Some(Box::new(move |_: &crate::FilePick| Some(path.clone())));
    app.run("ui.runScript", json!({})).unwrap();
    assert_eq!(count(&app), 2);
    let labels: Vec<String> = app.session.doc().unwrap().history.undo.iter().map(|h| h.label.clone()).collect();
    assert_eq!(labels, ["Script: draw.js"]);
    assert!(app.ui.dialog.is_none(), "a quiet script shows no dialog");
}

#[test]
fn a_failing_script_shows_the_error_and_changes_nothing() {
    let mut app = app();
    let path = script("boom.js", "vc.run('shape.rectangle', {x: 10, y: 10, width: 50, height: 50});\nthrow new Error('kaput');");
    assert!(app.run("ui.runScript", json!({"path": path})).is_err());
    assert_eq!(count(&app), 0);
    let d = app.ui.dialog.as_ref().expect("error dialog");
    assert!(d.str("message").contains("boom.js"), "{}", d.str("message"));
    assert!(d.str("detail").contains("boom.js:2:") && d.str("detail").contains("kaput"), "{}", d.str("detail"));
    frame(&mut app);
}

#[test]
fn alerts_and_the_log_are_shown_after_the_run() {
    let mut app = app();
    let path = script("hello.js", "vc.log('logged'); alert('hello there');");
    app.run("ui.runScript", json!({"path": path})).unwrap();
    let d = app.ui.dialog.as_ref().expect("dialog");
    assert!(d.str("detail").contains("hello there") && d.str("detail").contains("logged"), "{}", d.str("detail"));
    frame(&mut app);
}

#[test]
fn scripts_picked_in_the_browser_run_from_the_inbox() {
    let mut app = app();
    let inbox: crate::Inbox = Arc::new(Mutex::new(vec![]));
    let started = Arc::new(Mutex::new(0));
    let s2 = started.clone();
    app.services.script_async = Some(Box::new(move || *s2.lock().unwrap() += 1));
    app.services.script_inbox = Some(inbox.clone());
    app.run("ui.runScript", json!({})).unwrap();
    assert_eq!(*started.lock().unwrap(), 1, "the browser's picker opened");
    inbox.lock().unwrap().push(("web.js".into(), b"vc.run('shape.rectangle', {x: 1, y: 1, width: 5, height: 5})".to_vec()));
    scripts::drain(&mut app);
    assert_eq!(count(&app), 1);
}

#[test]
fn a_script_needs_no_open_document() {
    let mut app = VectorcraftApp::new(Session::new(), Default::default());
    app.services.read = Some(Box::new(|p: &str| std::fs::read(p).map_err(|e| e.to_string())));
    let path = script("new.js", "vc.run('file.new', {width: 100, height: 100}); vc.run('shape.rectangle', {x: 1, y: 1, width: 5, height: 5});");
    app.run("ui.runScript", json!({"path": path})).unwrap();
    assert_eq!(count(&app), 1);
}
