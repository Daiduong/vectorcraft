//! Math drawing: the style commands, the coordinate frame (parameters, reference segment,
//! artboard) and the drawing helpers.

use super::*;
use crate::cmd::mathdraw::{Frame, FrameSource, color_param, curve_path, dot, line_look, line_param, resolve_frame, stroked};
use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{AppearanceItem, LineRef, MathDrawStyle, Node, NodeKind};
use vectorcraft_geom::{PathData, Point, Rect, shapes};

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 600, "height": 400})).unwrap();
    s
}

fn node(s: &Session, id: NodeId) -> &Node {
    s.doc().unwrap().doc.node(id).unwrap()
}

/// A line segment, selected.
fn segment(s: &mut Session, a: (f64, f64), b: (f64, f64), color: &str) -> NodeId {
    let id = s.execute("shape.line", &json!({"x1": a.0, "y1": a.1, "x2": b.0, "y2": b.1})).unwrap()["id"].as_u64().unwrap();
    s.execute("paint.setStroke", &json!({"color": color, "ids": [id]})).unwrap();
    s.execute("select.set", &json!({"ids": [id]})).unwrap();
    NodeId(id)
}

fn stroke_rgb(n: &Node) -> [f32; 3] {
    match &n.appearance.stroke().unwrap().paint {
        Paint::Solid { color, .. } => color.to_rgb_uncalibrated(),
        p => panic!("{p:?}"),
    }
}

fn count(s: &Session) -> usize {
    let mut n = 0;
    for l in &s.doc().unwrap().doc.layers {
        l.walk(&mut |_| n += 1);
    }
    n
}

fn close(a: Point, b: Point) -> bool {
    (a - b).hypot() < 1e-9
}

// ---------- style ----------

#[test]
fn the_style_commands_are_registered_and_need_a_document() {
    for (id, journal) in [("mathdraw.style", false), ("mathdraw.setStyle", true)] {
        let c = find_command(id).unwrap();
        assert!(!c.label.is_empty() && c.journal == journal, "{id}");
        assert!((c.enabled)(&Session::new()).is_err(), "{id} without a document");
        assert!((c.enabled)(&session()).is_ok(), "{id} with a document");
    }
}

#[test]
fn the_style_reads_as_the_shipped_defaults() {
    let mut s = session();
    let v = s.execute("mathdraw.style", &json!({})).unwrap();
    assert_eq!(v, serde_json::to_value(MathDrawStyle::default()).unwrap());
    assert_eq!(v["solid"], json!([6.0, 2.5, 2.0, 1.0, 0.5]));
    assert_eq!(v["dashed"][0], json!({"width": 2.0, "dash": 12.0, "gap": 6.0}));
    assert_eq!((v["dotSize"].as_f64(), v["dotBorder"].as_str(), v["axisLine"].as_str()), (Some(14.0), Some("solid3"), Some("solid2")));
    assert_eq!((v["stdLength"].as_f64(), v["arrowType"].as_u64(), v["labelSize"].as_f64()), (Some(150.0), Some(3), Some(73.33)));
}

#[test]
fn set_style_merges_and_undoes_in_one_step() {
    let mut s = session();
    let undo = s.doc().unwrap().history.undo.len();
    let v = s
        .execute("mathdraw.setStyle", &json!({"dotSize": 10, "labelY": {"italic": false}, "dashed": [null, null, {"dash": 5}, null, null]}))
        .unwrap();
    assert_eq!(v, s.execute("mathdraw.style", &json!({})).unwrap(), "returns the whole style");
    let st = &s.doc().unwrap().doc.math_draw;
    assert_eq!((st.dot_size, st.label_y.italic, st.label_y.offset), (10.0, false, MathDrawStyle::default().label_y.offset));
    assert_eq!((st.dashed[2].width, st.dashed[2].dash, st.dashed[2].gap), (1.0, 5.0, 3.0));
    let h = &s.doc().unwrap().history;
    assert_eq!(h.undo.len(), undo + 1);
    assert_eq!(h.undo.last().unwrap().label, "Math Drawing Style");
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(s.doc().unwrap().doc.math_draw.is_default());
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().doc.math_draw.dot_size, 10.0);
    // Setting what is already set is no step.
    let undo = s.doc().unwrap().history.undo.len();
    s.execute("mathdraw.setStyle", &json!({"dotSize": 10})).unwrap();
    s.execute("mathdraw.setStyle", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().history.undo.len(), undo);
}

