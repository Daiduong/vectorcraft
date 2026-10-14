//! Equations: `math.create` and `math.edit`. Typesetting needs a font with a MATH table, which the
//! repository doesn't ship (fonts live in craft-fonts): the tests that set a formula run when the
//! font database has one (a system font, or the file `VECTORCRAFT_MATH_TEST_FONT` names, added to
//! it) and skip otherwise; the parameter checks and the no-font error need none.

use super::*;
use serde_json::json;
use vectorcraft_doc::{MathSpec, Node};
use vectorcraft_geom::Rect;
use vectorcraft_text::FontDb;

/// Whether the global font database has a math font, after adding the one the environment names.
pub(crate) fn math_font() -> bool {
    static ADD: std::sync::Once = std::sync::Once::new();
    ADD.call_once(|| {
        if let Some(bytes) = std::env::var_os("VECTORCRAFT_MATH_TEST_FONT").and_then(|p| std::fs::read(p).ok()) {
            FontDb::global().add_font(bytes);
        }
    });
    vectorcraft_math::find_math_face(FontDb::global()).is_some()
}

macro_rules! font_or_skip {
    () => {
        if !math_font() {
            eprintln!("skipped: no font with a MATH table (set VECTORCRAFT_MATH_TEST_FONT)");
            return;
        }
    };
}

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 600, "height": 400})).unwrap();
    s
}

fn node(s: &Session, id: NodeId) -> &Node {
    s.doc().unwrap().doc.node(id).unwrap()
}

fn spec(s: &Session, id: NodeId) -> MathSpec {
    node(s, id).extras().math.as_deref().cloned().unwrap()
}

fn bounds(s: &Session, id: NodeId) -> Rect {
    node(s, id).geometric_bounds().unwrap()
}

fn create(s: &mut Session, params: serde_json::Value) -> NodeId {
    NodeId(s.execute("math.create", &params).unwrap()["id"].as_u64().unwrap())
}

fn near(a: Rect, b: Rect) -> bool {
    (a.x0 - b.x0).abs() + (a.y0 - b.y0).abs() + (a.x1 - b.x1).abs() + (a.y1 - b.y1).abs() < 1e-6
}

#[test]
fn the_commands_are_registered_and_need_a_document() {
    for id in ["math.create", "math.edit"] {
        let c = find_command(id).unwrap();
        assert!(!c.label.is_empty() && c.journal, "{id}");
        assert!((c.enabled)(&Session::new()).is_err(), "{id} without a document");
        assert!((c.enabled)(&session()).is_ok(), "{id} with a document");
    }
}

/// Wrong or missing parameters are refused before anything changes, font or no font.
#[test]
fn bad_parameters_are_refused_and_change_nothing() {
    let mut s = session();
    let before = s.doc().unwrap().doc.clone();
    let undo = s.doc().unwrap().history.undo.len();
    for p in [
        json!({}),
        json!({"source": 5}),
        json!({"source": ["x"]}),
        json!({"source": "x", "size": 0}),
        json!({"source": "x", "size": -3}),
        json!({"source": "x", "size": 1e308}),
        json!({"source": "x", "size": "big"}),
        json!({"source": "x", "display": "yes"}),
        json!({"source": "x", "x": 1e300, "y": 0}),
        json!({"source": "x", "x": "left"}),
        json!({"source": "x", "artboard": -1}),
        json!({"source": "x", "artboard": "first"}),
    ] {
        let r = s.execute("math.create", &p);
        assert!(matches!(r, Err(EngineError::BadParams { .. })), "{p}: {r:?}");
    }
    assert!(s.doc().unwrap().doc == before);
    assert_eq!(s.doc().unwrap().history.undo.len(), undo);
    // Nothing to edit.
    assert!(matches!(s.execute("math.edit", &json!({"source": "x"})), Err(EngineError::BadParams { .. })));
    assert!(matches!(s.execute("math.edit", &json!({"id": "a", "source": "x"})), Err(EngineError::BadParams { .. })));
    assert!(matches!(s.execute("math.edit", &json!({"id": 9999, "source": "x"})), Err(EngineError::BadParams { .. })));
}

/// With no math font, the command says so and leaves the document alone.
#[test]
fn without_a_math_font_create_reports_it() {
    if math_font() {
        eprintln!("skipped: this machine has a math font");
        return;
    }
    let mut s = session();
    let before = s.doc().unwrap().doc.clone();
    let err = s.execute("math.create", &json!({"source": "x^2"})).unwrap_err();
    assert!(err.to_string().contains("No math font available"), "{err}");
    assert!(s.doc().unwrap().doc == before);
    assert!(s.doc().unwrap().history.undo.is_empty());
}

