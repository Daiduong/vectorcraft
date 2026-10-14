//! Math drawing, the plane figure and general commands (`mathdraw.lengthMark` …
//! `mathdraw.clipRect`, `mathdraw.toBlack` … `mathdraw.label`). Labels need a font with a MATH
//! table: the checks of them run when there is one and are skipped otherwise (as in `tests_math`).

use super::*;
use serde_json::{Value, json};
use vectorcraft_color::Paint;
use vectorcraft_doc::{Node, NodeKind};
use vectorcraft_geom::{Point, Rect, SubPath};

const IDS: [&str; 13] = [
    "mathdraw.lengthMark",
    "mathdraw.angleMark",
    "mathdraw.sameMark",
    "mathdraw.parallelMark",
    "mathdraw.triangle",
    "mathdraw.dots",
    "mathdraw.circleFromSegment",
    "mathdraw.fractal",
    "mathdraw.clipRect",
    "mathdraw.toBlack",
    "mathdraw.recolor",
    "mathdraw.transform",
    "mathdraw.label",
];

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 600, "height": 400})).unwrap();
    s
}

fn run(s: &mut Session, id: &str, params: Value) -> Value {
    s.check_params(id, &params).unwrap();
    s.execute(id, &params).unwrap()
}

fn node(s: &Session, id: u64) -> &Node {
    s.doc().unwrap().doc.node(NodeId(id)).unwrap()
}

