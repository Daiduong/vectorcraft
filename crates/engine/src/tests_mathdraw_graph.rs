//! Math drawing, the graph commands (`mathdraw.axes` … `mathdraw.piecewise`) and the reading of
//! paths as functions they share. Labels need a font with a MATH table: the checks of them run
//! when there is one and are skipped otherwise (as in `tests_math`).

use super::*;
use crate::cmd::mathdraw::{Extremum, Frame, PathFunction};
use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Node, NodeKind};
use vectorcraft_geom::{PathData, Point, Rect};

const IDS: [&str; 10] = [
    "mathdraw.axes",
    "mathdraw.points",
    "mathdraw.graph",
    "mathdraw.parametric",
    "mathdraw.circles",
    "mathdraw.extrema",
    "mathdraw.integral",
    "mathdraw.derivative",
    "mathdraw.combine",
    "mathdraw.piecewise",
];

/// The frame the tests draw in: origin (300, 200), 50 pt per unit.
const FRAME: Frame = Frame { origin: Point::new(300.0, 200.0), unit: 50.0 };

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 600, "height": 400})).unwrap();
    s
}

/// `params` with the test frame.
fn framed(mut params: Value) -> Value {
    params["origin"] = json!([300, 200]);
    params["unit"] = json!(50);
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

fn fill_rgb(n: &Node) -> [f32; 3] {
    match &n.appearance.fill().unwrap().paint {
        Paint::Solid { color, .. } => color.to_rgb_uncalibrated(),
        p => panic!("{p:?}"),
    }
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

/// The curve node `id` read back as y(x) in the test frame.
fn function(s: &Session, id: u64) -> PathFunction {
    PathFunction::from_node(node(s, id), &FRAME).unwrap()
}

fn assert_near(a: f64, b: f64, tol: f64, what: &str) {
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b}");
}

fn math_font() -> bool {
    crate::tests_math::math_font()
}

/// A curve drawn by `mathdraw.graph` (no axes, no label): its id.
fn graph(s: &mut Session, f: &str, range: [f64; 2], more: Value) -> u64 {
    let mut p = framed(json!({"f": f, "xRange": range}));
    for (k, v) in more.as_object().unwrap() {
        p[k] = v.clone();
    }
    id_of(&run(s, "mathdraw.graph", p)["curve"])
}

// ---------- registry and parameters ----------

#[test]
fn the_commands_are_registered_and_need_a_document() {
    for id in IDS {
        let c = find_command(id).unwrap();
        assert!(!c.label.is_empty() && c.journal, "{id}");
        assert!((c.enabled)(&Session::new()).is_err(), "{id} without a document");
        assert!((c.enabled)(&session()).is_ok(), "{id} with a document");
        // The frame parameters every one takes, and a misspelt key refused.
        check_params(id, &json!({"origin": [0, 0], "unit": 10, "makeReference": false, "artboard": 0, "line": "solid1", "color": "#000000"}))
            .unwrap();
        assert!(check_params(id, &json!({"orign": [0, 0]})).is_err(), "{id}");
        let keys = crate::cmd::params::doc_keys(c.params).unwrap().keys;
        for k in ["origin", "unit", "makeReference", "artboard", "line", "color"] {
            assert!(keys.contains(&k), "{id} names {k}");
        }
    }
    check_params(
        "mathdraw.graph",
        &json!({"f": "x", "xRange": [0, 1], "precision": 4, "yClip": [-1, 1], "axes": true, "label": true, "labelAt": "end"}),
    )
    .unwrap();
    check_params("mathdraw.piecewise", &json!({"pieces": [], "axes": true, "yClip": [0, 1]})).unwrap();
    check_params("mathdraw.combine", &json!({"expr": "f", "ids": {}, "precision": 9})).unwrap();
    check_params("mathdraw.integral", &json!({"id": 1, "precision": 9, "c0At": "left"})).unwrap();
}

/// Wrong or missing parameters are refused, and nothing changes.
#[test]
fn bad_parameters_are_refused_and_change_nothing() {
    let mut s = session();
    let before = s.doc().unwrap().doc.clone();
    let undo = s.doc().unwrap().history.undo.len();
    let cases = [
        ("mathdraw.axes", json!({"xRange": [2, 1]})),
        ("mathdraw.axes", json!({"xRange": [0, "1"]})),
        ("mathdraw.axes", json!({"yRange": [0, 1e9]})),
        ("mathdraw.axes", json!({"fitSelection": 1})),
        ("mathdraw.axes", json!({"line": "solid9"})),
        ("mathdraw.points", json!({})),
        ("mathdraw.points", json!({"coords": []})),
        ("mathdraw.points", json!({"coords": [[1]]})),
        ("mathdraw.points", json!({"coords": [[1, 2]], "dot": -1})),
        ("mathdraw.points", json!({"coords": [[1, 2]], "guides": "no"})),
        ("mathdraw.graph", json!({"xRange": [0, 1]})),
        ("mathdraw.graph", json!({"f": "x", "xRange": [1, 1]})),
        ("mathdraw.graph", json!({"f": "y", "xRange": [0, 1]})),
        ("mathdraw.graph", json!({"f": "\\sin", "xRange": [0, 1]})),
        ("mathdraw.graph", json!({"f": "x", "xRange": [0, 1], "precision": -1})),
        ("mathdraw.graph", json!({"f": "x", "xRange": [0, 1], "labelAt": "start"})),
        ("mathdraw.graph", json!({"f": "100", "xRange": [0, 1]})),
        ("mathdraw.graph", json!({"f": "\\sqrt{x}", "xRange": [-2, -1]})),
        ("mathdraw.parametric", json!({"x": "t", "y": "x", "tRange": [0, 1]})),
        ("mathdraw.parametric", json!({"x": "t", "y": "t"})),
        ("mathdraw.circles", json!({"centers": [[0, 0]]})),
        ("mathdraw.circles", json!({"centers": [[0, 0]], "radius": 0})),
        ("mathdraw.extrema", json!({})),
        ("mathdraw.extrema", json!({"id": 999_999})),
        ("mathdraw.integral", json!({})),
        ("mathdraw.integral", json!({"c0At": "middle"})),
        ("mathdraw.derivative", json!({"id": "1"})),
        ("mathdraw.combine", json!({"expr": "x"})),
        ("mathdraw.combine", json!({"expr": "f+g"})),
        ("mathdraw.combine", json!({"expr": "f", "ids": {"k": 1}})),
        ("mathdraw.piecewise", json!({"pieces": []})),
        ("mathdraw.piecewise", json!({"pieces": [{"f": "x"}]})),
        ("mathdraw.piecewise", json!({"pieces": [{"f": "x", "range": [0, 1], "leftEnd": "half"}]})),
        ("mathdraw.piecewise", json!({"pieces": [{"f": "x", "range": [0, 1], "leftEnd": {"value": "1"}}]})),
        ("mathdraw.piecewise", json!({"pieces": [{"f": "x", "range": [0, 1], "colour": 1}]})),
        ("mathdraw.piecewise", json!({"pieces": vec![json!({"f": "x", "range": [0, 1]}); 9]})),
        ("mathdraw.graph", json!({"f": "x", "xRange": [0, 100], "yClip": [-1000, 1000], "unit": 1e5, "origin": [0, 0]})),
    ];
    for (id, p) in cases {
        let e = s.execute(id, &p).unwrap_err();
        assert!(matches!(e, EngineError::BadParams { .. }), "{id} {p}: {e}");
    }
    assert_eq!(s.doc().unwrap().doc, before);
    assert_eq!(s.doc().unwrap().history.undo.len(), undo);
}

// ---------- reading paths as functions ----------

#[test]
fn a_fitted_graph_reads_back_exactly() {
    let mut s = session();
    let id = graph(&mut s, "x^3-x", [-2.0, 2.0], json!({"precision": 8}));
    let f = function(&s, id);
    assert_eq!(f.range(), (-2.0, 2.0));
    for x in [-2.0, -1.3, -0.25, 0.0, 0.6, 1.0, 1.99, 2.0] {
        assert_near(f.at(x), x * x * x - x, 1e-9, &format!("f({x})"));
    }
    assert!(f.at(2.5).is_nan() && f.at(-2.1).is_nan(), "no value outside the curve");
}

#[test]
fn a_path_drawn_right_to_left_or_split_at_a_pole_reads_as_a_function() {
    // Right to left: the line y = x/2 from (2, 1) to (-2, -1).
    let line = vectorcraft_geom::shapes::line(FRAME.to_doc(2.0, 1.0), FRAME.to_doc(-2.0, -1.0));
    let f = PathFunction::from_paths(&[&line], &FRAME).unwrap();
    assert_eq!(f.range(), (-2.0, 2.0));
    assert_near(f.at(0.5), 0.25, 1e-12, "line");
    // 1/x on [-2, 2], cut at ±10: two pieces with a gap round 0.
    let mut s = session();
    let id = graph(&mut s, "\\frac{1}{x}", [-2.0, 2.0], json!({"precision": 40}));
    assert!(matches!(node(&s, id).kind, NodeKind::Compound { .. }), "two subpaths");
    let f = function(&s, id);
    assert_near(f.at(-0.5), -2.0, 1e-6, "left branch");
    assert_near(f.at(1.0), 1.0, 1e-6, "right branch");
    assert!(f.at(0.0).is_nan() && f.at(0.05).is_nan(), "the gap");
}

#[test]
fn extrema_of_a_read_path() {
    let mut s = session();
    let id = graph(&mut s, "x^3-3x", [-2.5, 2.5], json!({}));
    let found = function(&s, id).extrema();
    assert_eq!(found.len(), 2, "{found:?}");
    let (max, min) = (found[0], found[1]);
    assert_eq!((max.1, min.1), (Extremum::Max, Extremum::Min));
    assert_near(max.0.x, -1.0, 1e-6, "max x");
    assert_near(max.0.y, 2.0, 1e-9, "max y");
    assert_near(min.0.x, 1.0, 1e-6, "min x");
    assert_near(min.0.y, -2.0, 1e-9, "min y");
    // A monotone curve, and a flat one, have none.
    let id = graph(&mut s, "x^3", [-1.0, 1.0], json!({}));
    assert!(function(&s, id).extrema().is_empty());
    let id = graph(&mut s, "1", [-1.0, 1.0], json!({}));
    assert!(function(&s, id).extrema().is_empty());
    // |x| has its minimum at the corner.
    let id = graph(&mut s, "\\abs{x}", [-1.0, 1.5], json!({"precision": 5}));
    let found = function(&s, id).extrema();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].1, Extremum::Min);
    assert_near(found[0].0.x, 0.0, 1e-6, "corner");
}