#[test]
fn set_style_refuses_bad_values_and_changes_nothing() {
    let mut s = session();
    s.execute("mathdraw.setStyle", &json!({"stdLength": 120})).unwrap();
    let before = s.doc().unwrap().doc.clone();
    let undo = s.doc().unwrap().history.undo.len();
    for p in [
        json!({"dotsize": 3}),
        json!({"dotSize": "big"}),
        json!({"dotSize": -1}),
        json!({"stdLength": 120, "axisLine": "solid6"}),
        json!({"solid": [1, 2]}),
        json!({"labelX": {"offset": "here"}}),
        json!({"arrowType": 9}),
        json!([1, 2]),
        json!("solid1"),
    ] {
        let e = s.execute("mathdraw.setStyle", &p).unwrap_err();
        assert!(matches!(e, EngineError::BadParams { .. }), "{p}: {e}");
    }
    assert_eq!(s.doc().unwrap().doc, before);
    assert_eq!(s.doc().unwrap().history.undo.len(), undo);
}

#[test]
fn set_style_params_are_checked_by_key() {
    check_params("mathdraw.setStyle", &json!({"solid": [1, 1, 1, 1, 1], "labelO": {"italic": true}, "arrowRatio": 2, "hitTolerance": 4})).unwrap();
    assert!(check_params("mathdraw.setStyle", &json!({"colour": "red"})).is_err());
    check_params("mathdraw.style", &json!({})).unwrap();
}

/// The style is the document's: a saved file keeps it, a new document has the defaults.
#[test]
fn the_style_is_saved_with_the_document() {
    let mut s = session();
    s.execute("mathdraw.setStyle", &json!({"axisLine": "solid1", "labelSize": 20})).unwrap();
    let back = vectorcraft_format::load(&vectorcraft_format::save(&s.doc().unwrap().doc, false)).unwrap();
    assert_eq!(back.math_draw, s.doc().unwrap().doc.math_draw);
    assert!(session().doc().unwrap().doc.math_draw.is_default());
}

// ---------- frame ----------

#[test]
fn a_frame_maps_math_coordinates_y_up() {
    let f = Frame { origin: Point::new(100.0, 200.0), unit: 50.0 };
    assert_eq!(f.to_doc(0.0, 0.0), Point::new(100.0, 200.0));
    assert_eq!(f.to_doc(2.0, 1.0), Point::new(200.0, 150.0));
    assert_eq!(f.to_math(Point::new(50.0, 300.0)), (-1.0, -2.0));
    assert_eq!(f.affine() * Point::new(2.0, 1.0), f.to_doc(2.0, 1.0));
}

/// With nothing selected, the frame is the artboard's centre and the standard length, and a
/// magenta reference segment is drawn; selecting it gives the same frame back.
#[test]
fn the_default_frame_is_the_artboard_centre_with_a_reference_segment() {
    let mut s = session();
    let plan = resolve_frame(&s, &json!({}), "t").unwrap();
    assert_eq!((plan.source, plan.frame), (FrameSource::Artboard, Frame { origin: Point::new(300.0, 200.0), unit: 150.0 }));
    let before = count(&s);
    let parent = s.doc().unwrap().target_parent().unwrap();
    let id = s.edit("t", |d, _| plan.apply(d, parent)).unwrap().unwrap();
    assert_eq!(count(&s), before + 1);
    let n = node(&s, id);
    assert_eq!(n.geometric_bounds().unwrap(), Rect::new(300.0, 200.0, 450.0, 200.0));
    assert_eq!(stroke_rgb(n), [1.0, 0.0, 1.0]);
    assert_eq!(n.appearance.stroke().unwrap().width, 1.0);
    assert!(n.appearance.fill().is_none());
    s.execute("select.set", &json!({"ids": [id.0]})).unwrap();
    let again = resolve_frame(&s, &json!({}), "t").unwrap();
    assert_eq!((again.source, again.frame), (FrameSource::Reference(id), plan.frame));
    assert_eq!(s.edit("t", |d, _| again.apply(d, parent)).unwrap(), Some(id), "nothing more is drawn");
    assert_eq!(count(&s), before + 1);
}

