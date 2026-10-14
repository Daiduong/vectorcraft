//! Math drawing, the space coordinate commands (`mathdraw.axes3d` … `mathdraw.circle3d`) and the
//! projection they draw in. Labels need a font with a MATH table: the checks of them run when
//! there is one and are skipped otherwise (as in `tests_math`).

use super::*;
use crate::cmd::mathdraw::space::{Frame3, Frame3Source, resolve_frame3};
use serde_json::{Value, json};
use vectorcraft_color::Paint;
use vectorcraft_doc::Node;
use vectorcraft_geom::{ParamCurve, PathData, Point, Rect, Vec2};

const IDS: [&str; 4] = ["mathdraw.axes3d", "mathdraw.points3d", "mathdraw.parametric3d", "mathdraw.circle3d"];

/// The projection the tests draw in: origin (300, 200), x toward the lower left, y right, z up.
const FRAME: Frame3 = Frame3 { origin: Point::new(300.0, 200.0), ex: Vec2::new(-32.0, 38.0), ey: Vec2::new(50.0, 0.0), ez: Vec2::new(0.0, -50.0) };

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 600, "height": 400})).unwrap();
    s
}

/// `params` with the test projection.
fn framed(mut params: Value) -> Value {
    params["origin"] = json!([300, 200]);
    params["ex"] = json!([-32, 38]);
    params["ey"] = json!([50, 0]);
    params["ez"] = json!([0, -50]);
    params
}

fn run(s: &mut Session, id: &str, params: Value) -> Value {
    s.check_params(id, &params).unwrap();
    s.execute(id, &params).unwrap()
}

fn node(s: &Session, id: u64) -> &Node {
    s.doc().unwrap().doc.node(NodeId(id)).unwrap()
}

fn id_of(v: &Value) -> u64 {
    v.as_u64().unwrap_or_else(|| panic!("not an id: {v}"))
}

fn ids_of(v: &Value) -> Vec<u64> {
    v.as_array().unwrap().iter().map(id_of).collect()
}

fn children(n: &Node) -> Vec<&Node> {
    n.children().unwrap().iter().map(|c| &**c).collect()
}

fn stroke_rgb(n: &Node) -> [f32; 3] {
    match &n.appearance.stroke().unwrap().paint {
        Paint::Solid { color, .. } => color.to_rgb_uncalibrated(),
        p => panic!("{p:?}"),
    }
}

fn bounds(n: &Node) -> Rect {
    n.geometric_bounds().unwrap()
}

fn path(n: &Node) -> &PathData {
    n.path_data().unwrap()
}

fn ends(n: &Node) -> (Point, Point) {
    let sp = &path(n).subpaths[0];
    (sp.anchors[0].p, sp.anchors.last().unwrap().p)
}

fn assert_near(a: f64, b: f64, tol: f64, what: &str) {
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b}");
}

fn assert_at(a: Point, b: Point, tol: f64, what: &str) {
    assert!((a - b).hypot() <= tol, "{what}: {a:?} vs {b:?}");
}

fn math_font() -> bool {
    crate::tests_math::math_font()
}

fn count(s: &Session) -> usize {
    let mut n = 0;
    for l in &s.doc().unwrap().doc.layers {
        l.walk(&mut |_| n += 1);
    }
    n
}

fn undo_label(s: &Session) -> String {
    s.doc().unwrap().history.undo.last().unwrap().label.clone()
}

/// A straight segment from `a` to `b` stroked `color`.
fn segment(s: &mut Session, a: (f64, f64), b: (f64, f64), color: &str) -> u64 {
    let id = s.execute("shape.line", &json!({"x1": a.0, "y1": a.1, "x2": b.0, "y2": b.1})).unwrap()["id"].as_u64().unwrap();
    s.execute("paint.setStroke", &json!({"color": color, "ids": [id]})).unwrap();
    id
}

/// The given objects grouped and the group selected: its id.
fn group(s: &mut Session, ids: &[u64]) -> u64 {
    s.execute("select.set", &json!({"ids": ids})).unwrap();
    let g = s.execute("object.group", &json!({})).unwrap()["id"].as_u64().unwrap();
    s.execute("select.set", &json!({"ids": [g]})).unwrap();
    g
}

// ---------- registry and parameters ----------