fn exists(s: &Session, id: u64) -> bool {
    s.doc().unwrap().doc.node(NodeId(id)).is_some()
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

fn bounds(n: &Node) -> Rect {
    n.geometric_bounds().unwrap()
}

/// The subpaths of a path or compound path.
fn subpaths(n: &Node) -> Vec<SubPath> {
    match &n.kind {
        NodeKind::Path { path, .. } => path.subpaths.clone(),
        NodeKind::Compound { children, .. } => children.iter().flat_map(|c| c.path_data().unwrap().subpaths.clone()).collect(),
        k => panic!("not a path: {k:?}"),
    }
}

fn ends(sp: &SubPath) -> (Point, Point) {
    (sp.anchors.first().unwrap().p, sp.anchors.last().unwrap().p)
}

fn rgb(p: &Paint) -> [f32; 3] {
    match p {
        Paint::Solid { color, .. } => color.to_rgb_uncalibrated(),
        p => panic!("{p:?}"),
    }
}

fn near(a: Point, b: Point, tol: f64) -> bool {
    (a - b).hypot() <= tol
}

fn assert_pt(a: Point, b: Point, what: &str) {
    assert!(near(a, b, 1e-6), "{what}: {a:?} vs {b:?}");
}

fn assert_rect(a: Rect, b: Rect, tol: f64, what: &str) {
    let d = [a.x0 - b.x0, a.y0 - b.y0, a.x1 - b.x1, a.y1 - b.y1];
    assert!(d.iter().all(|v| v.abs() <= tol), "{what}: {a:?} vs {b:?}");
}

fn line(s: &mut Session, x1: f64, y1: f64, x2: f64, y2: f64) -> u64 {
    s.execute("shape.line", &json!({"x1": x1, "y1": y1, "x2": x2, "y2": y2})).unwrap()["id"].as_u64().unwrap()
}

fn stroke_color(s: &mut Session, id: u64, color: &str) {
    s.execute("paint.setStroke", &json!({"color": color, "ids": [id]})).unwrap();
}

fn select(s: &mut Session, ids: &[u64]) {
    s.execute("select.set", &json!({"ids": ids})).unwrap();
}

fn last_undo(s: &Session) -> String {
    s.doc().unwrap().history.undo.last().unwrap().label.clone()
}

fn math_font() -> bool {
    crate::tests_math::math_font()
}

// ---------- registry and parameters ----------

#[test]
fn the_commands_are_registered_and_need_a_document() {
    for id in IDS {
        let c = find_command(id).unwrap();
        assert!(!c.label.is_empty() && c.journal, "{id}");
        assert!((c.enabled)(&Session::new()).is_err(), "{id} without a document");
        assert!((c.enabled)(&session()).is_ok(), "{id} with a document");
        assert!(crate::cmd::params::doc_keys(c.params).is_ok(), "{id} describes its keys");
        assert!(check_params(id, &json!({"colour": "#000000"})).is_err(), "{id} refuses a misspelt key");
    }
    check_params("mathdraw.lengthMark", &json!({"segments": [], "distance": 1, "gap": 1, "line": "solid1", "color": "#000000"})).unwrap();
    check_params("mathdraw.angleMark", &json!({"segments": [], "size": 1, "line": "solid1", "color": "#000000"})).unwrap();
    for id in ["mathdraw.sameMark", "mathdraw.parallelMark"] {
        check_params(id, &json!({"segments": [], "count": 1, "gap": 1, "len": 1, "angle": 1, "line": "solid1", "color": "#000000"})).unwrap();
    }
    check_params(
        "mathdraw.triangle",
        &json!({"mode": "SSS", "base": 1, "left": 1, "right": 1, "leftAngle": 1, "rightAngle": 1, "baseSegment": [0, 0, 1, 0], "rotation": 0, "unit": 1, "line": "solid1", "color": "#000000"}),
    )
    .unwrap();
    check_params("mathdraw.dots", &json!({"ids": [], "dot": 1, "color": "#000000"})).unwrap();
    check_params("mathdraw.circleFromSegment", &json!({"mode": "radius", "segments": [], "line": "solid1", "color": "#000000"})).unwrap();
    check_params("mathdraw.fractal", &json!({"levels": 1, "baseSegment": [], "placeSegments": [], "targetIds": []})).unwrap();
    check_params("mathdraw.clipRect", &json!({"rectId": 1, "ids": [], "keepRect": true})).unwrap();
    check_params("mathdraw.toBlack", &json!({"ids": [], "stroke": true, "fill": true})).unwrap();
    check_params("mathdraw.recolor", &json!({"color": "#000000", "ids": [], "stroke": true, "fill": true})).unwrap();
    check_params(
        "mathdraw.transform",
        &json!({"ids": [], "center": [0, 0], "scale": [1, 1], "shear": 1, "rotate": 1, "reflect": {"axisAngle": 0}, "keepTextSize": true, "copy": true}),
    )
    .unwrap();
    check_params(
        "mathdraw.label",
        &json!({"latex": "x", "at": [0, 0], "size": 9, "anchor": "center", "color": "#000000", "origin": [0, 0], "unit": 9, "makeReference": false, "artboard": 0}),
    )
    .unwrap();
    assert!(check_params("mathdraw.label", &json!({"line": "solid1"})).is_err(), "a label isn't stroked");
}

/// Wrong or missing parameters are refused, and nothing changes.
#[test]
fn bad_parameters_are_refused_and_change_nothing() {
    let mut s = session();
    let l = line(&mut s, 0.0, 0.0, 100.0, 0.0);
    s.execute("select.none", &json!({})).unwrap();
    let before = s.doc().unwrap().doc.clone();
    let undo = s.doc().unwrap().history.undo.len();
    let seg = json!([[0, 0, 100, 0]]);
    let cases = [
        ("mathdraw.lengthMark", json!({})),
        ("mathdraw.lengthMark", json!({"segments": [[0, 0, 0, 0]]})),
        ("mathdraw.lengthMark", json!({"segments": [[0, 0, 1]]})),
        ("mathdraw.lengthMark", json!({"segments": seg, "distance": 0})),
        ("mathdraw.lengthMark", json!({"segments": [[0, 0, 10, 0]], "gap": 40})),
        ("mathdraw.lengthMark", json!({"segments": seg, "line": "solid9"})),
        ("mathdraw.angleMark", json!({"segments": seg})),
        ("mathdraw.angleMark", json!({"segments": [[0, 0, 100, 0], [0, 10, 100, 10]]})),
        ("mathdraw.angleMark", json!({"segments": [[0, 0, 100, 0], [200, -10, 200, 10]]})),
        ("mathdraw.angleMark", json!({"segments": [[0, 0, 100, 0], [0, 0, 0, 100]], "size": -1})),
        ("mathdraw.sameMark", json!({"segments": seg, "len": 0})),
        ("mathdraw.sameMark", json!({"segments": seg, "angle": "x"})),
        ("mathdraw.sameMark", json!({"segments": seg, "count": -1})),
        ("mathdraw.parallelMark", json!({"segments": seg, "gap": -1})),
        ("mathdraw.triangle", json!({})),
        ("mathdraw.triangle", json!({"mode": "SSA", "base": 1})),
        ("mathdraw.triangle", json!({"mode": "SSS", "base": 1, "left": 1, "right": 3})),
        ("mathdraw.triangle", json!({"mode": "SSS", "base": 1, "left": 1})),
        ("mathdraw.triangle", json!({"mode": "SSS", "base": 1, "left": 1, "right": 1, "leftAngle": 30})),
        ("mathdraw.triangle", json!({"mode": "SAS", "base": 1, "left": 1, "leftAngle": 180})),
        ("mathdraw.triangle", json!({"mode": "ASA", "base": 1, "leftAngle": 100, "rightAngle": 80})),
        ("mathdraw.triangle", json!({"mode": "ASA", "base": 0, "leftAngle": 10, "rightAngle": 80})),
        ("mathdraw.dots", json!({"ids": [999_999]})),
        ("mathdraw.dots", json!({"dot": 0})),
        ("mathdraw.circleFromSegment", json!({"segments": seg})),
        ("mathdraw.circleFromSegment", json!({"segments": seg, "mode": "chord"})),
        ("mathdraw.fractal", json!({})),
        ("mathdraw.fractal", json!({"levels": 9, "baseSegment": [0, 0, 1, 0], "placeSegments": [[0, 0, 1, 1]], "targetIds": [l]})),
        ("mathdraw.fractal", json!({"levels": 1})),
        ("mathdraw.fractal", json!({"levels": 1, "baseSegment": [0, 0, 1, 0]})),
        ("mathdraw.fractal", json!({"levels": 1, "baseSegment": [0, 0, 1, 0], "placeSegments": [[0, 0, 1, 1]]})),
        ("mathdraw.fractal", json!({"levels": 1, "baseSegment": [0, 0, 1, 0], "placeSegments": [[0, 0, 1, 1]], "targetIds": [999_999]})),
        (
            "mathdraw.fractal",
            json!({"levels": 8, "baseSegment": [0, 0, 3, 0], "placeSegments": [[0, 0, 1, 0], [1, 0, 2, 0], [2, 0, 3, 0]], "targetIds": [l]}),
        ),
        ("mathdraw.clipRect", json!({})),
        ("mathdraw.clipRect", json!({"rectId": 999_999})),
        ("mathdraw.clipRect", json!({"rectId": l, "ids": [l]})),
        ("mathdraw.toBlack", json!({})),
        ("mathdraw.toBlack", json!({"ids": [l], "stroke": false, "fill": false})),
        ("mathdraw.recolor", json!({"ids": [l]})),
        ("mathdraw.recolor", json!({"ids": [l], "color": "nope"})),
        ("mathdraw.transform", json!({"ids": [l]})),
        ("mathdraw.transform", json!({"rotate": 10})),
        ("mathdraw.transform", json!({"ids": [l], "scale": [0, 1]})),
        ("mathdraw.transform", json!({"ids": [l], "shear": 90})),
        ("mathdraw.transform", json!({"ids": [l], "reflect": {"angle": 0}})),
        ("mathdraw.transform", json!({"ids": [l], "rotate": 10, "center": [1]})),
        ("mathdraw.label", json!({})),
        ("mathdraw.label", json!({"latex": "x"})),
        ("mathdraw.label", json!({"latex": "x", "at": [0, 0], "anchor": "middle"})),
        ("mathdraw.label", json!({"latex": "x", "at": [0, 0], "size": 0})),
    ];
    for (id, p) in cases {
        let e = s.execute(id, &p).unwrap_err();
        assert!(matches!(e, EngineError::BadParams { .. }), "{id} {p}: {e}");
    }
    assert_eq!(s.doc().unwrap().doc, before);
    assert_eq!(s.doc().unwrap().history.undo.len(), undo);
}

// ---------- marks ----------

#[test]
fn a_length_mark_is_two_arcs_beside_the_segment_with_a_gap() {
    let mut s = session();
    let undo = s.doc().unwrap().history.undo.len();
    let v = run(&mut s, "mathdraw.lengthMark", json!({"segments": [[100, 200, 300, 200], [0, 0, 30, 0]], "distance": 20, "gap": 40}));
    assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);
    assert_eq!(last_undo(&s), "Mark Segment Length");
    assert_eq!(v["skipped"], json!([1]), "the short segment");
    let [m] = ids_of(&v["marks"])[..] else { panic!("{v}") };
    assert_eq!(v["id"], json!(m), "one mark: no group");
    let n = node(&s, m);
    let sp = subpaths(n);
    assert_eq!(sp.len(), 2);
    // From each end out to 20 pt above (the segment's left, seen from its start), to the gap.
    assert_eq!(ends(&sp[0]), (Point::new(100.0, 200.0), Point::new(180.0, 180.0)));
    assert_eq!(ends(&sp[1]), (Point::new(300.0, 200.0), Point::new(220.0, 180.0)));
    assert_rect(bounds(n), Rect::new(100.0, 180.0, 300.0, 200.0), 1e-9, "bounds");
    assert_eq!(n.appearance.stroke().unwrap().width, vectorcraft_doc::MathDrawStyle::default().solid[3], "solid4");
    assert!(n.appearance.fill().is_none());
    // A negative distance: the other side; from the selection.
    let l = line(&mut s, 100.0, 300.0, 300.0, 300.0);
    select(&mut s, &[l]);
    let v = run(&mut s, "mathdraw.lengthMark", json!({"distance": -10}));
    let b = bounds(node(&s, id_of(&v["id"])));
    assert_rect(b, Rect::new(100.0, 300.0, 300.0, 310.0), 1e-9, "below");
}