#[test]
fn make_reference_false_draws_none_and_the_artboard_is_chosen() {
    let mut s = session();
    s.execute("artboard.new", &json!({"x": 1000, "y": 0, "width": 200, "height": 100})).unwrap();
    let plan = resolve_frame(&s, &json!({"makeReference": false, "artboard": 1}), "t").unwrap();
    assert_eq!(plan.frame.origin, Point::new(1100.0, 50.0));
    let before = count(&s);
    assert_eq!(s.edit("t", |d, _| plan.apply(d, None)).unwrap(), None);
    assert_eq!(count(&s), before);
    assert!(resolve_frame(&s, &json!({"artboard": 7}), "t").is_err());
}

#[test]
fn origin_and_unit_parameters_win() {
    let mut s = session();
    segment(&mut s, (0.0, 0.0), (40.0, 0.0), "#ff00ff");
    let plan = resolve_frame(&s, &json!({"origin": [10, 20], "unit": 30}), "t").unwrap();
    assert_eq!((plan.source, plan.frame), (FrameSource::Params, Frame { origin: Point::new(10.0, 20.0), unit: 30.0 }));
    // Either one alone overrides that part of the frame found otherwise.
    let plan = resolve_frame(&s, &json!({"unit": 30}), "t").unwrap();
    assert_eq!(plan.frame, Frame { origin: Point::new(0.0, 0.0), unit: 30.0 });
    s.execute("select.none", &json!({})).unwrap();
    let plan = resolve_frame(&s, &json!({"origin": [5, 6]}), "t").unwrap();
    assert_eq!((plan.source, plan.frame), (FrameSource::Artboard, Frame { origin: Point::new(5.0, 6.0), unit: 150.0 }));
    s.execute("mathdraw.setStyle", &json!({"stdLength": 80})).unwrap();
    assert_eq!(resolve_frame(&s, &json!({}), "t").unwrap().frame.unit, 80.0);
}

#[test]
fn bad_frame_parameters_are_refused() {
    let s = session();
    for p in [
        json!({"origin": [1]}),
        json!({"origin": [1, "2"]}),
        json!({"origin": [1e300, 0]}),
        json!({"origin": 5}),
        json!({"unit": 0}),
        json!({"unit": -2}),
        json!({"unit": 1e9}),
        json!({"unit": "1"}),
        json!({"makeReference": "yes"}),
        json!({"artboard": -1}),
    ] {
        let e = resolve_frame(&s, &p, "t").unwrap_err();
        assert!(matches!(e, EngineError::BadParams { .. }), "{p}: {e}");
    }
}