#[test]
fn the_commands_are_registered_and_need_a_document() {
    for id in IDS {
        let c = find_command(id).unwrap();
        assert!(!c.label.is_empty() && c.journal, "{id}");
        assert!((c.enabled)(&Session::new()).is_err(), "{id} without a document");
        assert!((c.enabled)(&session()).is_ok(), "{id} with a document");
        check_params(
            id,
            &json!({"origin": [0, 0], "ex": [1, 1], "ey": [1, 0], "ez": [0, 1], "unit": 10, "makeReference": false, "artboard": 0, "line": "solid1", "color": "#000000"}),
        )
        .unwrap();
        assert!(check_params(id, &json!({"orign": [0, 0]})).is_err(), "{id}");
        assert!(check_params(id, &json!({"ew": [0, 0]})).is_err(), "{id}");
    }
    check_params("mathdraw.axes3d", &json!({"lengths": [1, 2, 3], "negative": [0, 1, 0]})).unwrap();
    check_params("mathdraw.points3d", &json!({"coords": [], "guides": true, "planeGuides": true, "dot": 9})).unwrap();
    check_params("mathdraw.parametric3d", &json!({"x": "t", "y": "t", "z": "t", "tRange": [0, 1], "precision": 4})).unwrap();
    check_params("mathdraw.circle3d", &json!({"center": [0, 0, 0], "normal": [0, 0, 1], "radius": 1, "shadow": {}, "shadowLine": "dashed1"}))
        .unwrap();
}

/// Wrong or missing parameters, and projections that can't be drawn in, are refused, and nothing
/// changes.
#[test]
fn bad_parameters_are_refused_and_change_nothing() {
    let mut s = session();
    let before = s.doc().unwrap().doc.clone();
    let undo = s.doc().unwrap().history.undo.len();
    let cases = [
        ("mathdraw.axes3d", json!({"lengths": [0, 1, 1]})),
        ("mathdraw.axes3d", json!({"lengths": [1, 1]})),
        ("mathdraw.axes3d", json!({"negative": [-1, 0, 0]})),
        ("mathdraw.axes3d", json!({"lengths": [1, 1, 1e9]})),
        ("mathdraw.axes3d", json!({"ex": [0, 0]})),
        ("mathdraw.axes3d", json!({"ex": [1, 1], "ey": [2, 2], "ez": [-3, -3]})),
        ("mathdraw.axes3d", json!({"ez": [1e9, 0]})),
        ("mathdraw.axes3d", json!({"origin": [0, 0], "ex": [1, 0], "ey": [0, 1], "ez": [0, "1"]})),
        ("mathdraw.axes3d", json!({"unit": 0})),
        ("mathdraw.axes3d", json!({"artboard": 5})),
        ("mathdraw.points3d", json!({})),
        ("mathdraw.points3d", json!({"coords": []})),
        ("mathdraw.points3d", json!({"coords": [[1, 2]]})),
        ("mathdraw.points3d", json!({"coords": [[1, 2, 1e9]]})),
        ("mathdraw.points3d", json!({"coords": [[1, 2, 3]], "planeGuides": 1})),
        ("mathdraw.points3d", json!({"coords": [[1, 2, 3]], "dot": 0})),
        ("mathdraw.parametric3d", json!({"x": "t", "y": "t", "tRange": [0, 1]})),
        ("mathdraw.parametric3d", json!({"x": "t", "y": "t", "z": "x", "tRange": [0, 1]})),
        ("mathdraw.parametric3d", json!({"x": "t", "y": "t", "z": "t"})),
        ("mathdraw.parametric3d", json!({"x": "t", "y": "t", "z": "t", "tRange": [1, 1]})),
        ("mathdraw.parametric3d", json!({"x": "\\sqrt{t}", "y": "t", "z": "t", "tRange": [-2, -1]})),
        ("mathdraw.circle3d", json!({"normal": [0, 0, 1], "radius": 1})),
        ("mathdraw.circle3d", json!({"center": [0, 0, 0], "radius": 1})),
        ("mathdraw.circle3d", json!({"center": [0, 0, 0], "normal": [0, 0, 0], "radius": 1})),
        ("mathdraw.circle3d", json!({"center": [0, 0, 0], "normal": [0, 0, 1], "radius": 0})),
        ("mathdraw.circle3d", json!({"center": [0, 0, 0], "normal": [0, 0, 1]})),
        ("mathdraw.circle3d", json!({"center": [0, 0, 0], "normal": [0, 0, 1], "radius": 1, "shadow": {}})),
        ("mathdraw.circle3d", json!({"center": [0, 0, 0], "normal": [0, 0, 1], "radius": 1, "shadow": {"planeNormal": [0, 0, 0]}})),
        (
            "mathdraw.circle3d",
            json!({"center": [0, 0, 0], "normal": [0, 0, 1], "radius": 1, "shadow": {"planeNormal": [0, 0, 1], "lightDir": [1, 0, 0]}}),
        ),
        (
            "mathdraw.circle3d",
            json!({"center": [0, 0, 0], "normal": [0, 0, 1], "radius": 1, "shadow": {"planeNormal": [0, 0, 1], "light": [1, 0, 0]}}),
        ),
        ("mathdraw.circle3d", json!({"center": [0, 0, 0], "normal": [0, 0, 1], "radius": 1, "shadow": [0, 0, 1]})),
        ("mathdraw.circle3d", json!({"center": [0, 0, 0], "normal": [0, 0, 1], "radius": 1, "shadowLine": "dotted"})),
        // Off the canvas.
        ("mathdraw.circle3d", json!({"center": [0, 0, 0], "normal": [0, 0, 1], "radius": 1e6})),
    ];
    for (id, p) in cases {
        let e = s.execute(id, &p).unwrap_err();
        assert!(matches!(e, EngineError::BadParams { .. }), "{id} {p}: {e}");
    }
    assert_eq!(s.doc().unwrap().doc, before);
    assert_eq!(s.doc().unwrap().history.undo.len(), undo);
}

