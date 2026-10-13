//! The infinite canvas (#1087): an infinite document takes coordinates to its own limit, far past a
//! standard one's, and starts with no artboards.

use serde_json::json;
use vectorcraft_doc::{CanvasKind, ColorMode};

use super::*;

/// A session with an infinite document open.
pub(crate) fn infinite() -> Session {
    let mut s = Session::new();
    s.add_document(Document::new_infinite(ColorMode::Rgb), None);
    s
}

/// A rectangle drawn at `x` (points), or the error.
fn rect_at(s: &mut Session, x: f64) -> std::result::Result<serde_json::Value, EngineError> {
    s.execute("shape.rectangle", &json!({"x": x, "y": 0, "width": 10, "height": 10}))
}

#[test]
fn an_infinite_document_reaches_past_the_standard_canvas() {
    let mut s = infinite();
    let d = &s.doc().unwrap().doc;
    assert_eq!((d.canvas, d.artboards.len(), d.coord_limit()), (CanvasKind::Infinite, 0, 1.0e9));
    // 5,000,000 pt is past a standard canvas, and 900,000,000 pt is near the infinite limit.
    rect_at(&mut s, 5.0e6).expect("past the standard canvas");
    rect_at(&mut s, 9.0e8).expect("near the infinite limit");
    let err = rect_at(&mut s, 2.0e9).expect_err("past the infinite limit");
    assert!(err.to_string().contains("exceed the canvas"), "{err}");
    // A guide and the ruler origin go as far.
    s.execute("document.setRulerOrigin", &json!({"x": 6.0e8, "y": -6.0e8})).unwrap();
    assert_eq!(s.doc().unwrap().doc.ruler_origin, vectorcraft_geom::Point::new(6.0e8, -6.0e8));

    // A standard document keeps its limit.
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().doc.canvas, CanvasKind::Standard);
    let err = rect_at(&mut s, 5.0e6).expect_err("past the standard canvas");
    assert!(err.to_string().contains("exceed the canvas"), "{err}");
    assert!(s.execute("document.setRulerOrigin", &json!({"x": 6.0e8, "y": 0})).is_err());
}

/// New Document: `canvas: "infinite"` (or an Infinite Canvas preset) makes an endless sheet with no
/// artboards; its size settings don't apply. Recent remembers it once per units.
#[test]
fn file_new_makes_an_infinite_canvas() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"canvas": "infinite", "width": -5, "artboards": 9, "units": "Millimeters"})).unwrap();
    let d = &s.doc().unwrap().doc;
    assert_eq!((d.canvas, d.artboards.len(), d.units), (CanvasKind::Infinite, 0, vectorcraft_doc::Unit::Millimeters));
    assert_eq!(d.layers.len(), 1, "one layer to draw on");
    s.execute("file.new", &json!({"preset": "Infinite Canvas (Print)"})).unwrap();
    let d = &s.doc().unwrap().doc;
    assert_eq!((d.canvas, d.color_mode, d.raster_effects_ppi), (CanvasKind::Infinite, ColorMode::Cmyk, 300.0));
    // The category is listed after Recent and Saved, with sizes shown as "Infinite".
    let cats = s.execute("file.newPresets", &json!({})).unwrap();
    let names: Vec<&str> = cats["categories"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert_eq!(&names[..3], ["Recent", "Saved", "Infinite Canvas"]);
    let inf = s.execute("file.newPresets", &json!({"category": "infinite canvas"})).unwrap();
    let preset = &inf["categories"][0]["presets"][0];
    assert_eq!((preset["canvas"].as_str(), preset["size"].as_str()), (Some("infinite"), Some("Infinite")));
    // Recent: the infinite canvases, newest first, then the standard ones.
    let recent: Vec<String> = s.prefs.recent_new_docs.iter().map(|r| format!("{}/{:?}", r.canvas.key(), r.units)).collect();
    assert_eq!(recent.iter().filter(|r| r.starts_with("infinite")).count(), 2, "{recent:?}");
    s.execute("file.new", &json!({"canvas": "infinite", "units": "Millimeters"})).unwrap();
    assert_eq!(s.prefs.recent_new_docs.iter().filter(|r| r.canvas.is_infinite()).count(), 2, "one entry per units");
    // A bad value is refused; a standard canvas is the default.
    for bad in [json!("bounded"), json!(7), json!(true)] {
        assert!(s.execute("file.new", &json!({"canvas": bad})).is_err(), "{bad}");
    }
    s.execute("file.new", &json!({"canvas": "standard"})).unwrap();
    assert_eq!(s.doc().unwrap().doc.canvas, CanvasKind::Standard);
}