#[test]
fn an_angle_mark_sits_where_segments_meet() {
    let mut s = session();
    // A corner: the right angle at (100, 200), between the segments.
    let v = run(&mut s, "mathdraw.angleMark", json!({"segments": [[100, 200, 300, 200], [100, 200, 100, 0]]}));
    assert_eq!(last_undo(&s), "Mark Angles");
    assert_eq!(v["vertices"], json!([[100.0, 200.0]]));
    let arc = node(&s, ids_of(&v["arcs"])[0]);
    let sp = subpaths(arc);
    for a in &sp[0].anchors {
        assert!(((a.p - Point::new(100.0, 200.0)).hypot() - 40.0).abs() < 1e-9, "radius 40: {a:?}");
    }
    assert_rect(bounds(arc), Rect::new(100.0, 160.0, 140.0, 200.0), 1e-6, "in the quarter between the segments");

    // A crossing: one arc, in the acute angle.
    let v = run(&mut s, "mathdraw.angleMark", json!({"segments": [[0, 100, 200, 100], [50, 0, 150, 200]], "size": 30}));
    assert_eq!(v["vertices"], json!([[100.0, 100.0]]));
    let sp = subpaths(node(&s, id_of(&v["id"])));
    let (a, b) = ends(&sp[0]);
    let c = Point::new(100.0, 100.0);
    let turn = ((a - c).cross(b - c)).atan2((a - c).dot(b - c)).abs().to_degrees();
    assert!((turn - 2.0_f64.atan().to_degrees()).abs() < 1e-6, "the acute angle: {turn}");

    // Three sides of a triangle: three corners; a pair that doesn't meet is passed over.
    let v = run(
        &mut s,
        "mathdraw.angleMark",
        json!({"segments": [[100, 300, 300, 300], [300, 300, 200, 200], [200, 200, 100, 300], [400, 0, 500, 0]], "size": 10}),
    );
    assert_eq!(ids_of(&v["arcs"]).len(), 3);
    assert_eq!(children(node(&s, id_of(&v["id"]))).len(), 3);
}