// ---------- the projection ----------

#[test]
fn a_projection_maps_space_onto_the_page() {
    assert_eq!(FRAME.to_doc([0.0, 0.0, 0.0]), Point::new(300.0, 200.0));
    assert_eq!(FRAME.to_doc([1.0, 2.0, 3.0]), Point::new(300.0 - 32.0 + 100.0, 200.0 + 38.0 - 150.0));
    assert_eq!(FRAME.vec([0.0, 0.0, -1.0]), Vec2::new(0.0, 50.0));
}

/// Nothing selected: the artboard's centre, the standard length and the textbook view, with a
/// reference object drawn and selected; the next command draws in it.
#[test]
fn the_default_projection_makes_a_reference_object() {
    let mut s = session();
    let plan = resolve_frame3(&s, &json!({}), "t").unwrap();
    assert_eq!(plan.source, Frame3Source::Artboard);
    let f = plan.frame;
    assert_eq!(f.origin, Point::new(300.0, 200.0));
    assert_eq!((f.ex, f.ey, f.ez), (Vec2::new(-0.64 * 150.0, 0.77 * 150.0), Vec2::new(150.0, 0.0), Vec2::new(0.0, -150.0)));
    let before = count(&s);
    let v = run(&mut s, "mathdraw.points3d", json!({"coords": [[1, 1, 1]], "guides": false}));
    let r = id_of(&v["reference"]);
    let g = node(&s, r);
    assert_eq!(g.name.as_deref(), Some("Space Reference"));
    let segs = children(g);
    assert_eq!(segs.len(), 3);
    for (seg, (rgb, axis)) in segs.iter().zip([([0.0, 1.0, 1.0], f.ex), ([1.0, 0.0, 1.0], f.ey), ([1.0, 1.0, 0.0], f.ez)]) {
        assert_eq!(stroke_rgb(seg), rgb);
        assert_eq!(ends(seg), (f.origin, f.origin + axis), "one unit from the origin");
    }
    assert_eq!(count(&s), before + 4 + 1, "the reference (a group of 3) and the dot");
    assert_eq!(s.doc().unwrap().selection.objects, vec![NodeId(r)], "the reference is selected");
    // The dot at (1, 1, 1).
    assert_eq!(bounds(node(&s, id_of(&v["id"]))).center(), f.to_doc([1.0, 1.0, 1.0]));
    // The next command finds the reference and draws no other.
    let again = run(&mut s, "mathdraw.points3d", json!({"coords": [[0, 0, 0]], "guides": false}));
    assert_eq!(again["reference"], v["reference"]);
    assert_eq!(count(&s), before + 4 + 2);
    // `makeReference: false` draws none; `unit` scales the default view.
    s.execute("select.none", &json!({})).unwrap();
    let v = run(&mut s, "mathdraw.points3d", json!({"coords": [[0, 1, 0]], "guides": false, "makeReference": false, "unit": 100}));
    assert!(v.get("reference").is_none());
    assert_eq!(bounds(node(&s, id_of(&v["id"]))).center(), Point::new(400.0, 200.0));
}

