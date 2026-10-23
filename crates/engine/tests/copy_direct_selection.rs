//! Direct-selection Copy and Cut tests (#2122, #2441).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::{Value, json};
use vectorcraft_doc::{NodeId, NodeKind};
use vectorcraft_engine::Session;
use vectorcraft_geom::FillRule;

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap()
}

fn rect(s: &mut Session, x: f64, y: f64, w: f64, h: f64) -> NodeId {
    NodeId(run(s, "shape.rectangle", json!({"x": x, "y": y, "width": w, "height": h}))["id"].as_u64().unwrap())
}

#[test]
fn copy_and_paste_direct_selected_segment_of_rect() {
    let mut s = Session::new();
    run(&mut s, "file.new", json!({"width": 200, "height": 200}));
    let r = rect(&mut s, 10.0, 10.0, 100.0, 50.0);

    // Select segment 0 of the rectangle.
    run(&mut s, "select.segments", json!({"id": r.0, "segments": [[0, 0]]}));
    run(&mut s, "edit.copy", json!({}));
    let pasted = run(&mut s, "edit.paste", json!({}));
    let ids: Vec<u64> = pasted["ids"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    assert_eq!(ids.len(), 1);

    let pasted_node = s.doc().unwrap().doc.node(NodeId(ids[0])).unwrap();
    if let NodeKind::Path { live, .. } = &pasted_node.kind {
        assert!(live.is_none(), "extracted partial path must not retain live shape metadata");
    }
    let path = pasted_node.path_data().unwrap();
    assert_eq!(path.subpaths.len(), 1);
    let sp = &path.subpaths[0];
    assert!(!sp.closed);
    assert_eq!(sp.anchors.len(), 2);
}

#[test]
fn copy_and_paste_direct_selected_adjacent_anchors_of_rect() {
    let mut s = Session::new();
    run(&mut s, "file.new", json!({"width": 200, "height": 200}));
    let r = rect(&mut s, 10.0, 10.0, 100.0, 50.0);

    // Direct-select two adjacent anchors: 0 and 1.
    run(&mut s, "select.anchors", json!({"id": r.0, "anchors": [[0, 0], [0, 1]], "mode": "set"}));
    run(&mut s, "edit.copy", json!({}));
    let pasted = run(&mut s, "edit.paste", json!({}));
    let ids: Vec<u64> = pasted["ids"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    assert_eq!(ids.len(), 1);

    let pasted_node = s.doc().unwrap().doc.node(NodeId(ids[0])).unwrap();
    let path = pasted_node.path_data().unwrap();
    assert_eq!(path.subpaths.len(), 1);
    let sp = &path.subpaths[0];
    assert!(!sp.closed);
    assert_eq!(sp.anchors.len(), 2);
}

#[test]
fn copy_single_anchor_of_rect_copies_three_anchors_with_two_segments() {
    let mut s = Session::new();
    run(&mut s, "file.new", json!({"width": 200, "height": 200}));
    let r = rect(&mut s, 10.0, 10.0, 100.0, 50.0);

    // Direct-select only anchor 1 of the closed 4-anchor rectangle.
    run(&mut s, "select.anchors", json!({"id": r.0, "anchors": [[0, 1]], "mode": "set"}));
    run(&mut s, "edit.copy", json!({}));
    let pasted = run(&mut s, "edit.paste", json!({}));
    let ids: Vec<u64> = pasted["ids"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    assert_eq!(ids.len(), 1);

    let pasted_node = s.doc().unwrap().doc.node(NodeId(ids[0])).unwrap();
    let path = pasted_node.path_data().unwrap();
    assert_eq!(path.subpaths.len(), 1);
    let sp = &path.subpaths[0];
    assert!(!sp.closed);
    // Copying a single anchor on a closed path copies the anchor and both its incident segments.
    assert_eq!(sp.anchors.len(), 3);
    assert_eq!(sp.segment_count(), 2);
}

#[test]
fn copy_disjoint_segments_does_not_copy_unselected_segment_in_between() {
    let mut s = Session::new();
    run(&mut s, "file.new", json!({"width": 200, "height": 200}));
    let r = rect(&mut s, 10.0, 10.0, 100.0, 50.0);

    // Rectangle has 4 segments (0, 1, 2, 3). Select disjoint segments 0 and 2.
    run(&mut s, "select.segments", json!({"id": r.0, "segments": [[0, 0], [0, 2]]}));
    run(&mut s, "edit.copy", json!({}));
    let pasted = run(&mut s, "edit.paste", json!({}));
    let ids: Vec<u64> = pasted["ids"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    // Two separate open path pieces should be created, each with 2 anchors.
    assert_eq!(ids.len(), 2);
    for id in ids {
        let node = s.doc().unwrap().doc.node(NodeId(id)).unwrap();
        let path = node.path_data().unwrap();
        assert_eq!(path.subpaths.len(), 1);
        let sp = &path.subpaths[0];
        assert!(!sp.closed);
        assert_eq!(sp.anchors.len(), 2);
    }
}

#[test]
fn cut_direct_selected_segment_cuts_source_and_pastes_segment() {
    let mut s = Session::new();
    run(&mut s, "file.new", json!({"width": 200, "height": 200}));
    let r = rect(&mut s, 10.0, 10.0, 100.0, 50.0);

    // Select segment 0 and cut it.
    run(&mut s, "select.segments", json!({"id": r.0, "segments": [[0, 0]]}));
    run(&mut s, "edit.cut", json!({}));

    // Source rectangle should now have segment 0 removed, turning it into an open path.
    let src_node = s.doc().unwrap().doc.node(r).unwrap();
    let src_path = src_node.path_data().unwrap();
    assert_eq!(src_path.subpaths.len(), 1);
    assert!(!src_path.subpaths[0].closed);

    // Paste the cut segment:
    let pasted = run(&mut s, "edit.paste", json!({}));
    let ids: Vec<u64> = pasted["ids"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    assert_eq!(ids.len(), 1);
    let pasted_node = s.doc().unwrap().doc.node(NodeId(ids[0])).unwrap();
    let pasted_path = pasted_node.path_data().unwrap();
    assert_eq!(pasted_path.subpaths.len(), 1);
    assert!(!pasted_path.subpaths[0].closed);
    assert_eq!(pasted_path.subpaths[0].anchors.len(), 2);
}

#[test]
fn copy_compound_path_direct_selected_segment_inherits_appearance() {
    let mut s = Session::new();
    run(&mut s, "file.new", json!({"width": 200, "height": 200}));
    let outer = NodeId(run(&mut s, "shape.rectangle", json!({"x": 10, "y": 10, "width": 80, "height": 80}))["id"].as_u64().unwrap());
    let inner = NodeId(run(&mut s, "shape.rectangle", json!({"x": 30, "y": 30, "width": 40, "height": 40}))["id"].as_u64().unwrap());
    run(&mut s, "select.set", json!({"ids": [outer.0, inner.0]}));
    let compound = NodeId(run(&mut s, "object.compoundPath.make", json!({}))["id"].as_u64().unwrap());
    run(&mut s, "paint.setFill", json!({"color": "#ff0000"}));

    // Select segment 0 of the inner member.
    run(&mut s, "select.segments", json!({"id": inner.0, "segments": [[0, 0]]}));
    run(&mut s, "edit.copy", json!({}));
    let pasted = run(&mut s, "edit.paste", json!({}));
    let ids: Vec<u64> = pasted["ids"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    assert_eq!(ids.len(), 1);

    let pasted_node = s.doc().unwrap().doc.node(NodeId(ids[0])).unwrap();
    let compound_node = s.doc().unwrap().doc.node(compound).unwrap();
    assert_eq!(pasted_node.appearance, compound_node.appearance);
}

#[test]
fn direct_selection_copy_preserves_fill_rule_and_attributes() {
    let mut s = Session::new();
    run(&mut s, "file.new", json!({"width": 200, "height": 200}));
    let r = rect(&mut s, 10.0, 10.0, 100.0, 50.0);
    run(&mut s, "path.setFillRule", json!({"rule": "evenOdd"}));

    run(&mut s, "select.segments", json!({"id": r.0, "segments": [[0, 0]]}));
    run(&mut s, "edit.copy", json!({}));
    let pasted = run(&mut s, "edit.paste", json!({}));
    let ids: Vec<u64> = pasted["ids"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    let pasted_node = s.doc().unwrap().doc.node(NodeId(ids[0])).unwrap();
    assert!(matches!(pasted_node.kind, NodeKind::Path { rule: FillRule::EvenOdd, .. }));
}

#[test]
fn pen_tool_drawing_copies_whole_path_not_single_anchor() {
    let mut s = Session::new();
    run(&mut s, "file.new", json!({"width": 200, "height": 200}));
    // Start drawing with Pen tool
    s.select_tool("pen", vectorcraft_engine::ViewInfo::default()).unwrap();
    let id = NodeId(run(&mut s, "path.create", json!({"anchors": [{"x": 10, "y": 10}]}))["id"].as_u64().unwrap());
    run(&mut s, "path.appendAnchor", json!({"id": id.0, "x": 50, "y": 10}));
    run(&mut s, "path.appendAnchor", json!({"id": id.0, "x": 50, "y": 50}));

    // Pen tool has the last anchor in sel.anchors, but active tool is pen
    run(&mut s, "edit.copy", json!({}));
    let pasted = run(&mut s, "edit.paste", json!({}));
    let ids: Vec<u64> = pasted["ids"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
    assert_eq!(ids.len(), 1);

    // The whole 3-point path must be copied, not just the last anchor!
    let pasted_node = s.doc().unwrap().doc.node(NodeId(ids[0])).unwrap();
    let path = pasted_node.path_data().unwrap();
    assert_eq!(path.subpaths[0].anchors.len(), 3);
}
