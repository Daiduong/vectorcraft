//! Rectangular Grid and Polar Grid tool options (double-click the tool): divider counts and skews,
//! frame or compound path, and Fill Grid. OK sets them with `tool.setOption` for the next drag or
//! click-to-size dialog; it does not draw a grid.

use serde_json::{Map, Value, json};

use super::{DialogSpec, form, run_and_close};
use crate::state::Dialog;
use crate::VectorcraftApp;

/// The dialog kind of the grid tools' Tool Options.
pub const KIND: &str = "gridOptions";

pub(super) const SPEC: DialogSpec = DialogSpec { heading, body, confirm, min_width: 420.0, ..DialogSpec::FORM };

/// The tools the dialog serves.
pub const TOOLS: [&str; 2] = ["rectangularGrid", "polarGrid"];

fn heading(d: &Dialog) -> String {
    let label = vectorcraft_tools::tool_info(&d.str("tool")).map_or("", |t| t.label);
    crate::i18n::fmt(tl!("{tool} Options"), &[("tool", tl!(label))])
}

/// The grids' divider counts and skews: (rows / concentric, columns / radial).
fn grid_keys(tool: &str) -> ([&'static str; 2], [&'static str; 2]) {
    if tool == "polarGrid" {
        (["concentric", "radial"], ["concentricSkew", "radialSkew"])
    } else {
        (["rows", "columns"], ["rowSkew", "columnSkew"])
    }
}

/// Open the options of grid tool `tool` with their current values.
pub fn open(app: &mut VectorcraftApp, tool: &str) -> Result<Value, String> {
    if !TOOLS.contains(&tool) {
        return Err(format!("`{tool}` isn't a grid tool"));
    }
    let o = app.session.tool_options_of(tool);
    let mut fields: Map<String, Value> = match o {
        Value::Object(m) => m,
        _ => Map::new(),
    };
    fields.insert("tool".into(), json!(tool));
    app.ui.dialog = Some(Dialog { kind: KIND.into(), fields });
    Ok(json!({ "dialog": KIND }))
}

fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    let tool = d.str("tool");
    let saved = d.kind.clone();
    d.kind = tool.clone();
    form::grid(ui, d, app.session.general_unit());
    let own = match tool.as_str() {
        "rectangularGrid" => ("frame", tl!("Use Outside Rectangle As Frame")),
        "polarGrid" => ("compound", tl!("Create Compound Path From Ellipses")),
        _ => {
            d.kind = saved;
            return false;
        }
    };
    ui.add_space(8.0);
    for (key, label) in [own, ("fill", tl!("Fill Grid"))] {
        form::check(ui, d, key, label);
    }
    d.kind = saved;
    false
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let tool = d.str("tool");
    let (counts, skews) = grid_keys(tool.as_str());
    let mut values = Map::new();
    for (count, skew) in counts.into_iter().zip(skews) {
        values.insert(count.into(), json!((d.f64(count, 5.0) as u64).min(999)));
        values.insert(skew.into(), json!(d.f64(skew, 0.0)));
    }
    let switch = if tool == "polarGrid" { "compound" } else { "frame" };
    values.insert(switch.into(), json!(d.bool(switch)));
    values.insert("fill".into(), json!(d.bool("fill")));
    run_and_close(app, "tool.setOption", json!({ "tool": tool, "values": values }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_doc::NodeKind;
    use vectorcraft_engine::Session;
    use vectorcraft_tools::{PointerEvent, PointerKind};

    fn app() -> VectorcraftApp {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("file.new", json!({"width": 600, "height": 600})).unwrap();
        app
    }

    fn painted(app: &mut VectorcraftApp) -> String {
        crate::tests_labels::painted_text(app, |app, ui| super::super::show(app, ui.ctx()))
    }

    #[test]
    fn tool_options_opens_for_each_grid_tool_and_ok_sets_options_without_drawing() {
        for (tool, count_labels) in [
            ("rectangularGrid", ["Horizontal Dividers:", "Vertical Dividers:", "Use Outside Rectangle As Frame"]),
            ("polarGrid", ["Concentric Dividers:", "Radial Dividers:", "Create Compound Path From Ellipses"]),
        ] {
            let mut app = app();
            app.run("tool.setOption", json!({"tool": tool, "values": {"rows": 4, "columns": 2, "concentric": 4, "radial": 2}})).unwrap();
            let objects = |app: &VectorcraftApp| app.session.doc().unwrap().doc.layers[0].children().map_or(0, |c| c.len());
            app.run("tool.options", json!({"tool": tool})).unwrap();
            assert_eq!(app.ui.dialog.as_ref().map(|d| d.kind.as_str()), Some(KIND));
            let text = painted(&mut app);
            for label in count_labels {
                assert!(text.contains(label), "{label} in {text}");
            }
            assert!(text.contains("Fill Grid"), "{text}");
            let d = app.ui.dialog.as_mut().unwrap();
            let (counts, _) = grid_keys(tool);
            d.fields.insert(counts[0].into(), json!(7));
            d.fields.insert(counts[1].into(), json!(3));
            super::super::confirm(&mut app).unwrap();
            assert!(app.ui.dialog.is_none());
            assert_eq!(objects(&app), 0, "OK from tool options draws nothing");
            let o = app.session.tool_options_of(tool);
            assert_eq!((o[counts[0]].as_u64(), o[counts[1]].as_u64()), (Some(7), Some(3)));
        }
    }

    #[test]
    fn options_from_the_toolbar_apply_to_the_next_grid_drag() {
        let mut app = app();
        app.run("tool.options", json!({"tool": "rectangularGrid"})).unwrap();
        let d = app.ui.dialog.as_mut().unwrap();
        d.fields.insert("rows".into(), json!(2));
        d.fields.insert("columns".into(), json!(4));
        super::super::confirm(&mut app).unwrap();
        app.select_tool("rectangularGrid");
        let view = app.view_info();
        for (kind, x, y) in [(PointerKind::Down, 50.0, 50.0), (PointerKind::Drag, 150.0, 150.0), (PointerKind::Up, 150.0, 150.0)] {
            crate::canvas::dispatch(&mut app, &PointerEvent::new(kind, x, y), view);
        }
        let doc = &app.session.doc().unwrap().doc;
        let grid = doc.layers[0].children().unwrap().last().expect("a grid");
        let NodeKind::Group { children, .. } = &grid.kind else { panic!("grid is a group") };
        // One frame path, two horizontal dividers and four vertical ones.
        assert_eq!(children.len(), 7);
    }
}