#[test]
fn equal_length_ticks_and_circle() {
    let mut s = session();
    let seg = json!([[100, 100, 300, 100]]);
    let v = run(&mut s, "mathdraw.sameMark", json!({"segments": seg, "count": 3, "gap": 10, "len": 20}));
    assert_eq!(last_undo(&s), "Mark Equal Lengths");
    let n = node(&s, id_of(&v["id"]));
    let sp = subpaths(n);
    assert_eq!(sp.len(), 3, "three ticks");
    for (t, x) in sp.iter().zip([190.0, 200.0, 210.0]) {
        assert_eq!(ends(t), (Point::new(x, 110.0), Point::new(x, 90.0)), "square across the middle");
    }
    assert_eq!(n.appearance.stroke().unwrap().width, vectorcraft_doc::MathDrawStyle::default().solid[2], "solid3");
    // Turned 90°: along the segment.
    let v = run(&mut s, "mathdraw.sameMark", json!({"segments": seg, "len": 20, "angle": 90}));
    let (a, b) = ends(&subpaths(node(&s, id_of(&v["id"])))[0]);
    assert!(near(a, Point::new(210.0, 100.0), 1e-9) && near(b, Point::new(190.0, 100.0), 1e-9), "{a:?} {b:?}");
    // None: a small circle of radius len on the middle.
    let v = run(&mut s, "mathdraw.sameMark", json!({"segments": seg, "count": 0, "len": 6}));
    assert_rect(bounds(node(&s, id_of(&v["id"]))), Rect::new(194.0, 94.0, 206.0, 106.0), 1e-9, "circle");
    // One mark per segment, grouped.
    let v = run(&mut s, "mathdraw.sameMark", json!({"segments": [[0, 0, 10, 0], [0, 50, 10, 50]], "count": 2}));
    assert_eq!(ids_of(&v["marks"]).len(), 2);
    assert_eq!(node(&s, id_of(&v["id"])).name.as_deref(), Some("Equal Length Marks"));
}

#[test]
fn parallel_chevrons_point_along_the_segment() {
    let mut s = session();
    let v = run(&mut s, "mathdraw.parallelMark", json!({"segments": [[100, 100, 300, 100]], "count": 2, "gap": 10, "len": 20}));
    assert_eq!(last_undo(&s), "Mark Parallel Segments");
    let sp = subpaths(node(&s, id_of(&v["id"])));
    assert_eq!(sp.len(), 2);
    let h = 10.0 * std::f64::consts::FRAC_1_SQRT_2;
    for (c, x) in sp.iter().zip([195.0, 205.0]) {
        let [a, apex, b] = c.anchors.iter().map(|a| a.p).collect::<Vec<_>>()[..] else { panic!("{c:?}") };
        assert!(!c.closed);
        assert!(near(apex, Point::new(x + h / 2.0, 100.0), 1e-9), "apex ahead, the chevron centred on {x}: {apex:?}");
        assert!(near(a, Point::new(apex.x - h, 100.0 - h), 1e-9) && near(b, Point::new(apex.x - h, 100.0 + h), 1e-9), "arms back at 45°");
    }
}

