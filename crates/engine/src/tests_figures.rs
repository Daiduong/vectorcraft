//! Heart, Arrow and Gear (#3865): `shape.heart`, `shape.arrow` and `shape.gear`.

use serde_json::{Value, json};
use vectorcraft_doc::NodeKind;
use vectorcraft_geom::{FillRule, PathData, Point, Rect};

use super::*;
use crate::tests_adjust::{fill_hex, node, session};

fn draw(s: &mut Session, cmd: &str, p: Value) -> NodeId {
    NodeId(s.execute(cmd, &p).unwrap()["id"].as_u64().unwrap())
}

fn path(s: &Session, id: NodeId) -> PathData {
    let NodeKind::Path { path, rule, .. } = &node(s, id).kind else { panic!("not a path") };
    assert_eq!(*rule, FillRule::NonZero);
    path.clone()
}

fn drawn(s: &mut Session, cmd: &str, p: Value) -> PathData {
    let id = draw(s, cmd, p);
    path(s, id)
}

fn same(a: Option<Rect>, b: Rect) -> bool {
    a.is_some_and(|a| [a.x0 - b.x0, a.y0 - b.y0, a.x1 - b.x1, a.y1 - b.y1].iter().all(|d| d.abs() < 1e-6))
}

/// The box every test draws in: 120 × 80 at (20, 30).
fn in_box(extra: Value) -> Value {
    let mut p = json!({"x": 20, "y": 30, "width": 120, "height": 80});
    if let (Some(o), Some(e)) = (p.as_object_mut(), extra.as_object()) {
        o.extend(e.clone());
    }
    p
}

const BOX: Rect = Rect::new(20.0, 30.0, 140.0, 110.0);

fn points(p: &PathData) -> Vec<Point> {
    p.anchors().map(|(_, _, a)| a.p).collect()
}

#[test]
fn each_figure_fills_its_box_as_one_undo_step_with_the_current_paint() {
    let mut s = session();
    s.execute("paint.setFill", &json!({"color": "#cc2244"})).unwrap();
    for (cmd, label) in [("shape.heart", "Heart"), ("shape.arrow", "Arrow"), ("shape.gear", "Gear")] {
        let undo = s.doc().unwrap().history.undo.len();
        let id = draw(&mut s, cmd, in_box(json!({})));
        let h = &s.doc().unwrap().history;
        assert_eq!(h.undo.len(), undo + 1, "{cmd}");
        assert_eq!(h.undo.last().unwrap().label, label);
        assert!(same(path(&s, id).bounds(), BOX), "{cmd}: {:?}", path(&s, id).bounds());
        assert_eq!(fill_hex(&node(&s, id)), "#cc2244", "{cmd}");
        assert_eq!(s.doc().unwrap().selection.objects, vec![id], "{cmd}: the new shape is selected");
        s.execute("edit.undo", &json!({})).unwrap();
        assert!(s.doc().unwrap().doc.node(id).is_none(), "{cmd}: undone");
    }
}

#[test]
fn a_box_drawn_from_its_far_corner_is_the_same_box() {
    let mut s = session();
    for cmd in ["shape.heart", "shape.arrow", "shape.gear"] {
        let id = draw(&mut s, cmd, json!({"x": 140, "y": 110, "width": -120, "height": -80}));
        assert!(same(path(&s, id).bounds(), BOX), "{cmd}");
    }
}

#[test]
fn arrow_takes_its_shaft_head_and_heads_in_percent() {
    let mut s = session();
    let one = drawn(&mut s, "shape.arrow", in_box(json!({"shaft": 50, "head": 25})));
    let pts = points(&one);
    assert_eq!(pts.len(), 7);
    assert!(pts.contains(&Point::new(140.0, 70.0)), "the tip, half way down the right side");
    assert!(pts.contains(&Point::new(20.0, 50.0)) && pts.contains(&Point::new(20.0, 90.0)), "the shaft half the height");
    assert!(pts.contains(&Point::new(110.0, 30.0)), "the head a quarter of the width");
    let two = drawn(&mut s, "shape.arrow", in_box(json!({"heads": 2})));
    let pts = points(&two);
    assert_eq!(pts.len(), 10);
    assert!(pts.contains(&Point::new(20.0, 70.0)) && pts.contains(&Point::new(140.0, 70.0)), "a tip at each end");
    // Defaults: shaft 40 %, head 35 %, one head.
    let plain = drawn(&mut s, "shape.arrow", in_box(json!({"shaft": null, "heads": null})));
    assert!(points(&plain).contains(&Point::new(20.0, 54.0)));
    assert!(s.execute("shape.arrow", &in_box(json!({"heads": 3}))).is_err(), "one or two heads");
    assert!(s.execute("shape.arrow", &in_box(json!({"heads": 0}))).is_err());
}