// ---------- axes ----------

#[test]
fn axes_have_two_lines_two_arrowheads_and_three_labels() {
    let mut s = session();
    let undo = s.doc().unwrap().history.undo.len();
    let v = run(&mut s, "mathdraw.axes", framed(json!({"xRange": [-1, 3], "yRange": [-2, 2]})));
    assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);
    assert_eq!(s.doc().unwrap().history.undo.last().unwrap().label, "Coordinate Axes");
    assert!(v.get("reference").is_none(), "a frame from parameters draws no reference");
    let g = node(&s, id_of(&v["id"]));
    assert_eq!(g.name.as_deref(), Some("Axes"));
    let [x, y] = ids_of(&v["axes"])[..] else { panic!("{v}") };
    let arrows = ids_of(&v["arrows"]);
    assert_eq!(arrows.len(), 2);
    // The x axis runs from u = −1 to under its arrowhead; the arrow's tip is at u = 3.
    let (bx, by) = (bounds(node(&s, x)), bounds(node(&s, y)));
    assert_near(bx.x0, 250.0, 1e-9, "x axis start");
    assert!(bx.x1 < 450.0 && bx.x1 > 420.0 && bx.y0 == 200.0, "{bx:?}");
    assert_near(by.y1, 300.0, 1e-9, "y axis start");
    assert!(by.y0 > 100.0 && by.x0 == 300.0, "{by:?}");
    let style = vectorcraft_doc::MathDrawStyle::default();
    assert_eq!(node(&s, x).appearance.stroke().unwrap().width, style.stroke(style.axis_line).0);
    let head = node(&s, arrows[0]);
    assert_near(bounds(head).x1, 450.0, 1e-9, "the arrow's tip");
    assert!(head.appearance.fill().is_some() && head.appearance.stroke().is_none(), "type 3 is filled");
    assert_eq!(path(head).subpaths[0].anchors.len(), 4, "swallowtail: tip, two barbs, the notch");
    if !math_font() {
        assert_eq!(v["labels"], json!("skipped (no math font)"));
        assert_eq!(children(g).len(), 4);
        return;
    }
    let labels = ids_of(&v["labels"]);
    assert_eq!(labels.len(), 3);
    assert_eq!(children(g).len(), 7);
    let names: Vec<_> = labels.iter().map(|l| node(&s, *l).name.clone().unwrap()).collect();
    assert_eq!(names, ["x", "y", "O"]);
    for l in &labels {
        let spec = node(&s, *l).extras().math.as_deref().cloned().unwrap();
        assert_eq!(spec.size, style.label_size, "set at the label size");
    }
    assert_eq!(node(&s, labels[2]).extras().math.as_deref().unwrap().source, "\\mathrm{O}");
    // x sits below the x axis's tip, y left of the y axis's tip, O below-left of the origin.
    let (lx, ly, lo) = (bounds(node(&s, labels[0])), bounds(node(&s, labels[1])), bounds(node(&s, labels[2])));
    assert!(lx.y0 > 200.0 && lx.x0 < 450.0 && lx.x1 > 420.0, "{lx:?}");
    assert!(ly.x1 < 300.0 && ly.y0 < 100.0 && ly.y1 > 100.0, "{ly:?}");
    assert!(lo.x1 < 300.0 && lo.y0 > 200.0, "{lo:?}");
}