/// The reference segment's bounding box gives the frame: bottom-left (y down) is the origin, the
/// width the unit. A magenta one wins among others; a lone segment of another colour becomes the
/// reference and turns magenta.
#[test]
fn the_reference_segment_in_the_selection() {
    let mut s = session();
    let diag = segment(&mut s, (10.0, 100.0), (70.0, 40.0), "#ff00ff");
    let other = segment(&mut s, (0.0, 0.0), (5.0, 5.0), "#000000");
    let rect = s.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap()["id"].as_u64().unwrap();
    s.execute("select.set", &json!({"ids": [rect, other.0, diag.0]})).unwrap();
    let plan = resolve_frame(&s, &json!({}), "t").unwrap();
    assert_eq!((plan.source, plan.frame), (FrameSource::Reference(diag), Frame { origin: Point::new(10.0, 100.0), unit: 60.0 }));

    // A lone black segment (beside a rectangle) is recoloured when used.
    s.execute("select.set", &json!({"ids": [rect, other.0]})).unwrap();
    let plan = resolve_frame(&s, &json!({}), "t").unwrap();
    assert_eq!((plan.source, plan.frame), (FrameSource::Reference(other), Frame { origin: Point::new(0.0, 5.0), unit: 5.0 }));
    assert_eq!(stroke_rgb(node(&s, other)), [0.0, 0.0, 0.0], "resolving changes nothing");
    let undo = s.doc().unwrap().history.undo.len();
    assert_eq!(s.edit("t", |d, _| plan.apply(d, None)).unwrap(), Some(other));
    assert_eq!(stroke_rgb(node(&s, other)), [1.0, 0.0, 1.0]);
    assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);

    // Two segments, neither magenta: no reference, the artboard.
    let a = segment(&mut s, (0.0, 0.0), (5.0, 0.0), "#000000");
    let b = segment(&mut s, (0.0, 9.0), (5.0, 9.0), "#0000ff");
    s.execute("select.set", &json!({"ids": [a.0, b.0]})).unwrap();
    assert_eq!(resolve_frame(&s, &json!({}), "t").unwrap().source, FrameSource::Artboard);

    // A segment inside a selected group is found.
    s.execute("select.set", &json!({"ids": [diag.0, rect]})).unwrap();
    let g = s.execute("object.group", &json!({})).unwrap()["id"].as_u64().unwrap();
    s.execute("select.set", &json!({"ids": [g]})).unwrap();
    assert_eq!(resolve_frame(&s, &json!({}), "t").unwrap().source, FrameSource::Reference(diag));

    // A vertical magenta segment has no width to be a unit.
    let v = segment(&mut s, (3.0, 0.0), (3.0, 50.0), "#ff00ff");
    s.execute("select.set", &json!({"ids": [v.0]})).unwrap();
    assert!(matches!(resolve_frame(&s, &json!({}), "t"), Err(EngineError::BadParams { .. })));
}

/// A curved two-anchor path, a closed path or a three-anchor one is not a segment.
#[test]
fn only_straight_segments_are_references() {
    let mut s = session();
    let curve = s.execute("shape.arc", &json!({"x1": 0, "y1": 0, "x2": 50, "y2": 50})).unwrap()["id"].as_u64().unwrap();
    s.execute("paint.setStroke", &json!({"color": "#ff00ff", "ids": [curve]})).unwrap();
    let poly = s.execute("shape.polygon", &json!({"cx": 0, "cy": 0, "radius": 20, "sides": 3})).unwrap()["id"].as_u64().unwrap();
    s.execute("paint.setStroke", &json!({"color": "#ff00ff", "ids": [poly]})).unwrap();
    s.execute("select.set", &json!({"ids": [curve, poly]})).unwrap();
    assert_eq!(resolve_frame(&s, &json!({}), "t").unwrap().source, FrameSource::Artboard);
}

#[test]
fn cmyk_documents_use_cmyk_magenta() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 600, "height": 400, "colorMode": "cmyk"})).unwrap();
    let plan = resolve_frame(&s, &json!({}), "t").unwrap();
    let id = s.edit("t", |d, _| plan.apply(d, None)).unwrap().unwrap();
    let Paint::Solid { color, .. } = &node(&s, id).appearance.stroke().unwrap().paint else { panic!() };
    assert_eq!(*color, Color::cmyk(0.0, 1.0, 0.0, 0.0));
    s.execute("select.set", &json!({"ids": [id.0]})).unwrap();
    assert_eq!(resolve_frame(&s, &json!({}), "t").unwrap().source, FrameSource::Reference(id), "CMYK magenta is magenta");
}

// ---------- drawing helpers ----------

