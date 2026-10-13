//! Shape size dialogs (a click with a shape tool): Rectangle, Rounded Rectangle, Ellipse, Polygon,
//! Star, Line Segment Tool Options and the two grids (size, divider counts and skews, switches).

use serde_json::{Value, json};

use super::{DialogSpec, form, run_and_close};
use crate::VectorcraftApp;
use crate::state::Dialog;

pub(super) const SPEC: DialogSpec = DialogSpec { heading: |d| title(&d.kind).into(), body, confirm, ..DialogSpec::FORM };

fn title(kind: &str) -> &'static str {
    match kind {
        "rectangle" => tl!("Rectangle"),
        "roundedRectangle" => tl!("Rounded Rectangle"),
        "ellipse" => tl!("Ellipse"),
        "polygon" => tl!("Polygon"),
        "star" => tl!("Star"),
        "rectangularGrid" => tl!("Rectangular Grid"),
        "polarGrid" => tl!("Polar Grid"),
        _ => tl!("Line Segment Tool Options"),
    }
}

/// The grids' divider counts and skews: (rows / concentric, columns / radial).
fn grid_keys(kind: &str) -> ([&'static str; 2], [&'static str; 2]) {
    if kind == "polarGrid" { (["concentric", "radial"], ["concentricSkew", "radialSkew"]) } else { (["rows", "columns"], ["rowSkew", "columnSkew"]) }
}

/// The form's fields, then a grid's switches.
fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    form::grid(ui, d, app.session.general_unit());
    let own = match d.kind.as_str() {
        "rectangularGrid" => ("frame", tl!("Use Outside Rectangle As Frame")),
        "polarGrid" => ("compound", tl!("Create Compound Path From Ellipses")),
        _ => return false,
    };
    ui.add_space(8.0);
    for (key, label) in [own, ("fill", tl!("Fill Grid"))] {
        form::check(ui, d, key, label);
    }
    false
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let (x, y) = (d.f64("x", 0.0), d.f64("y", 0.0));
    // Alt held at the click: the shape is drawn from its centre at the clicked point.
    let centre = |w: f64, h: f64| (x - w / 2.0, y - h / 2.0);
    let (id, params) = match d.kind.as_str() {
        "ellipse" => {
            let (w, h) = (d.f64("width", 100.0), d.f64("height", 100.0));
            let (x, y) = if d.bool("center") { centre(w, h) } else { (x, y) };
            ("shape.ellipse", json!({"x": x, "y": y, "width": w, "height": h}))
        }
        "polygon" => ("shape.polygon", json!({"cx": x, "cy": y, "radius": d.f64("radius", 50.0), "sides": d.f64("sides", 6.0) as u64})),
        "star" => (
            "shape.star",
            json!({"cx": x, "cy": y, "radius1": d.f64("radius1", 50.0), "radius2": d.f64("radius2", 25.0), "points": d.f64("points", 5.0) as u64}),
        ),
        kind @ ("rectangularGrid" | "polarGrid") => {
            // The switches as they are; the typed fields as numbers.
            let (mut p, (counts, skews)) = (form::params(d), grid_keys(kind));
            for (k, default) in [("width", 100.0), ("height", 100.0)] {
                p[k] = json!(d.f64(k, default));
            }
            if d.bool("center") {
                let (bx, by) = centre(d.f64("width", 100.0), d.f64("height", 100.0));
                p["x"] = json!(bx);
                p["y"] = json!(by);
            }
            for (count, skew) in counts.into_iter().zip(skews) {
                p[count] = json!((d.f64(count, 5.0) as u64).min(999));
                p[skew] = json!(d.f64(skew, 0.0));
            }
            (if kind == "polarGrid" { "shape.polarGrid" } else { "shape.rectangularGrid" }, p)
        }
        "lineSegment" => {
            let (l, a) = (d.f64("length", 100.0), d.f64("angle", 0.0).to_radians());
            if d.bool("center") {
                let (dx, dy) = (l * a.cos() / 2.0, l * a.sin() / 2.0);
                ("shape.line", json!({"x1": x - dx, "y1": y + dy, "x2": x + dx, "y2": y - dy}))
            } else {
                ("shape.line", json!({"x1": x, "y1": y, "x2": x + l * a.cos(), "y2": y - l * a.sin()}))
            }
        }
        _ => {
            let (w, h) = (d.f64("width", 100.0), d.f64("height", 100.0));
            let (x, y) = if d.bool("center") { centre(w, h) } else { (x, y) };
            ("shape.rectangle", json!({"x": x, "y": y, "width": w, "height": h, "radius": d.f64("radius", 0.0)}))
        }
    };
    // On the active plane while the perspective grid shows, sized in plane units from the click.
    let options = matches!(d.kind.as_str(), "rectangularGrid" | "polarGrid").then(|| params.clone());
    let result = match vectorcraft_engine::perspective_click(&app.session, id, &params, vectorcraft_geom::Point::new(x, y)) {
        Some((id, params)) => run_and_close(app, &id, params),
        None => run_and_close(app, id, params),
    }?;
    if let Some(options) = options {
        app.run("tool.setOption", json!({"tool": d.kind, "values": options}))?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_doc::NodeId;
    use vectorcraft_engine::Session;

    /// The bounds centre of the object `id`.
    fn centre(app: &VectorcraftApp, id: u64) -> (f64, f64) {
        let b = app.session.doc().unwrap().doc.bounds_of(&[NodeId(id)], false).unwrap();
        (b.center().x, b.center().y)
    }

    /// Alt held at the click: the size dialogs draw their shape from its centre at the clicked
    /// point; without it, the click stays the top-left corner as before.
    #[test]
    fn alt_click_dialogs_draw_from_the_clicks_centre() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("file.new", json!({"width": 400, "height": 400})).unwrap();
        crate::dialogs::tools::open_tool_dialog(&mut app, "ellipse", json!({"x": 200.0, "y": 120.0, "center": true}));
        let d = app.ui.dialog.take().unwrap();
        assert!(d.bool("center"));
        let id = confirm(&mut app, &d).unwrap()["id"].as_u64().unwrap();
        let (cx, cy) = centre(&app, id);
        assert!((cx - 200.0).abs() < 1e-9 && (cy - 120.0).abs() < 1e-9, "{cx} {cy}");
        // Without Alt: the click stays the top-left corner.
        crate::dialogs::tools::open_tool_dialog(&mut app, "rectangle", json!({"x": 40.0, "y": 30.0, "center": false}));
        let d = app.ui.dialog.take().unwrap();
        let id = confirm(&mut app, &d).unwrap()["id"].as_u64().unwrap();
        let b = app.session.doc().unwrap().doc.bounds_of(&[NodeId(id)], false).unwrap();
        assert!((b.x0 - 40.0).abs() < 1e-9 && (b.y0 - 30.0).abs() < 1e-9, "{b:?}");
        // A grid dialog Alt+clicked lands centred too (its fields come round by way of the tool's
        // stored options).
        crate::dialogs::tools::open_tool_dialog(&mut app, "rectangularGrid", json!({"x": 150.0, "y": 250.0, "center": true}));
        let d = app.ui.dialog.take().unwrap();
        assert!(d.bool("center"));
        let id = confirm(&mut app, &d).unwrap()["id"].as_u64().unwrap();
        let (cx, cy) = centre(&app, id);
        assert!((cx - 150.0).abs() < 1e-9 && (cy - 250.0).abs() < 1e-9, "{cx} {cy}");
    }
}