// ---------- constructions ----------

#[test]
fn a_3_4_5_triangle() {
    let mut s = session();
    // Without a segment: from the artboard's centre along +x, lengths in units of 50 pt.
    let v = run(&mut s, "mathdraw.triangle", json!({"mode": "SSS", "base": 3, "left": 4, "right": 5, "unit": 50}));
    assert_eq!(last_undo(&s), "Triangle by Sides and Angles");
    assert_eq!(v["vertices"], json!([[300.0, 200.0], [450.0, 200.0], [300.0, 0.0]]), "the right angle at the base's start");
    let n = node(&s, id_of(&v["id"]));
    let sp = subpaths(n);
    assert!(sp[0].closed && sp[0].anchors.len() == 3);
    assert_eq!(n.appearance.stroke().unwrap().width, 2.0, "solid3");

    // Turned 90°: the base goes up, the apex to its left.
    let v = run(&mut s, "mathdraw.triangle", json!({"mode": "SSS", "base": 3, "left": 4, "right": 5, "unit": 50, "rotation": 90}));
    let pts: Vec<Point> = v["vertices"].as_array().unwrap().iter().map(|p| Point::new(p[0].as_f64().unwrap(), p[1].as_f64().unwrap())).collect();
    assert_pt(pts[1], Point::new(300.0, 50.0), "base end");
    assert_pt(pts[2], Point::new(100.0, 200.0), "apex");

    // On a selected segment: it is the base, its length `base`.
    let l = line(&mut s, 100.0, 300.0, 400.0, 300.0);
    select(&mut s, &[l]);
    let v = run(&mut s, "mathdraw.triangle", json!({"mode": "SSS", "base": 3, "left": 4, "right": 5, "rotation": 45}));
    assert_eq!(v["vertices"], json!([[100.0, 300.0], [400.0, 300.0], [100.0, -100.0]]));

    // ASA 60°/60°: equilateral; SAS with a right angle.
    let v = run(&mut s, "mathdraw.triangle", json!({"mode": "ASA", "base": 2, "leftAngle": 60, "rightAngle": 60, "baseSegment": [0, 0, 100, 0]}));
    let apex = &v["vertices"][2];
    assert_pt(Point::new(apex[0].as_f64().unwrap(), apex[1].as_f64().unwrap()), Point::new(50.0, -50.0 * 3f64.sqrt()), "equilateral apex");
    let v = run(&mut s, "mathdraw.triangle", json!({"mode": "SAS", "base": 2, "left": 1, "leftAngle": 90, "baseSegment": [0, 0, 100, 0]}));
    let apex = &v["vertices"][2];
    assert_pt(Point::new(apex[0].as_f64().unwrap(), apex[1].as_f64().unwrap()), Point::new(0.0, -50.0), "SAS apex");
}

#[test]
fn dots_at_anchor_points_or_the_centre() {
    let mut s = session();
    let r = s.execute("shape.rectangle", &json!({"x": 100, "y": 100, "width": 50, "height": 20})).unwrap()["id"].as_u64().unwrap();
    let v = run(&mut s, "mathdraw.dots", json!({"ids": [r]}));
    assert_eq!(last_undo(&s), "Dots at Anchor Points");
    let mut pts: Vec<(i64, i64)> =
        v["points"].as_array().unwrap().iter().map(|p| (p[0].as_f64().unwrap() as i64, p[1].as_f64().unwrap() as i64)).collect();
    pts.sort();
    assert_eq!(pts, vec![(100, 100), (100, 120), (150, 100), (150, 120)]);
    let dots = ids_of(&v["dots"]);
    assert_eq!(dots.len(), 4);
    let d = node(&s, dots[0]);
    assert_eq!(bounds(d).width(), 14.0, "the style's dot size");
    assert_eq!((rgb(&d.appearance.fill().unwrap().paint), rgb(&d.appearance.stroke().unwrap().paint)), ([0.0; 3], [1.0; 3]));
    // Nothing given or selected: one dot at the centre of the artboard.
    s.execute("select.none", &json!({})).unwrap();
    let v = run(&mut s, "mathdraw.dots", json!({"dot": 6, "color": "#ff0000"}));
    let d = node(&s, id_of(&v["id"]));
    assert_rect(bounds(d), Rect::new(297.0, 197.0, 303.0, 203.0), 1e-9, "centre dot");
    assert_eq!(rgb(&d.appearance.fill().unwrap().paint), [1.0, 0.0, 0.0]);
}