/// A selected group of three cyan, magenta and yellow segments meeting at a point: that point is
/// the origin whichever ends meet there, and each segment's other end one unit along its axis.
#[test]
fn the_reference_object_in_the_selection() {
    let mut s = session();
    let o = (100.0, 300.0);
    let x = segment(&mut s, o, (60.0, 350.0), "#00ffff");
    // Drawn toward the origin, and its end a little off it.
    let y = segment(&mut s, (180.0, 300.0), (100.2, 300.1), "#ff00ff");
    let z = segment(&mut s, o, (100.0, 220.0), "#ffff00");
    let g = group(&mut s, &[x, y, z]);
    let plan = resolve_frame3(&s, &json!({}), "t").unwrap();
    assert_eq!(plan.source, Frame3Source::Reference(NodeId(g)));
    let f = plan.frame;
    assert_at(f.origin, Point::new(100.0 + 0.2 / 3.0, 300.0 + 0.1 / 3.0), 1e-9, "origin");
    assert_at(f.origin + f.ex, Point::new(60.0, 350.0), 1e-9, "x");
    assert_at(f.origin + f.ey, Point::new(180.0, 300.0), 1e-9, "y");
    assert_at(f.origin + f.ez, Point::new(100.0, 220.0), 1e-9, "z");
    let before = count(&s);
    let v = run(&mut s, "mathdraw.points3d", json!({"coords": [[1, 0, 0], [0, 1, 0], [0, 0, 2]], "guides": false}));
    assert_eq!(id_of(&v["reference"]), g);
    assert_eq!(count(&s), before + 4, "the dots' group and three dots, no new reference");
    let dots = ids_of(&v["dots"]);
    assert_at(bounds(node(&s, dots[0])).center(), Point::new(60.0, 350.0), 1e-6, "dot on x");
    assert_at(bounds(node(&s, dots[2])).center(), f.to_doc([0.0, 0.0, 2.0]), 1e-6, "dot on z");
    // Parameters override parts of it.
    let plan = resolve_frame3(&s, &json!({"origin": [0, 0], "ez": [0, -10]}), "t").unwrap();
    assert_eq!(plan.source, Frame3Source::Reference(NodeId(g)));
    assert_eq!((plan.frame.origin, plan.frame.ez, plan.frame.ey), (Point::ZERO, Vec2::new(0.0, -10.0), f.ey));
    // All four given: the selection isn't looked at.
    let plan = resolve_frame3(&s, &framed(json!({})), "t").unwrap();
    assert_eq!((plan.source, plan.frame), (Frame3Source::Params, FRAME));
}

#[test]
fn a_group_that_is_not_a_reference_object_is_refused() {
    let mut s = session();
    // The three colours, but not meeting at one point.
    let x = segment(&mut s, (0.0, 0.0), (-40.0, 50.0), "#00ffff");
    let y = segment(&mut s, (0.0, 0.0), (80.0, 0.0), "#ff00ff");
    let z = segment(&mut s, (10.0, 10.0), (10.0, -80.0), "#ffff00");
    group(&mut s, &[x, y, z]);
    let e = s.execute("mathdraw.axes3d", &json!({})).unwrap_err();
    assert!(matches!(&e, EngineError::BadParams { msg, .. } if msg.contains("not a 3D reference object")), "{e}");
    // A fourth segment of one colour.
    let mut s = session();
    let x = segment(&mut s, (0.0, 0.0), (-40.0, 50.0), "#00ffff");
    let x2 = segment(&mut s, (0.0, 0.0), (-40.0, 60.0), "#00ffff");
    let y = segment(&mut s, (0.0, 0.0), (80.0, 0.0), "#ff00ff");
    let z = segment(&mut s, (0.0, 0.0), (0.0, -80.0), "#ffff00");
    group(&mut s, &[x, x2, y, z]);
    assert!(resolve_frame3(&s, &json!({}), "t").is_err());
    // A group without the three colours is no reference: the default projection.
    let mut s = session();
    let a = segment(&mut s, (0.0, 0.0), (-40.0, 50.0), "#000000");
    let b = segment(&mut s, (0.0, 0.0), (80.0, 0.0), "#ff00ff");
    group(&mut s, &[a, b]);
    assert_eq!(resolve_frame3(&s, &json!({}), "t").unwrap().source, Frame3Source::Artboard);
}