#[test]
fn arrow_types_and_the_axis_line_follow_the_style() {
    let mut s = session();
    s.execute("mathdraw.setStyle", &json!({"arrowType": 1, "axisLine": "solid4"})).unwrap();
    let v = run(&mut s, "mathdraw.axes", framed(json!({})));
    let head = node(&s, ids_of(&v["arrows"])[0]);
    assert!(head.appearance.fill().is_none() && head.appearance.stroke().unwrap().width == 1.0, "type 1 is an open V of strokes");
    assert!(!path(head).subpaths[0].closed && path(head).subpaths[0].anchors.len() == 3);
    let x = node(&s, ids_of(&v["axes"])[0]);
    assert_near(bounds(x).x1, 400.0, 1e-9, "the line runs to the V's tip");
    s.execute("mathdraw.setStyle", &json!({"arrowType": 2, "arrowRatio": 2})).unwrap();
    let v = run(&mut s, "mathdraw.axes", framed(json!({"line": "solid1", "color": "#0000ff"})));
    let head = node(&s, ids_of(&v["arrows"])[1]);
    assert_eq!(path(head).subpaths[0].anchors.len(), 3, "type 2 is a triangle");
    assert_eq!(fill_rgb(head), [0.0, 0.0, 1.0]);
    // Ratio 2 on a 6 pt line: 10 × 6 × 2 long.
    assert_near(bounds(head).height(), 120.0, 1e-9, "arrow length");
}

