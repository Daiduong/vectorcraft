//! The tools' per-state cache ([`vectorcraft_tools::cache`]) never answers for another state:
//! through commands, drags, undo, redo and selection changes, every pointer, cursor and overlay
//! the selection tools give is what they give with the cache emptied first.

use serde_json::json;
use vectorcraft_geom::Point;
use vectorcraft_tools::{Cursor, Mods, Overlay, PointerEvent, PointerKind};

use super::*;

/// A page of 30 rectangles (live ones, which show corner widgets).
fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 600, "height": 400})).unwrap();
    for i in 0..30 {
        let (x, y) = ((i % 6) as f64 * 90.0 + 20.0, (i / 6) as f64 * 70.0 + 20.0);
        s.execute("shape.rectangle", &json!({"x": x, "y": y, "width": 50, "height": 30})).unwrap();
    }
    s
}

/// What the active tool shows at `p`: the cursor, then the overlays after a hover there.
fn look(s: &mut Session, p: Point, view: ViewInfo) -> (Cursor, Vec<Overlay>) {
    let _ = s.pointer(&PointerEvent { kind: PointerKind::Move, pos: p, mods: Mods::default(), pressure: 1.0 }, view);
    (s.cursor(p, Mods::default(), view), s.overlays(view))
}

#[test]
fn the_tool_cache_follows_every_change() {
    let mut s = session();
    let view = ViewInfo::default();
    let probes: Vec<Point> = (0..400).map(|i| Point::new((i % 40) as f64 * 15.0 + 1.0, (i / 40) as f64 * 40.0 + 2.0)).collect();
    let steps: Vec<(&str, serde_json::Value)> = vec![
        ("select.all", json!({})),
        ("object.move", json!({"dx": 7, "dy": 3})),
        ("edit.undo", json!({})),
        ("edit.redo", json!({})),
        ("object.scale", json!({"sx": 1.2, "sy": 0.9})),
        ("select.none", json!({})),
        ("select.all", json!({})),
        ("object.rotate", json!({"angle": 15})),
    ];
    for tool in ["selection", "directSelection"] {
        s.select_tool(tool, view).unwrap();
        for (cmd, params) in &steps {
            // A command the session doesn't take here changes nothing: the check holds anyway.
            let _ = s.execute(cmd, params);
            for p in &probes {
                let cached = look(&mut s, *p, view);
                s.tool_cache.clear();
                assert_eq!(cached, look(&mut s, *p, view), "{tool} after {cmd} at {p:?}");
            }
        }
        // A drag of the selection: every preview frame is a state of its own.
        let at = s.doc().unwrap().doc.bounds_of(&s.doc().unwrap().selection.objects, false).map(|b| b.center());
        if let Some(down) = at {
            let ev = |kind, dx: f64| PointerEvent { kind, pos: down + vectorcraft_geom::Vec2::new(dx, 0.0), mods: Mods::default(), pressure: 1.0 };
            s.pointer(&ev(PointerKind::Down, 0.0), view).unwrap();
            for k in 1..6 {
                s.pointer(&ev(PointerKind::Drag, 4.0 * k as f64), view).unwrap();
                let cached = (s.cursor(down, Mods::default(), view), s.overlays(view));
                s.tool_cache.clear();
                assert_eq!(cached, (s.cursor(down, Mods::default(), view), s.overlays(view)), "{tool} dragging");
            }
            s.pointer(&ev(PointerKind::Up, 20.0), view).unwrap();
        }
    }
}