#[test]
fn cmyk_documents_draw_and_read_cmyk_references() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 600, "height": 400, "colorMode": "cmyk"})).unwrap();
    let v = run(&mut s, "mathdraw.axes3d", json!({}));
    let r = id_of(&v["reference"]);
    for (seg, cmyk) in children(node(&s, r)).iter().zip([[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]]) {
        match &seg.appearance.stroke().unwrap().paint {
            Paint::Solid { color, .. } => assert_eq!(*color, vectorcraft_color::Color::cmyk(cmyk[0], cmyk[1], cmyk[2], cmyk[3])),
            p => panic!("{p:?}"),
        }
    }
    let again = run(&mut s, "mathdraw.axes3d", json!({}));
    assert_eq!(again["reference"], v["reference"], "read back as the reference");
}

#[test]
fn degenerate_references_are_refused() {
    // Three segments meeting at a point, all along one line on the page.
    let mut s = session();
    let x = segment(&mut s, (0.0, 0.0), (-40.0, 0.0), "#00ffff");
    let y = segment(&mut s, (0.0, 0.0), (80.0, 0.0), "#ff00ff");
    let z = segment(&mut s, (0.0, 0.0), (30.0, 0.0), "#ffff00");
    group(&mut s, &[x, y, z]);
    let e = s.execute("mathdraw.axes3d", &json!({})).unwrap_err();
    assert!(matches!(&e, EngineError::BadParams { msg, .. } if msg.contains("one line")), "{e}");
}

// ---------- axes ----------

#[test]
fn space_axes_with_arrowheads_and_labels() {
    let mut s = session();
    let v = run(&mut s, "mathdraw.axes3d", framed(json!({"lengths": [2, 3, 4], "negative": [0, 1, 0]})));
    assert!(v.get("reference").is_none());
    let g = node(&s, id_of(&v["id"]));
    assert_eq!(g.name.as_deref(), Some("3D Axes"));
    assert_eq!(undo_label(&s), "Space Coordinate Axes");
    let (axes, arrows) = (ids_of(&v["axes"]), ids_of(&v["arrows"]));
    assert_eq!((axes.len(), arrows.len()), (3, 3));
    let style = s.doc().unwrap().doc.math_draw.clone();
    let tips = [FRAME.to_doc([2.0, 0.0, 0.0]), FRAME.to_doc([0.0, 3.0, 0.0]), FRAME.to_doc([0.0, 0.0, 4.0])];
    let froms = [FRAME.origin, FRAME.to_doc([0.0, -1.0, 0.0]), FRAME.origin];
    for k in 0..3 {
        let axis = node(&s, axes[k]);
        assert_eq!(axis.appearance.stroke().unwrap().width, style.stroke(style.axis_line).0, "the style's axis line");
        let (a, b) = ends(axis);
        assert_at(a, froms[k], 1e-9, "the axis starts at its negative end");
        // It stops short of the tip under the arrowhead, along the axis.
        assert!((b - a).cross(tips[k] - a).abs() <= 1e-6 * (b - a).hypot() * (tips[k] - a).hypot() && (b - a).hypot() < (tips[k] - a).hypot());
        let head = node(&s, arrows[k]);
        assert_eq!(path(head).subpaths[0].anchors[0].p, tips[k], "the arrowhead's tip");
        assert!(head.appearance.fill().is_some());
    }
    assert_eq!(bounds(node(&s, arrows[1])).x1, tips[1].x, "y points right");
    assert_eq!(bounds(node(&s, arrows[2])).y0, tips[2].y, "z points up");
    if !math_font() {
        assert_eq!(v["labels"], json!("skipped (no math font)"));
        assert_eq!(children(g).len(), 6);
        return;
    }
    let labels = ids_of(&v["labels"]);
    assert_eq!(children(g).len(), 10);
    let names: Vec<_> = labels.iter().map(|l| node(&s, *l).name.clone().unwrap()).collect();
    assert_eq!(names, ["x", "y", "z", "O"]);
    assert_eq!(node(&s, labels[0]).extras().math.as_deref().unwrap().source, "x", "italic");
    assert_eq!(node(&s, labels[3]).extras().math.as_deref().unwrap().source, "\\mathrm{O}");
    let b: Vec<Rect> = labels.iter().map(|l| bounds(node(&s, *l))).collect();
    // x (more vertical than horizontal) left of its tip, y (horizontal) under its tip, z left of
    // its tip.
    assert!(b[0].x1 < tips[0].x && b[0].y0 < tips[0].y && b[0].y1 > tips[0].y - 20.0, "{:?}", b[0]);
    assert!(b[1].y0 > tips[1].y && b[1].x0 < tips[1].x && b[1].x1 > tips[1].x - 30.0, "{:?}", b[1]);
    assert!(b[2].x1 < tips[2].x && b[2].y0 < tips[2].y + 20.0 && b[2].y1 > tips[2].y - 20.0, "{:?}", b[2]);
    // O in the widest gap between the rays from the origin (z up 270°, y right 0° and left 180°,
    // x lower left 130°): between y and x, lower right, clear of both.
    let angle = |b: Rect| {
        let d = b.center() - FRAME.origin;
        d.y.atan2(d.x).to_degrees()
    };
    assert!((20.0..110.0).contains(&angle(b[3])), "{:?}", b[3]);
    // Without the negative y axis the widest gap is between x and z: left of the origin.
    let v = run(&mut s, "mathdraw.axes3d", framed(json!({})));
    let o = bounds(node(&s, ids_of(&v["labels"])[3]));
    assert!(angle(o).abs() > 150.0, "{o:?}");
    assert!(o.x1 < FRAME.origin.x, "{o:?}");
}