/// With nothing selected the frame comes from the artboard and a reference segment is drawn and
/// selected, so the next command draws in the same frame.
#[test]
fn axes_in_the_default_frame_draw_and_select_a_reference() {
    let mut s = session();
    let v = run(&mut s, "mathdraw.axes", json!({}));
    let r = id_of(&v["reference"]);
    assert_eq!(bounds(node(&s, r)), Rect::new(300.0, 200.0, 450.0, 200.0));
    assert_eq!(s.doc().unwrap().selection.objects, vec![NodeId(r)]);
    let v2 = run(&mut s, "mathdraw.points", json!({"coords": [[1, 1]]}));
    assert_eq!(v2["reference"], v["reference"], "the same reference, no new one");
    let dot = node(&s, ids_of(&v2["dots"])[0]);
    assert_eq!(bounds(dot).center(), Point::new(450.0, 50.0));
}

#[test]
fn axes_fit_the_selection() {
    let mut s = session();
    let r = s.execute("shape.rectangle", &json!({"x": 350, "y": 100, "width": 100, "height": 50})).unwrap()["id"].as_u64().unwrap();
    s.execute("select.set", &json!({"ids": [r]})).unwrap();
    // The rectangle spans u 1…3, v 1…2; the axes reach from the origin to it.
    let v = run(&mut s, "mathdraw.axes", framed(json!({"fitSelection": true, "yRange": [-1, 2]})));
    let [x, y] = ids_of(&v["axes"])[..] else { panic!() };
    assert_near(bounds(node(&s, x)).x0, 300.0, 1e-9, "x from 0");
    let tip = bounds(node(&s, ids_of(&v["arrows"])[0])).x1;
    assert_near(tip, 450.0, 1e-9, "x to 3");
    assert_near(bounds(node(&s, y)).y1, 250.0, 1e-9, "the y range given wins");
}