#[test]
fn circles_from_segments() {
    let mut s = session();
    let seg = json!([[100, 100, 160, 180]]);
    let v = run(&mut s, "mathdraw.circleFromSegment", json!({"segments": seg, "mode": "radius"}));
    assert_eq!(last_undo(&s), "Circle from Segment");
    assert_rect(bounds(node(&s, id_of(&v["id"]))), Rect::new(0.0, 0.0, 200.0, 200.0), 1e-9, "radius 100 about the start");
    let v = run(&mut s, "mathdraw.circleFromSegment", json!({"segments": seg, "mode": "diameter"}));
    assert_rect(bounds(node(&s, id_of(&v["id"]))), Rect::new(80.0, 90.0, 180.0, 190.0), 1e-9, "on the middle");
    assert!(node(&s, id_of(&v["id"])).appearance.fill().is_none());
}

#[test]
fn a_fractal_copies_through_the_maps_level_by_level() {
    let mut s = session();
    let t = line(&mut s, 0.0, 0.0, 100.0, 0.0);
    let undo = s.doc().unwrap().history.undo.len();
    // Two halves of the base: level 1 is the target's two halves, level 2 its quarters.
    let p = json!({"levels": 2, "baseSegment": [0, 0, 100, 0], "placeSegments": [[0, 0, 50, 0], [50, 0, 100, 0]], "targetIds": [t]});
    let v = run(&mut s, "mathdraw.fractal", p);
    assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);
    assert_eq!(last_undo(&s), "Fractal Figure");
    assert_eq!((v["count"].clone(), v["levels"].clone()), (json!(6), json!([2, 4])));
    let g = node(&s, id_of(&v["id"]));
    assert_eq!(g.name.as_deref(), Some("Fractal"));
    let kids = children(g);
    assert_eq!(kids.len(), 6);
    assert_rect(bounds(kids[0]), Rect::new(0.0, 0.0, 50.0, 0.0), 1e-9, "level 1, first");
    assert_rect(bounds(kids[1]), Rect::new(50.0, 0.0, 100.0, 0.0), 1e-9, "level 1, second");
    assert_rect(bounds(kids[2]), Rect::new(0.0, 0.0, 25.0, 0.0), 1e-9, "level 2");
    assert!(exists(&s, t), "the original stays");
    assert_eq!(node(&s, kids[0].id.0).appearance.stroke().unwrap().width, node(&s, t).appearance.stroke().unwrap().width, "weights stay");

    // From the selection: the magenta base, a yellow place turning it up, the target.
    // (The target first: new lines take the stroke colour set last.)
    let target = line(&mut s, 0.0, 0.0, 50.0, 0.0);
    let base = line(&mut s, 0.0, 0.0, 100.0, 0.0);
    stroke_color(&mut s, base, "#ff00ff");
    let place = line(&mut s, 200.0, 200.0, 200.0, 100.0);
    stroke_color(&mut s, place, "#ffff00");
    select(&mut s, &[base, place, target]);
    let v = run(&mut s, "mathdraw.fractal", json!({"levels": 1}));
    assert_eq!(v["count"], json!(1));
    let copy = node(&s, id_of(&v["id"]));
    assert_rect(bounds(copy), Rect::new(200.0, 150.0, 200.0, 200.0), 1e-9, "turned up from the place's start");
}

#[test]
fn clip_rect_keeps_what_is_inside() {
    let mut s = session();
    let rect = s.execute("shape.rectangle", &json!({"x": 100, "y": 100, "width": 100, "height": 100})).unwrap()["id"].as_u64().unwrap();
    let across = line(&mut s, 50.0, 150.0, 250.0, 150.0);
    // A filled circle overlapping the corner, a line outside.
    let disc = s.execute("shape.ellipse", &json!({"x": 150, "y": 150, "width": 100, "height": 100})).unwrap()["id"].as_u64().unwrap();
    let outside = line(&mut s, 300.0, 300.0, 400.0, 300.0);
    // An unfilled circle half inside (stroked only): cut along its outline, an open arc.
    let ring = id_of(&run(&mut s, "mathdraw.circles", json!({"centers": [[4, -1]], "radius": 1, "origin": [0, 100], "unit": 50}))["id"]);
    select(&mut s, &[rect, across, disc, outside, ring]);
    let undo = s.doc().unwrap().history.undo.len();
    let v = run(&mut s, "mathdraw.clipRect", json!({}));
    assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);
    assert_eq!(last_undo(&s), "Keep Inside Rectangle");
    assert_eq!(v["removed"], json!([outside]));
    assert!(!exists(&s, outside) && !exists(&s, rect), "the outside line and the rectangle are gone");
    assert_eq!(ids_of(&v["clipped"]).len(), 3);
    assert_rect(bounds(node(&s, across)), Rect::new(100.0, 150.0, 200.0, 150.0), 1e-9, "the line");
    let b = bounds(node(&s, disc));
    // (Boolean results are refitted to the Pathfinder's precision.)
    assert_rect(b, Rect::new(150.0, 150.0, 200.0, 200.0), 1e-2, "the disc's quarter");
    let sp = subpaths(node(&s, ring));
    assert_eq!(sp.len(), 1);
    assert!(!sp[0].closed, "the ring is cut open");
    assert_rect(bounds(node(&s, ring)), Rect::new(150.0, 100.0, 200.0, 200.0), 1e-6, "the left half of the ring");
    assert!(!s.doc().unwrap().selection.objects.contains(&NodeId(outside)));

    // With `rectId` and `keepRect`; a group is cut inside, an equation or type is not a path.
    let rect = s.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap()["id"].as_u64().unwrap();
    let a = line(&mut s, -5.0, 5.0, 5.0, 5.0);
    let b = line(&mut s, 5.0, -5.0, 5.0, 5.0);
    select(&mut s, &[a, b]);
    let g = id_of(&s.execute("object.group", &json!({})).unwrap()["id"]);
    let v = run(&mut s, "mathdraw.clipRect", json!({"rectId": rect, "ids": [g], "keepRect": true}));
    assert_eq!(v["rect"], json!(rect));
    assert!(exists(&s, rect));
    assert_rect(bounds(node(&s, a)), Rect::new(0.0, 5.0, 5.0, 5.0), 1e-9, "inside the group");
    assert_rect(bounds(node(&s, b)), Rect::new(5.0, 0.0, 5.0, 5.0), 1e-9, "inside the group");
    assert_eq!(v["skipped"], json!([]));
}