#[test]
fn create_makes_a_selected_group_of_outlines_centred_where_asked_and_undoes_in_one_step() {
    font_or_skip!();
    let mut s = session();
    let undo = s.doc().unwrap().history.undo.len();
    let r = s.execute("math.create", &json!({"source": r"y=\sqrt{x}+\dfrac{1}{\sin x}", "x": 300, "y": 200})).unwrap();
    let id = NodeId(r["id"].as_u64().unwrap());
    let n = node(&s, id);
    assert!(matches!(n.kind, NodeKind::Group { .. }));
    assert_eq!(n.name.as_deref(), Some("Equation"));
    let kids = n.children().unwrap();
    assert!(kids.len() > 8, "{} shapes", kids.len());
    // Plain outlines with a solid black fill, nothing on the page but the group.
    assert!(kids.iter().all(|c| matches!(c.kind, NodeKind::Path { .. } | NodeKind::Compound { .. })));
    assert!(kids.iter().all(|c| c.appearance.stroke_paint().is_none() && !c.appearance.fill_paint().is_none()));
    // Centred on (300, 200); the spec holds the settings and where the art stands.
    let b = bounds(&s, id);
    assert!((b.center().x - 300.0).abs() < 1e-6 && (b.center().y - 200.0).abs() < 1e-6, "{b:?}");
    assert!(b.width() > 40.0 && b.height() > 20.0, "{b:?}");
    let sp = spec(&s, id);
    assert_eq!((sp.source.as_str(), sp.size, sp.display), (r"y=\sqrt{x}+\dfrac{1}{\sin x}", 24.0, true));
    assert!(near(sp.bounds, b));
    assert_eq!(r["bounds"], json!([b.x0, b.y0, b.x1, b.y1]));
    assert_eq!(s.doc().unwrap().selection.objects, vec![id]);
    // One undo step.
    assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);
    assert_eq!(s.doc().unwrap().history.undo.last().unwrap().label, "Insert Equation");
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(s.doc().unwrap().doc.node(id).is_none());
}

#[test]
fn create_defaults_to_the_artboard_centre_and_takes_size_and_display() {
    font_or_skip!();
    let mut s = session();
    let id = create(&mut s, json!({"source": r"\frac{a}{b}"}));
    let c = bounds(&s, id).center();
    assert!((c.x - 300.0).abs() < 1e-6 && (c.y - 200.0).abs() < 1e-6, "{c:?}");
    let big = create(&mut s, json!({"source": r"\sum_{i=1}^{n} i", "size": 48, "display": false, "x": 100}));
    let sp = spec(&s, big);
    assert_eq!((sp.size, sp.display), (48.0, false));
    let c = bounds(&s, big).center();
    assert!((c.x - 100.0).abs() < 1e-6 && (c.y - 200.0).abs() < 1e-6, "x alone moves only x: {c:?}");
    // A bigger base size sets a bigger formula.
    let small = create(&mut s, json!({"source": r"\sum_{i=1}^{n} i", "size": 12, "display": false}));
    assert!(bounds(&s, big).width() > 3.0 * bounds(&s, small).width());
}

#[test]
fn a_formula_that_cannot_be_set_is_an_error_and_changes_nothing() {
    font_or_skip!();
    let mut s = session();
    let before = s.doc().unwrap().doc.clone();
    for src in [r"\nosuchcommand", r"\frac{1}", "", "   ", "x^"] {
        let r = s.execute("math.create", &json!({"source": src}));
        assert!(r.is_err(), "{src:?}: {r:?}");
        assert!(s.doc().unwrap().doc == before, "{src:?}");
    }
    assert!(s.doc().unwrap().history.undo.is_empty());
}

