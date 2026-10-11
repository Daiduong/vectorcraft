//! #1234: empty artboard colours go through the same paint/undo path as object samples.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_color::Paint;
use vectorcraft_doc::{Node, NodeId};
use vectorcraft_engine::{Session, UiRequest, ViewInfo};
use vectorcraft_tools::{Mods, PointerEvent, PointerKind};

fn fixture() -> (Session, NodeId) {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 300, "height": 200})).unwrap();
    let id = NodeId(s.execute("shape.rectangle", &json!({"x": 40, "y": 40, "width": 80, "height": 60})).unwrap()["id"].as_u64().unwrap());
    s.execute("paint.setStroke", &json!({"color": "#0055ff"})).unwrap();
    s.execute("stroke.set", &json!({"weight": 5})).unwrap();
    s.execute("paint.setFill", &json!({"color": "#ff3300"})).unwrap();
    s.select_tool("eyedropper", ViewInfo::default()).unwrap();
    (s, id)
}

fn node(s: &Session, id: NodeId) -> Node {
    s.doc().unwrap().doc.node(id).unwrap().clone()
}

fn sample(s: &mut Session, x: f64, y: f64, mods: Mods) -> Vec<UiRequest> {
    let mut requests = vec![];
    for kind in [PointerKind::Down, PointerKind::Up] {
        requests.extend(s.pointer(&PointerEvent::new(kind, x, y).with_mods(mods), ViewInfo::default()).unwrap());
    }
    requests
}

#[test]
fn paper_click_changes_only_the_active_paint_in_one_undo_step() {
    let (mut s, id) = fixture();
    let before = node(&s, id);
    let undo = s.doc().unwrap().history.undo.len();
    sample(&mut s, 220.0, 140.0, Mods::default());
    let after = node(&s, id);
    assert_eq!(after.appearance.fill_paint().color().unwrap().to_hex(), "#ffffff");
    assert_eq!(after.appearance.stroke(), before.appearance.stroke());
    assert_eq!(after.kind, before.kind);
    assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(node(&s, id), before);
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(node(&s, id), after);
}

#[test]
fn simulated_paper_can_be_sampled_into_the_stroke_with_shift() {
    let (mut s, id) = fixture();
    s.execute("document.setup", &json!({"simulatePaper": true, "gridColors": ["#22aa66", "#dddddd"]})).unwrap();
    s.execute("paint.toggleActive", &json!({"fill": false})).unwrap();
    let before = node(&s, id);
    sample(&mut s, 220.0, 140.0, Mods { shift: true, ..Default::default() });
    let after = node(&s, id);
    assert_eq!(after.appearance.stroke_paint().color().unwrap().to_hex(), "#22aa66");
    assert_eq!(after.appearance.fill_paint(), before.appearance.fill_paint());
    assert_eq!(after.appearance.stroke().unwrap().width, 5.0);
    assert_eq!(after.kind, before.kind);
}

#[test]
fn checkerboard_and_pasteboard_are_not_paint_samples() {
    let (mut s, id) = fixture();
    let before = node(&s, id);
    let undo = s.doc().unwrap().history.undo.len();
    for mods in [Mods::default(), Mods { shift: true, ..Default::default() }, Mods { alt: true, ..Default::default() }] {
        sample(&mut s, 320.0, 140.0, mods);
    }
    // Alt applies the selected object's appearance to another object, not to the paper.
    sample(&mut s, 220.0, 140.0, Mods { alt: true, ..Default::default() });
    s.execute("view.transparencyGrid", &json!({"on": true})).unwrap();
    sample(&mut s, 220.0, 140.0, Mods::default());
    assert_eq!(node(&s, id), before);
    assert_eq!(s.doc().unwrap().history.undo.len(), undo);
    // White Background Contents hides the checkerboard, so it can be sampled again.
    s.execute("document.setup", &json!({"backgroundContents": "white"})).unwrap();
    sample(&mut s, 220.0, 140.0, Mods::default());
    assert_eq!(node(&s, id).appearance.fill_paint().color().unwrap().to_hex(), "#ffffff");
}

#[test]
fn paper_sampling_updates_only_the_selected_gradient_stop_and_returns_to_the_tool() {
    let (mut s, id) = fixture();
    s.execute("paint.setFill", &json!({"gradient": {}})).unwrap();
    s.execute("gradient.selectStop", &json!({"index": 1})).unwrap();
    s.set_tool_option("stop", &json!("gradient"));
    let before = node(&s, id);
    let requests = sample(&mut s, 220.0, 140.0, Mods::default());
    let after = node(&s, id);
    let Paint::Gradient(g) = after.appearance.fill_paint() else { panic!("gradient replaced") };
    assert_eq!(g.gradient.stops.iter().map(|s| s.color.to_hex()).collect::<Vec<_>>(), ["#ffffff", "#ffffff"]);
    assert_eq!(after.appearance.stroke(), before.appearance.stroke());
    assert!(matches!(requests.as_slice(), [UiRequest::SwitchTool(id)] if id == "gradient"));
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(node(&s, id), before);
}

#[test]
fn locked_art_is_sampled_as_art_instead_of_paper_and_stays_locked() {
    let (mut s, target) = fixture();
    let source = NodeId(s.execute("shape.rectangle", &json!({"x": 160, "y": 40, "width": 60, "height": 50})).unwrap()["id"].as_u64().unwrap());
    s.execute("paint.setFill", &json!({"color": "#1177dd"})).unwrap();
    s.execute("object.lock", &json!({})).unwrap();
    s.execute("select.set", &json!({"ids": [target.0]})).unwrap();
    let before = node(&s, source);
    sample(&mut s, 190.0, 65.0, Mods::default());
    assert_eq!(node(&s, target).appearance.fill_paint().color().unwrap().to_hex(), "#1177dd");
    assert!(before.locked);
    assert_eq!(node(&s, source), before);
}