// ---------- general ----------

#[test]
fn transform_about_a_given_centre() {
    let mut s = session();
    let l = line(&mut s, 300.0, 200.0, 400.0, 200.0);
    let w = node(&s, l).appearance.stroke().unwrap().width;
    let v = run(&mut s, "mathdraw.transform", json!({"ids": [l], "center": [300, 200], "rotate": 90}));
    assert_eq!(last_undo(&s), "Transform about Reference Point");
    assert_eq!(v, json!({"ids": [l], "center": [300.0, 200.0]}));
    assert_rect(bounds(node(&s, l)), Rect::new(300.0, 100.0, 300.0, 200.0), 1e-9, "turned up (counter-clockwise)");
    // Shear 45°: leaning right going up.
    run(&mut s, "mathdraw.transform", json!({"ids": [l], "center": [300, 200], "shear": 45}));
    assert_rect(bounds(node(&s, l)), Rect::new(300.0, 100.0, 400.0, 200.0), 1e-9, "sheared");
    // Scale, then reflect across the horizontal through the centre; weights stay.
    run(&mut s, "mathdraw.transform", json!({"ids": [l], "center": [300, 200], "scale": [2, 0.5], "reflect": {"axisAngle": 0}}));
    assert_rect(bounds(node(&s, l)), Rect::new(300.0, 200.0, 500.0, 250.0), 1e-9, "scaled and reflected down");
    assert_eq!(node(&s, l).appearance.stroke().unwrap().width, w);
    // Rotate, then reflect across the vertical: the order is scale, shear, rotate, reflect.
    let m = line(&mut s, 0.0, 0.0, 10.0, 0.0);
    run(&mut s, "mathdraw.transform", json!({"ids": [m], "center": [0, 0], "rotate": 90, "reflect": {"axisAngle": 90}, "scale": [3, 1]}));
    assert_rect(bounds(node(&s, m)), Rect::new(0.0, -30.0, 0.0, 0.0), 1e-9, "scaled, turned up, the vertical reflection keeps it");
    // A copy: the original stays.
    let v = run(&mut s, "mathdraw.transform", json!({"ids": [m], "center": [0, 0], "rotate": 180, "copy": true}));
    let [c] = ids_of(&v["ids"])[..] else { panic!("{v}") };
    assert_ne!(c, m);
    assert_rect(bounds(node(&s, c)), Rect::new(0.0, 0.0, 0.0, 30.0), 1e-9, "the copy");
    assert_rect(bounds(node(&s, m)), Rect::new(0.0, -30.0, 0.0, 0.0), 1e-9, "the original");
    assert_eq!(s.doc().unwrap().selection.objects, vec![NodeId(c)]);
}