// ---------- points and circles ----------

#[test]
fn points_have_dots_and_dashed_guides_to_both_axes() {
    let mut s = session();
    let v = run(&mut s, "mathdraw.points", framed(json!({"coords": [[1, 2], [0, -1]]})));
    let g = node(&s, id_of(&v["id"]));
    assert_eq!(g.name.as_deref(), Some("Points"));
    let dots = ids_of(&v["dots"]);
    let guides = ids_of(&v["guides"]);
    assert_eq!((dots.len(), guides.len()), (2, 3), "(0, −1) is on the y axis: one guide");
    assert_eq!(children(g).len(), 5);
    assert_eq!(children(g).last().unwrap().id, NodeId(dots[1]), "dots over the guides");
    assert_eq!(bounds(node(&s, dots[0])).center(), Point::new(350.0, 100.0));
    assert_eq!(bounds(node(&s, dots[1])).center(), Point::new(300.0, 250.0));
    let d = node(&s, dots[0]);
    assert_eq!((fill_rgb(d), stroke_rgb(d)), ([0.0; 3], [1.0; 3]), "black with a white border");
    assert_eq!(bounds(d).width(), 14.0);
    let style = vectorcraft_doc::MathDrawStyle::default();
    let guide = node(&s, guides[0]);
    let st = guide.appearance.stroke().unwrap();
    assert_eq!(
        (st.width, st.dash.as_ref().map(|d| d.pattern.clone())),
        (style.dashed[2].width, Some(vec![style.dashed[2].dash, style.dashed[2].gap]))
    );
    assert_eq!(bounds(guide), Rect::new(350.0, 100.0, 350.0, 200.0), "down to the x axis");
    assert_eq!(bounds(node(&s, guides[1])), Rect::new(300.0, 100.0, 350.0, 100.0), "across to the y axis");

    // No guides, a dot size: one point is one object.
    let v = run(&mut s, "mathdraw.points", framed(json!({"coords": [[1, 1]], "guides": false, "dot": 6})));
    assert_eq!(v["dots"], json!([id_of(&v["id"])]));
    assert_eq!(bounds(node(&s, id_of(&v["id"]))).width(), 6.0);
}

#[test]
fn circles_are_in_math_units() {
    let mut s = session();
    let v = run(&mut s, "mathdraw.circles", framed(json!({"centers": [[0, 0], [1, 1]], "radius": 0.5})));
    let c = ids_of(&v["circles"]);
    assert_eq!(bounds(node(&s, c[0])), Rect::new(275.0, 175.0, 325.0, 225.0));
    assert_eq!(bounds(node(&s, c[1])), Rect::new(325.0, 125.0, 375.0, 175.0));
    assert_eq!(node(&s, c[0]).appearance.stroke().unwrap().width, 2.0, "solid3");
    assert!(node(&s, c[0]).appearance.fill().is_none());
}

// ---------- graphs of expressions ----------

#[test]
fn the_graph_of_x_squared_goes_through_its_points() {
    let mut s = session();
    let undo = s.doc().unwrap().history.undo.len();
    let v = run(&mut s, "mathdraw.graph", framed(json!({"f": "x^2", "xRange": [-2, 2], "precision": 4})));
    assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);
    assert_eq!(v["id"], v["curve"], "only the curve: no group");
    let n = node(&s, id_of(&v["curve"]));
    let anchors: Vec<Point> = path(n).subpaths[0].anchors.iter().map(|a| a.p).collect();
    let want: Vec<Point> = [-2.0, -1.0, 0.0, 1.0, 2.0].iter().map(|&x: &f64| FRAME.to_doc(x, x * x)).collect();
    assert_eq!(anchors.len(), want.len());
    for (a, w) in anchors.iter().zip(&want) {
        assert!((*a - *w).hypot() < 1e-9, "{a:?} vs {w:?}");
    }
    // Between the anchors too: the cubics are exact for a parabola.
    let f = function(&s, id_of(&v["curve"]));
    for x in [-1.7, -0.4, 0.3, 1.5] {
        assert_near(f.at(x), x * x, 1e-9, "x^2");
    }
    let st = n.appearance.stroke().unwrap();
    assert_eq!((st.width, stroke_rgb(n)), (2.0, [0.0; 3]), "solid3, black");
}

