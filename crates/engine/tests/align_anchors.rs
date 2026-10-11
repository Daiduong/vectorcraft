//! Alignment of a partial path selection must leave every other anchor in place (#1180).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_doc::NodeId;
use vectorcraft_engine::Session;
use vectorcraft_geom::{PathData, Point};

fn triangle() -> (Session, NodeId) {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 400, "height": 240})).unwrap();
    let id = NodeId(s.execute("shape.rectangle", &json!({"x": 100, "y": 40, "width": 100, "height": 120})).unwrap()["id"].as_u64().unwrap());
    s.execute(
        "path.setAnchors",
        &json!({"id": id.0, "subpaths": [{"closed": true, "anchors": [
            {"x": 100, "y": 40, "out": [120, 45]}, {"x": 200, "y": 60, "in": [180, 55]}, {"x": 150, "y": 160}
        ]}]}),
    )
    .unwrap();
    s.execute("select.anchors", &json!({"id": id.0, "anchors": [[0, 0], [0, 1]]})).unwrap();
    (s, id)
}

fn path(s: &Session, id: NodeId) -> PathData {
    s.doc().unwrap().doc.node(id).unwrap().path_data().unwrap().clone()
}

#[test]
fn aligning_to_the_selection_uses_the_selected_points_and_preserves_handles() {
    for (direction, target) in [("top", 40.0), ("center", 50.0), ("bottom", 60.0)] {
        let (mut s, id) = triangle();
        let before = path(&s, id);
        let selection = s.doc().unwrap().selection.clone();
        s.execute("object.align", &json!({"vertical": direction, "to": "selection", "bounds": "preview"})).unwrap();
        let after = path(&s, id);
        for index in [0, 1] {
            let mut expected = before.subpaths[0].anchors[index];
            expected.translate(Point::new(expected.p.x, target) - expected.p);
            assert_eq!(after.subpaths[0].anchors[index], expected);
        }
        assert_eq!(after.subpaths[0].anchors[2], before.subpaths[0].anchors[2]);
        assert_eq!(s.doc().unwrap().selection, selection);
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(path(&s, id), before);
        s.execute("edit.redo", &json!({})).unwrap();
        assert_eq!(path(&s, id), after);
    }
}

#[test]
fn aligning_to_the_artboard_moves_only_selected_points_on_both_axes() {
    let (mut s, id) = triangle();
    let before = path(&s, id);
    let board = s.doc().unwrap().doc.artboards[0].rect;
    s.execute("object.align", &json!({"horizontal": "right", "vertical": "center", "to": "artboard", "artboard": 0})).unwrap();
    let after = path(&s, id);
    assert_eq!(after.subpaths[0].anchors[0].p, Point::new(board.x1, board.center().y));
    assert_eq!(after.subpaths[0].anchors[1].p, Point::new(board.x1, board.center().y));
    assert_eq!(after.subpaths[0].anchors[2], before.subpaths[0].anchors[2]);
}

#[test]
fn anchors_from_different_paths_align_together() {
    let (mut s, a) = triangle();
    let b = NodeId(s.execute("shape.rectangle", &json!({"x": 300, "y": 20, "width": 50, "height": 60})).unwrap()["id"].as_u64().unwrap());
    s.execute("select.anchorsMany", &json!({"items": [{"id": a.0, "anchors": [[0, 0]]}, {"id": b.0, "anchors": [[0, 0]]}]})).unwrap();
    let before_a = path(&s, a);
    let before_b = path(&s, b);
    s.execute("object.align", &json!({"horizontal": "center", "to": "selection"})).unwrap();
    let after_a = path(&s, a);
    let after_b = path(&s, b);
    assert_eq!(after_a.subpaths[0].anchors[0].p, Point::new(200.0, 40.0));
    assert_eq!(after_b.subpaths[0].anchors[0].p, Point::new(200.0, 20.0));
    assert_eq!(after_a.subpaths[0].anchors[1..], before_a.subpaths[0].anchors[1..]);
    assert_eq!(after_b.subpaths[0].anchors[1..], before_b.subpaths[0].anchors[1..]);
    // Editing a point expands a live shape, so a later shape edit cannot erase the alignment.
    assert!(matches!(&s.doc().unwrap().doc.node(b).unwrap().kind, vectorcraft_doc::NodeKind::Path { live: None, .. }));
}

#[test]
fn repeating_alignment_does_not_add_an_undo_step() {
    let (mut s, id) = triangle();
    let before = path(&s, id);
    let params = json!({"vertical": "top"});
    s.execute("object.align", &params).unwrap();
    s.execute("object.align", &params).unwrap();
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(path(&s, id), before);
}

#[test]
fn whole_object_alignment_still_translates_the_whole_path() {
    let (mut s, id) = triangle();
    s.execute("select.set", &json!({"ids": [id.0]})).unwrap();
    let mut expected = path(&s, id);
    let board = s.doc().unwrap().doc.artboards[0].rect;
    let bounds = expected.bounds().unwrap();
    expected.transform(vectorcraft_geom::Affine::translate((board.x0 - bounds.x0, 0.0)));
    s.execute("object.align", &json!({"horizontal": "left", "to": "artboard", "bounds": "geometric"})).unwrap();
    assert_eq!(path(&s, id), expected);
}