/// Document Setup switches the canvas. To infinite always; back to standard only when everything
/// lies within the standard limit, and then an artboard goes around the art.
#[test]
fn document_setup_switches_the_canvas() {
    let mut s = infinite();
    assert_eq!(s.execute("document.setup", &json!({})).unwrap()["canvas"], json!("infinite"));
    assert_eq!(crate::inspect::document(&s)["canvas"], json!("infinite"));
    rect_at(&mut s, 100.0).unwrap();
    rect_at(&mut s, 5.0e6).unwrap();
    let err = s.execute("document.setup", &json!({"canvas": "standard"})).expect_err("art past the standard canvas");
    assert!(err.to_string().contains("beyond the standard canvas"), "{err}");
    assert_eq!(s.doc().unwrap().doc.canvas, CanvasKind::Infinite, "nothing changed");
    s.execute("edit.undo", &json!({})).unwrap();
    // Now the art fits: one artboard goes around it, in one undo step.
    let out = s.execute("document.setup", &json!({"canvas": "standard"})).unwrap();
    assert_eq!(out["canvas"], json!("standard"));
    let d = &s.doc().unwrap().doc;
    assert_eq!(d.artboards.len(), 1);
    // Around the art as drawn: the 1 pt stroke reaches half a point past the path.
    assert_eq!(d.artboards[0].rect, vectorcraft_geom::Rect::new(99.5, -0.5, 110.5, 10.5));
    s.execute("edit.undo", &json!({})).unwrap();
    let d = &s.doc().unwrap().doc;
    assert_eq!((d.canvas, d.artboards.len()), (CanvasKind::Infinite, 0));
    // A standard canvas becomes infinite and keeps its artboards.
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("document.setup", &json!({"canvas": "infinite"})).unwrap();
    let d = &s.doc().unwrap().doc;
    assert_eq!((d.canvas, d.artboards.len()), (CanvasKind::Infinite, 1));
    assert!(s.execute("document.setup", &json!({"canvas": "round"})).is_err());
}

/// With no artboards, exports take the art's bounds: PNG, PDF and SVG. The rulers' zero point
/// resets to the origin, and the last artboard can be deleted again.
#[test]
fn exports_and_artboard_commands_work_without_artboards() {
    let mut s = infinite();
    assert!(s.execute("document.export", &json!({"format": "png"})).is_err(), "no art yet");
    s.execute("shape.rectangle", &json!({"x": 1.0e6, "y": -2.0e6, "width": 100, "height": 50})).unwrap();
    for format in ["png", "pdf", "svg"] {
        let out = s.execute("document.export", &json!({"format": format})).unwrap_or_else(|e| panic!("{format}: {e}"));
        assert!(out.to_string().len() > 10, "{format}: {out}");
    }
    s.execute("document.exportPdf", &json!({})).expect("a PDF page around the art");
    // Ruler origin: the origin without artboards.
    s.execute("document.setRulerOrigin", &json!({"x": 5, "y": 5})).unwrap();
    s.execute("document.resetRulerOrigin", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().doc.ruler_origin, vectorcraft_geom::Point::ZERO);
    // A first artboard at the origin, then delete it: an infinite canvas may have none.
    s.execute("artboard.new", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().doc.artboards[0].rect, vectorcraft_geom::Rect::new(0.0, 0.0, 612.0, 792.0));
    s.execute("artboard.delete", &json!({"index": 0})).unwrap();
    assert!(s.doc().unwrap().doc.artboards.is_empty());
    // A standard canvas still keeps its last one.
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    assert!(s.execute("artboard.delete", &json!({"index": 0})).is_err());
}

/// Helpers without the document (area type frames) check against the document's own limit: an
/// area-type anchor moves 10,000,000 pt on an infinite canvas, and stays refused on a standard one.
#[test]
fn area_type_follows_the_documents_limit() {
    let area = |s: &mut Session| {
        let r = s.execute("text.create", &json!({"x": 40, "y": 40, "size": 12, "area": {"width": 120, "height": 40}, "text": "Far"})).unwrap();
        r["id"].as_u64().unwrap()
    };
    let reshape = |id: u64| json!({"id": id, "anchors": [[0, 2]], "dx": -1.0e7, "dy": 0});
    let mut s = infinite();
    let id = area(&mut s);
    s.execute("text.reshapeArea", &reshape(id)).expect("past a standard canvas, on an infinite one");
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    let id = area(&mut s);
    assert!(s.execute("text.reshapeArea", &reshape(id)).is_err(), "a standard canvas keeps its limit");
}