#[test]
fn gear_takes_its_teeth_depth_and_hole() {
    let mut s = session();
    let sq = json!({"x": 0, "y": 0, "width": 100, "height": 100});
    let g = drawn(&mut s, "shape.gear", with(&sq, json!({"teeth": 8, "depth": 30, "hole": 25})));
    assert_eq!(g.subpaths.len(), 2, "the outline and the hole");
    assert_eq!(g.subpaths[0].anchors.len(), 32, "four anchors a tooth");
    assert!(g.subpaths[0].area() > 0.0 && g.subpaths[1].area() < 0.0, "the hole stays open under non-zero");
    let c = Point::new(50.0, 50.0);
    for a in &g.subpaths[0].anchors {
        let d = a.p.distance(c);
        assert!((d - 35.0).abs() < 1e-9 || (d - 50.0).abs() < 1e-9, "rim at 70 % of the radius, tops at 100 %: {d}");
    }
    let hole = g.subpaths[1].anchors.iter().map(|a| a.p.distance(c)).fold(0.0, f64::max);
    assert!((hole - 12.5).abs() < 1e-9, "{hole}");
    // hole 0: no hole; out-of-range counts are clamped.
    let solid = drawn(&mut s, "shape.gear", with(&sq, json!({"hole": 0, "teeth": 1})));
    assert_eq!((solid.subpaths.len(), solid.subpaths[0].anchors.len()), (1, 12));
    // Defaults: 12 teeth, 20 % deep, a hole of 30 %.
    let d = drawn(&mut s, "shape.gear", sq.clone());
    assert_eq!((d.subpaths.len(), d.subpaths[0].anchors.len()), (2, 48));
    assert!(s.execute("shape.gear", &with(&sq, json!({"teeth": -4}))).is_err(), "a count, not a negative number");
}

fn with(base: &Value, extra: Value) -> Value {
    let mut p = base.clone();
    if let (Some(o), Some(e)) = (p.as_object_mut(), extra.as_object()) {
        o.extend(e.clone());
    }
    p
}

#[test]
fn rotation_turns_the_figure_about_its_centre_and_its_bounding_box_with_it() {
    let mut s = session();
    let id = draw(&mut s, "shape.heart", in_box(json!({"rotation": 90})));
    let n = node(&s, id);
    assert!((n.bbox_angle.rem_euclid(360.0) - 270.0).abs() < 1e-9 || (n.bbox_angle - 90.0).abs() < 1e-9, "{}", n.bbox_angle);
    // A quarter turn about (80, 70): the 120 × 80 box stands up as 80 × 120.
    assert!(same(path(&s, id).bounds(), Rect::new(40.0, 10.0, 120.0, 130.0)), "{:?}", path(&s, id).bounds());
    assert!(s.execute("shape.gear", &in_box(json!({"rotation": f64::NAN.to_string()}))).is_err());
}

#[test]
fn figures_draw_in_perspective_on_the_active_plane() {
    let mut s = session();
    s.execute("perspective.grid.preset", &json!({"kind": 2})).unwrap();
    s.execute("perspective.plane.set", &json!({"plane": "left"})).unwrap();
    for cmd in ["shape.heart", "shape.arrow", "shape.gear"] {
        let (rewritten, params) = crate::cmd::distortcmds::perspective_rewrite(&s, cmd, &in_box(json!({}))).unwrap();
        assert_eq!((rewritten.as_str(), &params["command"]), ("perspective.draw", &json!(cmd)));
        s.execute(&rewritten, &params).unwrap();
    }
}