#[test]
fn a_graph_is_cut_at_the_clip_band() {
    let mut s = session();
    let id = graph(&mut s, "x^2", [-4.0, 4.0], json!({}));
    let b = bounds(node(&s, id));
    assert_near(b.y0, FRAME.to_doc(0.0, 10.0).y, 1e-6, "cut at y = 10");
    let id = graph(&mut s, "x^2", [-4.0, 4.0], json!({"yClip": [-1, 4]}));
    assert_near(bounds(node(&s, id)).y0, FRAME.to_doc(0.0, 4.0).y, 1e-6, "cut at y = 4");
    assert_near(bounds(node(&s, id)).x0, FRAME.to_doc(-2.0, 0.0).x, 1e-6, "from x = −2");
}

#[test]
fn a_graph_with_axes_and_a_label_is_one_group() {
    let mut s = session();
    let v = run(&mut s, "mathdraw.graph", framed(json!({"f": "x^2", "xRange": [0, 2], "axes": true, "label": true})));
    let g = node(&s, id_of(&v["id"]));
    assert_eq!(g.name.as_deref(), Some("Graph"));
    let axes = node(&s, id_of(&v["axes"]["id"]));
    assert_eq!(axes.name.as_deref(), Some("Axes"));
    assert_eq!(children(g)[0].id, axes.id, "axes under the curve");
    // The axes cover the graph (x 0…2, y 0…4) with a margin.
    let tip = bounds(node(&s, ids_of(&v["axes"]["arrows"])[1])).y0;
    assert!(tip < FRAME.to_doc(0.0, 4.0).y, "the y axis reaches above the curve");
    if !math_font() {
        assert_eq!(v["label"], json!("skipped (no math font)"));
        return;
    }
    let label = node(&s, id_of(&v["label"]));
    assert_eq!(label.extras().math.as_deref().unwrap().source, "y=x^2");
    assert!(bounds(label).x0 > FRAME.to_doc(2.0, 0.0).x, "right of the curve's end");
    assert_eq!(children(g).len(), 3);
}

#[test]
fn a_parametric_circle() {
    let mut s = session();
    let v =
        run(&mut s, "mathdraw.parametric", framed(json!({"x": "2\\cos t", "y": "2\\sin t", "tRange": [0, std::f64::consts::TAU], "precision": 32})));
    let n = node(&s, id_of(&v["curve"]));
    let b = bounds(n);
    assert!((b.width() - 200.0).abs() < 0.5 && (b.height() - 200.0).abs() < 0.5, "{b:?}");
    for a in &path(n).subpaths[0].anchors {
        assert_near((a.p - FRAME.origin).hypot(), 100.0, 1e-9, "on the circle");
    }
    assert_eq!(s.doc().unwrap().history.undo.last().unwrap().label, "Parametric Curve");
}

// ---------- piecewise ----------

fn dot_kinds(s: &Session, v: &Value) -> (usize, usize) {
    let dots = ids_of(&v["dots"]);
    let hollow = dots.iter().filter(|d| fill_rgb(node(s, **d)) == [1.0; 3]).count();
    (dots.len() - hollow, hollow)
}

#[test]
fn piecewise_ends_are_open_or_closed_dots() {
    let mut s = session();
    let v = run(
        &mut s,
        "mathdraw.piecewise",
        framed(json!({"pieces": [
            {"f": "x", "range": [-1, 1], "rightEnd": "closed"},
            {"f": "2", "range": [1, 3], "leftEnd": "open", "precision": 2}
        ]})),
    );
    assert_eq!(ids_of(&v["curves"]).len(), 2);
    assert_eq!((v["closed"].clone(), v["open"].clone()), (json!([[1.0, 1.0]]), json!([[1.0, 2.0]])));
    assert_eq!(dot_kinds(&s, &v), (1, 1));
    let dots = ids_of(&v["dots"]);
    let (closed, open) = (node(&s, dots[0]), node(&s, dots[1]));
    assert_eq!(bounds(closed).center(), FRAME.to_doc(1.0, 1.0));
    assert_eq!(bounds(open).center(), FRAME.to_doc(1.0, 2.0));
    assert_eq!((fill_rgb(open), stroke_rgb(open)), ([1.0; 3], [0.0; 3]), "hollow: white with a black border");
    assert_eq!((fill_rgb(closed), stroke_rgb(closed)), ([0.0; 3], [1.0; 3]));
    let g = node(&s, id_of(&v["id"]));
    assert_eq!(g.name.as_deref(), Some("Piecewise Graph"));
    assert_eq!(children(g).len(), 4, "two curves, then two dots over them");
}