// ---------- points ----------

#[test]
fn points_in_space_with_guides() {
    let mut s = session();
    let q = [1.0, 2.0, 3.0];
    let v = run(&mut s, "mathdraw.points3d", framed(json!({"coords": [q]})));
    assert_eq!(undo_label(&s), "Points by Space Coordinates");
    assert_eq!(node(&s, id_of(&v["id"])).name.as_deref(), Some("3D Points"));
    let dots = ids_of(&v["dots"]);
    assert_at(bounds(node(&s, dots[0])).center(), FRAME.to_doc(q), 1e-9, "the dot");
    let guides: Vec<(Point, Point)> = ids_of(&v["guides"]).iter().map(|g| ends(node(&s, *g))).collect();
    let p = |c: [f64; 3]| FRAME.to_doc(c);
    assert_eq!(guides, vec![(p(q), p([1.0, 2.0, 0.0])), (p([1.0, 2.0, 0.0]), p([1.0, 0.0, 0.0])), (p([1.0, 2.0, 0.0]), p([0.0, 2.0, 0.0]))]);
    // Down to the xy plane is along z.
    assert_eq!((guides[0].1 - guides[0].0).cross(FRAME.ez), 0.0);
    for g in ids_of(&v["guides"]) {
        let st = node(&s, g).appearance.stroke().unwrap().clone();
        assert!(st.dash.is_some(), "dashed");
    }
    // With the planes: the nine edges of the box off the axes.
    let v = run(&mut s, "mathdraw.points3d", framed(json!({"coords": [q], "planeGuides": true})));
    let guides: Vec<(Point, Point)> = ids_of(&v["guides"]).iter().map(|g| ends(node(&s, *g))).collect();
    assert_eq!(guides.len(), 9);
    for (a, b) in [(q, [0.0, 2.0, 3.0]), (q, [1.0, 0.0, 3.0]), ([0.0, 2.0, 3.0], [0.0, 0.0, 3.0]), ([1.0, 0.0, 3.0], [1.0, 0.0, 0.0])] {
        assert!(guides.contains(&(p(a), p(b))), "{a:?} → {b:?}");
    }
    // A point on the xy plane: no drop, and each guide once.
    let v = run(&mut s, "mathdraw.points3d", framed(json!({"coords": [[1, 2, 0]], "planeGuides": true})));
    assert_eq!(ids_of(&v["guides"]).len(), 4);
    let v = run(&mut s, "mathdraw.points3d", framed(json!({"coords": [[1, 2, 0]]})));
    assert_eq!(ids_of(&v["guides"]).len(), 2);
    // No guides: the dots alone.
    let v = run(&mut s, "mathdraw.points3d", framed(json!({"coords": [q, [0, 1, 1]], "guides": false, "dot": 6})));
    assert!(ids_of(&v["guides"]).is_empty());
    let dots = ids_of(&v["dots"]);
    assert_eq!(dots.len(), 2);
    assert_near(bounds(node(&s, dots[1])).width(), 6.0, 1e-9, "dot size");
}

