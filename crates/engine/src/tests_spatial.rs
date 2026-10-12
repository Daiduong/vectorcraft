//! The bounds cache stays right through commands, drags, undo and redo: the cached bounds of every
//! object equal its bounds measured again.

use serde_json::json;
use vectorcraft_doc::{Document, Node, bounds_cache};
use vectorcraft_geom::Rect;
use vectorcraft_tools::{PointerEvent, PointerKind};

use super::*;

/// A page of 120 rectangles and ellipses.
fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    for i in 0..120 {
        let (x, y) = ((i % 12) as f64 * 60.0 + 10.0, (i / 12) as f64 * 55.0 + 10.0);
        let shape = if i % 2 == 0 { "shape.rectangle" } else { "shape.ellipse" };
        s.execute(shape, &json!({"x": x, "y": y, "width": 30, "height": 20})).unwrap();
    }
    s
}

fn bits(r: Option<Rect>) -> Option<[u64; 4]> {
    r.map(|r| [r.x0, r.y0, r.x1, r.y1].map(f64::to_bits))
}

/// Every object's cached bounds are its bounds.
fn check(d: &Document, what: &str) {
    fn walk(nodes: &[Arc<Node>], f: &mut dyn FnMut(&Arc<Node>)) {
        for n in nodes {
            f(n);
            if let Some(c) = n.children() {
                walk(c, f);
            }
        }
    }
    walk(&d.layers, &mut |n| {
        assert_eq!(bits(bounds_cache::geometric(n)), bits(n.geometric_bounds()), "{what}: geometric bounds of {}", n.id);
        assert_eq!(bits(bounds_cache::visual(n)), bits(n.visual_bounds()), "{what}: visual bounds of {}", n.id);
        assert_eq!(bits(bounds_cache::reach(n)), bits(n.reach_bounds()), "{what}: reach of {}", n.id);
    });
}

#[test]
fn bounds_follow_edits_drags_undo_and_redo() {
    let mut s = session();
    let doc = |s: &Session| s.doc().unwrap().doc.clone();
    check(&doc(&s), "built");
    s.execute("select.all", &json!({})).unwrap();
    s.execute("object.move", &json!({"dx": 5, "dy": 3})).unwrap();
    check(&doc(&s), "moved everything");
    let ids: Vec<u64> = s.doc().unwrap().selection.objects.iter().map(|id| id.0).collect();
    s.execute("select.set", &json!({"ids": ids[..6]})).unwrap();
    s.execute("object.group", &json!({})).unwrap();
    check(&doc(&s), "grouped");
    s.execute("object.move", &json!({"dx": -40, "dy": 10})).unwrap();
    check(&doc(&s), "moved the group");
    s.execute("select.set", &json!({"ids": [ids[50]]})).unwrap();
    s.execute("edit.clear", &json!({})).unwrap();
    check(&doc(&s), "deleted one below others");
    // A drag with the Selection tool: each preview frame, then the drop.
    s.execute("select.set", &json!({"ids": [ids[70]]})).unwrap();
    let at = s.doc().unwrap().doc.node(NodeId(ids[70])).unwrap().geometric_bounds().unwrap().center();
    let v = ViewInfo::default();
    s.select_tool("selection", v).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Down, at.x, at.y), v).unwrap();
    for k in 1..6 {
        s.pointer(&PointerEvent::new(PointerKind::Drag, at.x + 7.0 * k as f64, at.y + 3.0), v).unwrap();
        check(&doc(&s), &format!("drag frame {k}"));
    }
    s.pointer(&PointerEvent::new(PointerKind::Up, at.x + 35.0, at.y + 3.0), v).unwrap();
    check(&doc(&s), "dropped");
    for i in 0..4 {
        s.execute("edit.undo", &json!({})).unwrap();
        check(&doc(&s), &format!("undo {i}"));
    }
    for i in 0..4 {
        s.execute("edit.redo", &json!({})).unwrap();
        check(&doc(&s), &format!("redo {i}"));
    }
}