/// A value at a boundary: a filled dot there, hollow ones where the pieces meeting there end.
#[test]
fn a_piecewise_value_end() {
    let mut s = session();
    let v = run(
        &mut s,
        "mathdraw.piecewise",
        framed(json!({"pieces": [
            {"f": "x", "range": [0, 1], "rightEnd": {"value": 3}},
            {"f": "x+1", "range": [1, 2]},
            {"f": "x^2", "range": [2, 3], "leftEnd": {"value": 4}}
        ], "axes": true})),
    );
    assert_eq!(v["closed"], json!([[1.0, 3.0], [2.0, 4.0]]));
    assert_eq!(v["open"], json!([[1.0, 1.0], [1.0, 2.0], [2.0, 3.0]]), "x+1 ends at 3 by x = 2, where x^2 is 4 already");
    assert_eq!(dot_kinds(&s, &v), (2, 3));
    assert!(v["axes"]["id"].is_u64());
}

// ---------- graphs read from curves ----------

#[test]
fn extrema_get_dots_and_guides() {
    let mut s = session();
    let curve = graph(&mut s, "x^3-3x", [-2.5, 2.5], json!({}));
    let v = run(&mut s, "mathdraw.extrema", framed(json!({"id": curve, "guides": true})));
    let (maxima, minima) = (v["maxima"].as_array().unwrap(), v["minima"].as_array().unwrap());
    assert_eq!((maxima.len(), minima.len()), (1, 1));
    assert_near(maxima[0][0].as_f64().unwrap(), -1.0, 1e-6, "max");
    assert_near(minima[0][1].as_f64().unwrap(), -2.0, 1e-9, "min");
    let dots = ids_of(&v["dots"]);
    assert!((bounds(node(&s, dots[0])).center() - FRAME.to_doc(-1.0, 2.0)).hypot() < 1e-4);
    assert_eq!(ids_of(&v["guides"]).len(), 4);
    assert_eq!(s.doc().unwrap().history.undo.last().unwrap().label, "Mark Maxima and Minima");
    // The selected curve, without guides.
    s.execute("select.set", &json!({"ids": [curve]})).unwrap();
    let v = run(&mut s, "mathdraw.extrema", framed(json!({})));
    assert_eq!((ids_of(&v["dots"]).len(), ids_of(&v["guides"]).len()), (2, 0));
}

/// The curve picked from a selected graph group is its curve, not its axes.
#[test]
fn the_curve_is_found_in_a_selected_graph_group() {
    let mut s = session();
    s.execute("mathdraw.setStyle", &json!({"arrowType": 1})).unwrap();
    let v = run(&mut s, "mathdraw.graph", framed(json!({"f": "-x^2+1", "xRange": [-1, 1], "axes": true})));
    s.execute("select.set", &json!({"ids": [id_of(&v["id"])]})).unwrap();
    let e = run(&mut s, "mathdraw.extrema", framed(json!({})));
    let max = e["maxima"].as_array().unwrap();
    assert_eq!((max.len(), e["minima"].as_array().unwrap().len()), (1, 0), "{e}");
    assert_near(max[0][0].as_f64().unwrap(), 0.0, 1e-6, "x");
    assert_near(max[0][1].as_f64().unwrap(), 1.0, 1e-9, "y");
}