#[test]
fn lines_are_stroked_from_the_style() {
    let style = MathDrawStyle::default();
    let look = line_look(&style, LineRef::Dashed(2), Color::BLACK);
    let [AppearanceItem::Stroke(st)] = look.items.as_slice() else { panic!("{look:?}") };
    assert_eq!((st.width, st.dash.as_ref().map(|d| d.pattern.clone())), (4.8, Some(vec![18.0, 9.0])));
    let solid = line_look(&style, LineRef::Solid(1), Color::BLACK);
    assert_eq!((solid.stroke().unwrap().width, solid.stroke().unwrap().dash.is_none()), (6.0, true));
    let mut d = vectorcraft_doc::Document::new(100.0, 100.0);
    let n = stroked(&mut d, shapes::line(Point::ZERO, Point::new(10.0, 0.0)), &style, LineRef::Solid(3), Color::rgb(0.0, 0.0, 1.0));
    assert!(matches!(n.kind, NodeKind::Path { .. }));
    assert_eq!((n.appearance.stroke().unwrap().width, n.appearance.fill().is_none()), (2.0, true));
}

#[test]
fn a_dot_is_black_with_a_white_border() {
    let mut style = MathDrawStyle::default();
    let mut d = vectorcraft_doc::Document::new(100.0, 100.0);
    let n = dot(&mut d, Point::new(50.0, 40.0), &style, Color::BLACK, Color::WHITE);
    assert_eq!(n.geometric_bounds().unwrap(), Rect::new(43.0, 33.0, 57.0, 47.0));
    assert_eq!(n.appearance.fill().unwrap().paint, Paint::solid(Color::BLACK));
    let border = n.appearance.stroke().unwrap();
    assert_eq!((border.paint.clone(), border.width), (Paint::solid(Color::WHITE), 2.0));
    assert!(matches!(n.appearance.items.as_slice(), [AppearanceItem::Fill(_), AppearanceItem::Stroke(_)]), "the border over the fill");
    style.patch(&json!({"dotSize": 8, "dotBorder": "solid5"})).unwrap();
    let n = dot(&mut d, Point::ZERO, &style, Color::BLACK, Color::WHITE);
    assert_eq!((n.geometric_bounds().unwrap().width(), n.appearance.stroke().unwrap().width), (8.0, 0.5));
}

#[test]
fn fitted_curves_go_onto_the_page_through_the_frame() {
    let f = vectorcraft_math::parse_expr("x^2", &["x"]).unwrap();
    let runs = vectorcraft_math::fit_function(&f, "x", (0.0, 1.0), 4, Default::default());
    let frame = Frame { origin: Point::new(100.0, 200.0), unit: 50.0 };
    let path: PathData = curve_path(&runs, &frame);
    assert_eq!(path.subpaths.len(), 1);
    let sp = &path.subpaths[0];
    assert_eq!(sp.anchors.len(), 5);
    assert!(close(sp.anchors[0].p, Point::new(100.0, 200.0)) && close(sp.anchors[4].p, Point::new(150.0, 150.0)));
    // x = 0.5 → y = 0.25 → 12.5 pt above the origin.
    assert!(close(sp.anchors[2].p, Point::new(125.0, 187.5)));
    let split =
        vectorcraft_math::fit_function(&vectorcraft_math::parse_expr(r"\frac{1}{x}", &["x"]).unwrap(), "x", (-1.0, 1.0), 8, Default::default());
    assert_eq!(curve_path(&split, &frame).subpaths.len(), 2);
}

#[test]
fn colour_and_line_parameters() {
    let p = json!({"color": "#0000ff", "line": "dashed3", "bad": 5, "worse": "solid0"});
    assert_eq!(color_param(&p, "color", "t").unwrap(), Some(Color::rgb(0.0, 0.0, 1.0)));
    assert_eq!(color_param(&p, "missing", "t").unwrap(), None);
    assert!(color_param(&p, "bad", "t").is_err());
    assert_eq!(line_param(&p, "line", "t").unwrap(), Some(LineRef::Dashed(3)));
    assert_eq!(line_param(&json!({"line": null}), "line", "t").unwrap(), None);
    assert!(line_param(&p, "bad", "t").is_err() && line_param(&p, "worse", "t").is_err());
    let _: Value = p;
}