// ---------- curves ----------

/// The helix (cos t, sin t, t/10): the curve passes through each step's point exactly and between
/// them stays within a hair of the projected curve.
#[test]
fn a_helix_in_space() {
    let mut s = session();
    let tau = std::f64::consts::TAU;
    let v = run(&mut s, "mathdraw.parametric3d", framed(json!({"x": "\\cos t", "y": "\\sin t", "z": "0.1t", "tRange": [0, 2.0 * tau]})));
    assert_eq!(undo_label(&s), "Space Parametric Curve");
    assert_eq!(v["id"], v["curve"], "one curve, not grouped");
    let n = node(&s, id_of(&v["curve"]));
    assert_eq!(n.appearance.stroke().unwrap().width, s.doc().unwrap().doc.math_draw.stroke(vectorcraft_doc::LineRef::Solid(3)).0);
    let helix = |t: f64| FRAME.to_doc([t.cos(), t.sin(), 0.1 * t]);
    let sp = &path(n).subpaths[0];
    assert_eq!(sp.anchors.len(), 41, "precision 40 by default");
    for (i, a) in sp.anchors.iter().enumerate() {
        assert_at(a.p, helix(2.0 * tau * i as f64 / 40.0), 1e-9, &format!("anchor {i}"));
    }
    for i in 0..sp.segment_count() {
        let t = 2.0 * tau * (i as f64 + 0.5) / 40.0;
        assert_at(sp.segment(i).eval(0.5), helix(t), 0.05, &format!("middle of step {i}"));
    }
    // The bounds of the projected helix.
    let pts: Vec<Point> = (0..=4000).map(|i| helix(2.0 * tau * i as f64 / 4000.0)).collect();
    let want = pts.iter().fold(Rect::from_points(pts[0], pts[0]), |r, p| r.union_pt(*p));
    let got = bounds(n);
    for (a, b) in [(got.x0, want.x0), (got.y0, want.y0), (got.x1, want.x1), (got.y1, want.y1)] {
        assert_near(a, b, 0.05, "bounds");
    }
}

/// A circle in space: its projection is the ellipse `centre + A cos t + B sin t` (A, B the page
/// vectors of two perpendicular radii), whose bounds don't depend on which radii.
#[test]
fn a_circle_in_space_is_an_ellipse() {
    let mut s = session();
    let (center, normal, r) = ([1.0, 1.0, 1.0], [1.0, 2.0, 2.0], 1.5);
    let v = run(&mut s, "mathdraw.circle3d", framed(json!({"center": center, "normal": normal, "radius": r})));
    assert_eq!(undo_label(&s), "Space Circle");
    assert!(v.get("shadow").is_none());
    let n = node(&s, id_of(&v["circle"]));
    let sp = &path(n).subpaths[0];
    assert!(sp.closed);
    assert_eq!(sp.anchors.len(), 16);
    // Two perpendicular radii: (2, -1, 0)/√5 and normal × that.
    let k = 1.0 / 3.0;
    let nn = [normal[0] * k, normal[1] * k, normal[2] * k];
    let u = [2.0 / 5f64.sqrt(), -1.0 / 5f64.sqrt(), 0.0];
    let w = [nn[1] * u[2] - nn[2] * u[1], nn[2] * u[0] - nn[0] * u[2], nn[0] * u[1] - nn[1] * u[0]];
    let (a, b) = (FRAME.vec(u) * r, FRAME.vec(w) * r);
    let c = FRAME.to_doc(center);
    let (hw, hh) = ((a.x * a.x + b.x * b.x).sqrt(), (a.y * a.y + b.y * b.y).sqrt());
    let got = bounds(n);
    for (g, want) in [(got.x0, c.x - hw), (got.x1, c.x + hw), (got.y0, c.y - hh), (got.y1, c.y + hh)] {
        assert_near(g, want, 0.02, "the ellipse's bounds");
    }
    // Every anchor is on the circle in space: page point = c + A cos + B sin for some angle.
    let det = a.cross(b);
    for an in &sp.anchors {
        let d = an.p - c;
        let (cos, sin) = (d.cross(b) / det, a.cross(d) / det);
        assert_near(cos * cos + sin * sin, 1.0, 1e-9, "on the circle");
    }
}