#[test]
fn the_derivative_of_x_squared_is_2x() {
    let mut s = session();
    let curve = graph(&mut s, "x^2", [-2.0, 2.0], json!({}));
    let v = run(&mut s, "mathdraw.derivative", framed(json!({"id": curve, "precision": 41})));
    let f = function(&s, id_of(&v["id"]));
    assert_eq!(f.range(), (-2.0, 2.0));
    for x in [-2.0, -1.5, -0.3, 0.0, 0.8, 1.9, 2.0] {
        assert_near(f.at(x), 2.0 * x, 1e-3, &format!("f'({x})"));
    }
    assert_eq!(s.doc().unwrap().history.undo.last().unwrap().label, "Derivative Graph");
}

#[test]
fn the_integral_of_2x_is_x_squared_from_the_left() {
    let mut s = session();
    let curve = graph(&mut s, "2x", [0.0, 2.0], json!({}));
    s.execute("select.set", &json!({"ids": [curve]})).unwrap();
    let v = run(&mut s, "mathdraw.integral", framed(json!({})));
    let f = function(&s, id_of(&v["id"]));
    for x in [0.0, 0.5, 1.0, 1.7, 2.0] {
        assert_near(f.at(x), x * x, 1e-3, &format!("F({x})"));
    }
    let v = run(&mut s, "mathdraw.integral", framed(json!({"id": curve, "c0At": "right"})));
    let f = function(&s, id_of(&v["id"]));
    assert_near(f.at(2.0), 0.0, 1e-9, "0 at the right");
    assert_near(f.at(0.0), -4.0, 1e-3, "F(0) − F(2)");
}

#[test]
fn graph_arithmetic_by_colour_and_by_id() {
    let mut s = session();
    let f = graph(&mut s, "x", [-1.0, 2.0], json!({}));
    let g = graph(&mut s, "x^2", [0.0, 3.0], json!({"color": "#0000ff"}));
    let h = graph(&mut s, "1", [-3.0, 3.0], json!({"color": "#ffff00"}));
    s.execute("select.set", &json!({"ids": [h, g, f]})).unwrap();
    let v = run(&mut s, "mathdraw.combine", framed(json!({"expr": "f+g*h", "precision": 31})));
    assert_eq!(v["range"], json!([0.0, 2.0]), "where all three are");
    let r = function(&s, id_of(&v["id"]));
    for x in [0.0, 0.7, 1.0, 2.0] {
        assert_near(r.at(x), x + x * x, 1e-6, &format!("({x})"));
    }
    // Given by id: g may be any colour; x counts too.
    s.execute("select.none", &json!({})).unwrap();
    let v = run(&mut s, "mathdraw.combine", framed(json!({"expr": "g-x", "ids": {"g": f}})));
    let r = function(&s, id_of(&v["id"]));
    assert_near(r.at(1.5), 0.0, 1e-9, "x − x");
    assert_eq!(s.doc().unwrap().history.undo.last().unwrap().label, "Graph Arithmetic");
}

/// A reference segment among the selected curves is the frame, not a curve.
#[test]
fn the_reference_segment_is_not_a_curve() {
    let mut s = session();
    let r = s.execute("shape.line", &json!({"x1": 300, "y1": 200, "x2": 350, "y2": 200})).unwrap()["id"].as_u64().unwrap();
    s.execute("paint.setStroke", &json!({"color": "#ff00ff", "ids": [r]})).unwrap();
    s.execute("select.set", &json!({"ids": [r]})).unwrap();
    assert!(matches!(s.execute("mathdraw.derivative", &json!({})), Err(EngineError::BadParams { .. })));
    let curve = graph(&mut s, "x^2", [-1.0, 1.0], json!({}));
    s.execute("select.set", &json!({"ids": [r, curve]})).unwrap();
    let v = run(&mut s, "mathdraw.derivative", json!({}));
    assert_eq!(v["reference"], json!(r), "drawn in the reference's frame");
    let f = function(&s, id_of(&v["id"]));
    assert_near(f.at(0.5), 1.0, 1e-3, "2x");
}

#[test]
fn colours_and_lines_are_taken() {
    let mut s = session();
    let id = graph(&mut s, "x", [0.0, 1.0], json!({"color": "#ff0000", "line": "dashed1"}));
    let st = node(&s, id).appearance.stroke().unwrap().clone();
    assert_eq!((st.width, st.dash.map(|d| d.pattern)), (2.0, Some(vec![12.0, 6.0])));
    assert_eq!(stroke_rgb(node(&s, id)), [1.0, 0.0, 0.0]);
    let _ = Color::BLACK;
}
