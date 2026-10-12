//! `shape.qrCode`: the dark modules as one path, optionally over a quiet-zone square.
#![allow(clippy::unwrap_used)]

use serde_json::json;
use vectorcraft_doc::{NodeId, NodeKind};
use vectorcraft_engine::Session;

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 400, "height": 300})).unwrap();
    s
}

#[test]
fn draws_the_symbol_as_one_selected_path() {
    let mut s = session();
    let r = s.execute("shape.qrCode", &json!({"x": 10, "y": 20, "size": 87, "text": "https://ape.photomates.nohost.me/"})).unwrap();
    assert_eq!(r["version"], 3);
    assert_eq!(r["modules"], 29);
    assert_eq!(r["ecc"], "M");
    let id = NodeId(r["id"].as_u64().unwrap());
    let st = s.doc().unwrap();
    assert_eq!(st.selection.objects, vec![id]);
    let node = st.doc.node(id).unwrap();
    assert_eq!(node.name.as_deref(), Some("QR Code"));
    let NodeKind::Path { path, .. } = &node.kind else { panic!("not a path") };
    assert!(path.subpaths.len() > 29);
    let b = node.geometric_bounds().unwrap();
    assert!((b.x0 - 10.0).abs() < 1e-9 && (b.y0 - 20.0).abs() < 1e-9);
    assert!((b.width() - 87.0).abs() < 1e-9 && (b.height() - 87.0).abs() < 1e-9);
    assert!(!node.appearance.fill_paint().is_none(), "filled even when the current fill is None");
}

#[test]
fn background_adds_a_quiet_zone_square() {
    let mut s = session();
    let r = s
        .execute("shape.qrCode", &json!({"x": 50, "y": 50, "size": 21, "text": "VectorCraft", "ecc": "L", "background": "#ffffff", "quietZone": 2}))
        .unwrap();
    assert_eq!(r["version"], 1);
    let st = s.doc().unwrap();
    let node = st.doc.node(NodeId(r["id"].as_u64().unwrap())).unwrap();
    let NodeKind::Group { children, .. } = &node.kind else { panic!("not a group") };
    assert_eq!(children.len(), 2);
    // One module is 1 pt: the square reaches 2 pt past the symbol on each side.
    let b = children[0].geometric_bounds().unwrap();
    assert_eq!((b.x0, b.y0, b.x1, b.y1), (48.0, 48.0, 73.0, 73.0));
}

#[test]
fn bad_params_fail_and_change_nothing() {
    let mut s = session();
    let before = s.doc().unwrap().history.undo.len();
    for p in [
        json!({"x": 0, "y": 0}),
        json!({"x": 0, "y": 0, "text": ""}),
        json!({"x": 0, "y": 0, "text": "a", "ecc": "X"}),
        json!({"x": 0, "y": 0, "text": "a", "size": -1}),
        json!({"x": 0, "y": 0, "text": "a", "quietZone": 1000}),
        json!({"x": 0, "y": 0, "text": "a", "background": 3}),
        json!({"x": 0, "y": 0, "text": "a".repeat(3000), "ecc": "L"}),
    ] {
        assert!(s.execute("shape.qrCode", &p).is_err(), "{p}");
    }
    assert_eq!(s.doc().unwrap().history.undo.len(), before);
}