#[test]
fn edit_retypesets_in_place_keeping_the_users_move_and_scale() {
    font_or_skip!();
    let mut s = session();
    let id = create(&mut s, json!({"source": "x+1", "x": 100, "y": 100}));
    s.execute("object.move", &json!({"dx": 150, "dy": 40})).unwrap();
    let moved = bounds(&s, id);
    assert!((moved.center().x - 250.0).abs() < 1e-6 && (moved.center().y - 140.0).abs() < 1e-6);
    // No parameters: what is there.
    let cur = s.execute("math.edit", &json!({})).unwrap();
    assert_eq!((cur["id"].as_u64(), cur["source"].as_str(), cur["size"].as_f64()), (Some(id.0), Some("x+1"), Some(24.0)));
    // A new formula appears where the old one was moved to.
    let undo = s.doc().unwrap().history.undo.len();
    s.execute("math.edit", &json!({"source": r"x+1=\frac{a+b}{c}"})).unwrap();
    assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);
    assert_eq!(s.doc().unwrap().history.undo.last().unwrap().label, "Edit Equation");
    let after = bounds(&s, id);
    assert!((after.center().x - 250.0).abs() < 1e-6 && (after.center().y - 140.0).abs() < 1e-6, "{after:?}");
    assert!(after.width() > moved.width());
    let sp = spec(&s, id);
    assert_eq!(sp.source, r"x+1=\frac{a+b}{c}");
    assert!(near(sp.bounds, after));
    // A scale made on the canvas carries over (the same formula comes out the same size).
    s.execute("object.scale", &json!({"sx": 200, "sy": 200})).unwrap();
    let scaled = bounds(&s, id);
    assert!((scaled.width() / after.width() - 2.0).abs() < 1e-6);
    s.execute("math.edit", &json!({"display": false})).unwrap();
    assert!(!spec(&s, id).display);
    let again = bounds(&s, id);
    assert!((again.center().x - scaled.center().x).abs() < 1e-6 && (again.center().y - scaled.center().y).abs() < 1e-6);
    s.execute("math.edit", &json!({"display": true})).unwrap();
    assert!(near(bounds(&s, id), scaled));
    // Undo goes back one edit.
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(!spec(&s, id).display);
}

#[test]
fn edit_finds_the_equation_from_a_selected_shape_or_an_id_and_keeps_the_fill() {
    font_or_skip!();
    let mut s = session();
    let id = create(&mut s, json!({"source": "ab"}));
    let first = node(&s, id).children().unwrap()[0].id;
    s.execute("paint.setFill", &json!({"color": "#ff0000", "ids": [first.0]})).unwrap();
    // A shape inside stands for its equation; so does `id` with nothing selected.
    s.execute("select.set", &json!({"ids": [first.0]})).unwrap();
    s.execute("math.edit", &json!({"source": "abc"})).unwrap();
    s.execute("select.none", &json!({})).unwrap();
    s.execute("math.edit", &json!({"id": id.0, "size": 36})).unwrap();
    assert_eq!(spec(&s, id).size, 36.0);
    assert_eq!(spec(&s, id).source, "abc");
    let kids = node(&s, id).children().unwrap();
    assert!(kids.iter().all(|c| c.appearance.fill_paint().label() == node(&s, id).children().unwrap()[0].appearance.fill_paint().label()));
    assert_ne!(kids[0].appearance.fill_paint(), vectorcraft_color::Paint::solid(vectorcraft_color::Color::BLACK), "the red fill is kept");
    // Not an equation.
    let r = NodeId(s.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap()["id"].as_u64().unwrap());
    let err = s.execute("math.edit", &json!({"id": r.0, "source": "x"}));
    assert!(matches!(err, Err(EngineError::BadParams { .. })), "{err:?}");
}

/// The equation is saved with its source and opens again as an equation, still editable.
#[test]
fn an_equation_survives_save_and_open() {
    font_or_skip!();
    let mut s = session();
    let id = create(&mut s, json!({"source": r"e^{i\pi}+1=0", "size": 30, "display": false, "x": 200, "y": 150}));
    let doc = &s.doc().unwrap().doc;
    let bytes = vectorcraft_format::save(doc, false);
    let back = vectorcraft_format::load(&bytes).unwrap();
    assert_eq!(back.node(id).unwrap().extras().math, node(&s, id).extras().math);
    assert!(back.node(id).unwrap().extras().graph.is_none());
}

/// The generated children are real nodes of the document: ids are unique and the tree is sound.
#[test]
fn the_generated_tree_passes_the_document_checks() {
    font_or_skip!();
    let mut s = session();
    create(&mut s, json!({"source": r"\sqrt[3]{\frac{x_1^2+y}{\sin\theta}}"}));
    let id = create(&mut s, json!({"source": r"\int_0^1 x^2\,dx"}));
    s.execute("math.edit", &json!({"id": id.0, "source": r"\left(\frac{a}{b}\right)^2"})).unwrap();
    vectorcraft_testkit::invariants::check_document(&s.doc().unwrap().doc).unwrap();
}