#[test]
fn transform_about_a_magenta_dot_or_the_bounds() {
    let mut s = session();
    let l = line(&mut s, 300.0, 200.0, 400.0, 200.0);
    let dot = s.execute("shape.ellipse", &json!({"x": 195, "y": 195, "width": 10, "height": 10})).unwrap()["id"].as_u64().unwrap();
    s.execute("paint.setFill", &json!({"color": "#ff00ff", "ids": [dot]})).unwrap();
    select(&mut s, &[l, dot]);
    let v = run(&mut s, "mathdraw.transform", json!({"rotate": 180}));
    assert_eq!(v, json!({"ids": [l], "center": [200.0, 200.0]}));
    assert_rect(bounds(node(&s, l)), Rect::new(0.0, 200.0, 100.0, 200.0), 1e-9, "turned about the dot");
    assert_rect(bounds(node(&s, dot)), Rect::new(195.0, 195.0, 205.0, 205.0), 1e-9, "the dot stays");
    // With a copy, the copies and the dot end up selected, for the next turn.
    let v = run(&mut s, "mathdraw.transform", json!({"rotate": 90, "copy": true}));
    let c = ids_of(&v["ids"])[0];
    assert_eq!(s.doc().unwrap().selection.objects, vec![NodeId(c), NodeId(dot)]);
    // No dot selected: the centre of the objects' bounds.
    s.execute("select.none", &json!({})).unwrap();
    let v = run(&mut s, "mathdraw.transform", json!({"ids": [l], "scale": [2, 2]}));
    let c = &v["center"];
    assert_pt(Point::new(c[0].as_f64().unwrap(), c[1].as_f64().unwrap()), Point::new(50.0, 200.0), "the bounds' centre");
    assert_rect(bounds(node(&s, l)), Rect::new(-50.0, 200.0, 150.0, 200.0), 1e-9, "scaled about its middle");
}

#[test]
fn keep_text_size_only_moves_equations() {
    if !math_font() {
        return;
    }
    let mut s = session();
    let v = run(&mut s, "mathdraw.label", json!({"latex": "A", "at": [1, 0], "origin": [300, 200], "unit": 50}));
    let e = id_of(&v["id"]);
    let before = bounds(node(&s, e));
    run(&mut s, "mathdraw.transform", json!({"ids": [e], "center": [300, 200], "scale": [2, 2], "keepTextSize": true}));
    let after = bounds(node(&s, e));
    assert!((after.width() - before.width()).abs() < 1e-9 && (after.height() - before.height()).abs() < 1e-9, "same size");
    assert_pt(after.center(), Point::new(400.0, 200.0), "its centre went where the scale takes it");
}

#[test]
fn to_black_and_recolor_change_strokes_and_fills() {
    let mut s = session();
    let r = s.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap()["id"].as_u64().unwrap();
    stroke_color(&mut s, r, "#ff0000");
    s.execute("paint.setFill", &json!({"color": "#00ff00", "ids": [r]})).unwrap();
    let ring = id_of(
        &run(&mut s, "mathdraw.circles", json!({"centers": [[0, 0]], "radius": 1, "origin": [100, 100], "unit": 10, "color": "#0000ff"}))["id"],
    );
    let paints = |s: &Session, id: u64| {
        let n = node(s, id);
        (n.appearance.stroke().map(|x| rgb(&x.paint)), n.appearance.fill().map(|x| rgb(&x.paint)))
    };
    select(&mut s, &[r, ring]);
    let v = run(&mut s, "mathdraw.toBlack", json!({"stroke": false}));
    assert_eq!(last_undo(&s), "Convert to Black");
    assert_eq!(v["changed"], json!(1), "the rectangle's fill");
    assert_eq!(paints(&s, r), (Some([1.0, 0.0, 0.0]), Some([0.0; 3])));
    run(&mut s, "mathdraw.toBlack", json!({}));
    assert_eq!(paints(&s, r), (Some([0.0; 3]), Some([0.0; 3])));
    assert_eq!(paints(&s, ring), (Some([0.0; 3]), None), "no fill stays none");
    run(&mut s, "mathdraw.recolor", json!({"ids": [ring], "color": "#0000ff"}));
    assert_eq!(last_undo(&s), "Change Color Tone");
    assert_eq!(paints(&s, ring), (Some([0.0, 0.0, 1.0]), None));
    run(&mut s, "mathdraw.recolor", json!({"ids": [r], "color": "#00ffff", "fill": false}));
    assert_eq!(paints(&s, r), (Some([0.0, 1.0, 1.0]), Some([0.0; 3])));
}

#[test]
fn a_label_is_centred_on_its_point() {
    let mut s = session();
    let p = json!({"latex": "P", "at": [1, 1], "origin": [300, 200], "unit": 50});
    if !math_font() {
        assert!(s.execute("mathdraw.label", &p).is_err(), "no math font: an error, nothing drawn");
        return;
    }
    let v = run(&mut s, "mathdraw.label", p);
    assert_eq!(last_undo(&s), "Math Label");
    let n = node(&s, id_of(&v["id"]));
    assert_eq!(n.extras().math.as_deref().unwrap().source, "P");
    assert_eq!(n.extras().math.as_deref().unwrap().size, vectorcraft_doc::MathDrawStyle::default().label_size);
    assert_pt(bounds(n).center(), Point::new(350.0, 150.0), "centred");
    let v = run(&mut s, "mathdraw.label", json!({"latex": "P", "at": [1, 1], "origin": [300, 200], "unit": 50, "size": 20, "anchor": "top-left"}));
    let b = bounds(node(&s, id_of(&v["id"])));
    assert_pt(Point::new(b.x0, b.y0), Point::new(350.0, 150.0), "top left");
}