/// The shadow on the xy plane cast by light along z is the circle with z dropped.
#[test]
fn a_circle_s_shadow_on_the_xy_plane() {
    let mut s = session();
    let (center, normal, r) = ([0.5, 1.0, 2.0], [1.0, 0.0, 1.0], 1.0);
    let v = run(
        &mut s,
        "mathdraw.circle3d",
        framed(json!({"center": center, "normal": normal, "radius": r, "shadow": {"planeNormal": [0, 0, 1], "lightDir": [0, 0, 1]}})),
    );
    let g = node(&s, id_of(&v["id"]));
    assert_eq!(g.name.as_deref(), Some("3D Circle"));
    assert_eq!(children(g).len(), 2);
    let (circle, shadow) = (node(&s, id_of(&v["circle"])), node(&s, id_of(&v["shadow"])));
    assert!(shadow.appearance.stroke().unwrap().dash.is_some(), "the shadow is dashed");
    assert!(circle.appearance.stroke().unwrap().dash.is_none());
    let (ca, sa) = (path(circle).subpaths[0].anchors.clone(), path(shadow).subpaths[0].anchors.clone());
    assert_eq!((ca.len(), sa.len()), (16, 16));
    // Page → (x, y) on the xy plane.
    let det = FRAME.ex.cross(FRAME.ey);
    for (c, sh) in ca.iter().zip(&sa) {
        let d = sh.p - FRAME.origin;
        let (x, y) = (d.cross(FRAME.ey) / det, FRAME.ex.cross(d) / det);
        // The circle's point is straight above it along z.
        let up = c.p - sh.p;
        assert_near(up.cross(FRAME.ez), 0.0, 1e-6, "above along z");
        let z = up.dot(FRAME.ez) / FRAME.ez.hypot2();
        let q = [x - center[0], y - center[1], z - center[2]];
        assert_near((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt(), r, 1e-9, "on the circle in space");
        assert_near(q[0] * normal[0] + q[1] * normal[1] + q[2] * normal[2], 0.0, 1e-9, "in its plane");
    }
    // The default light is along the plane's normal, and the plane point shifts the plane.
    let v2 = run(
        &mut s,
        "mathdraw.circle3d",
        framed(
            json!({"center": center, "normal": normal, "radius": r, "shadow": {"planeNormal": [0, 0, 2], "planePoint": [0, 0, 1]}, "shadowLine": "solid5"}),
        ),
    );
    let sh2 = node(&s, id_of(&v2["shadow"]));
    assert!(sh2.appearance.stroke().unwrap().dash.is_none());
    for (a, b) in path(sh2).subpaths[0].anchors.iter().zip(&sa) {
        assert_at(a.p, b.p + FRAME.ez, 1e-9, "the plane z = 1");
    }
}

/// Each command is one undo step that undoes everything, its reference object too.
#[test]
fn each_command_undoes_in_one_step() {
    let cases = [
        ("mathdraw.axes3d", json!({})),
        ("mathdraw.points3d", json!({"coords": [[1, 1, 1]], "planeGuides": true})),
        ("mathdraw.parametric3d", json!({"x": "t", "y": "t^2", "z": "1", "tRange": [0, 1]})),
        (
            "mathdraw.circle3d",
            json!({"center": [0, 0, 0], "normal": [0, 0, 1], "radius": 1, "shadow": {"planeNormal": [0, 1, 0], "lightDir": [1, 1, 1]}}),
        ),
    ];
    for (id, p) in cases {
        let mut s = session();
        let before = s.doc().unwrap().doc.clone();
        let v = run(&mut s, id, p);
        assert!(v.get("reference").is_some(), "{id}");
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(s.doc().unwrap().doc, before, "{id}");
    }
}
