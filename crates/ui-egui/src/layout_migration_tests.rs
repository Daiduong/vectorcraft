use super::*;
use craft_ui::docking::Side;
use serde_json::json;

/// `ui.json` as VectorCraft 0.16 (8bf090b25) saved it after: Swatches moved below Layers;
/// Color, then Stroke onto it, floated; Gradient floated with Transparency below it (a set);
/// Align floated and collapsed; Pathfinder floated with Transform below it, the set docked as
/// column 1; Libraries docked on the icon column; Symbols moved beside Swatches, then closed;
/// Swatches collapsed to its tab; the dock widened to 380; saved as workspace "Migration Test".
const SAVED_BY_0_16: &str = include_str!("../tests/fixtures/ui-before-canvas-layout.json");

fn read(saved: &Value) -> ReadPreferences {
    UiState::read_preferences(&serde_json::to_vec(saved).unwrap()).unwrap()
}

/// The dock of a layout in the current shape: the canvas beside a dock at the end edge.
fn dock(layout: &Layout<String>) -> &[Column<String>] {
    assert!(docking::valid(layout), "{layout:?}");
    let Some(Node::Split { first, second, .. }) = &layout.root else { panic!("expected the canvas split: {layout:?}") };
    assert!(matches!(&**first, Node::Tabs { panels, .. } if panels == &[CANVAS]));
    let Node::Dock { side: Side::End, columns } = &**second else { panic!("expected an end dock: {second:?}") };
    columns
}

fn groups(column: &Column<String>) -> Vec<Vec<&str>> {
    column.groups.iter().map(|g| g.panels.iter().map(String::as_str).collect()).collect()
}

fn window_groups(window: &Floating<String>) -> Vec<Vec<&str>> {
    let mut out = Vec::new();
    let mut visit = vec![&window.root];
    while let Some(node) = visit.pop() {
        match node {
            Node::Split { first, second, .. } => {
                visit.push(second);
                visit.push(first);
            }
            Node::Tabs { panels, .. } => out.push(panels.iter().map(String::as_str).collect()),
            _ => panic!("unexpected node {node:?}"),
        }
    }
    out
}

#[test]
fn a_layout_saved_before_the_canvas_joined_keeps_its_arrangement() {
    let saved: Value = serde_json::from_str(SAVED_BY_0_16).unwrap();
    let read = read(&saved);
    assert!(read.intact);
    let ui = read.ui;
    assert!(ui.floating_panels.is_empty() && ui.docking_icons.is_empty() && ui.docking_collapsed.is_empty());
    let layout = ui.docking.as_ref().expect("converted layout");
    let columns = dock(layout);
    assert_eq!(columns.len(), 3, "{columns:#?}");

    let main = &columns[0];
    assert_eq!(groups(main), [vec!["properties", "layers"], vec!["swatches"]]);
    assert_eq!(main.groups[0].active, 1, "Layers in front");
    assert!(main.groups[1].minimized, "Swatches collapsed to its tab");
    assert!(!main.groups[0].minimized && !main.iconic);
    assert_eq!(main.width, 380.0, "dock width");

    let icons = &columns[1];
    assert!(icons.iconic);
    assert_eq!(groups(icons)[0], ["libraries"], "dropped on the icon column, at its head");
    let iconic: Vec<&str> = groups(icons).concat();
    assert!(iconic.contains(&"colorGuide") && iconic.contains(&"brushes") && iconic.contains(&"variables"));
    for elsewhere in ["swatches", "symbols", "color", "stroke", "gradient", "transparency", "align", "pathfinder", "transform"] {
        assert!(!iconic.contains(&elsewhere), "{elsewhere} has a place of its own");
    }

    let docked = &columns[2];
    assert_eq!(groups(docked), [vec!["pathfinder"], vec!["transform"]], "set docked as a column");
    assert!(!docked.iconic);

    assert_eq!(layout.floating.len(), 3);
    let color = &layout.floating[0];
    assert_eq!(window_groups(color), [vec!["color", "stroke"]]);
    assert!(matches!(color.root, Node::Tabs { active: 1, .. }), "Stroke in front");
    assert_eq!(color.rect, [300.0, 120.0, 300.0, 400.0]);
    let set = &layout.floating[1];
    assert_eq!(window_groups(set), [vec!["gradient"], vec!["transparency"]], "floating set");
    assert_eq!(&set.rect[..3], &[640.0, 100.0, 300.0]);
    assert!(!set.minimized);
    let align = &layout.floating[2];
    assert_eq!(window_groups(align), [vec!["align"]]);
    assert!(align.minimized, "collapsed floating group");
    assert_eq!(&align.rect[..2], &[300.0, 560.0]);

    // Symbols stays closed and goes back beside Swatches. The floating panels went to the icon
    // column from where they came, so they keep no place in the dock.
    assert!(!layout.contains(&"symbols".to_string()));
    assert_eq!(ui.docking_hidden.keys().collect::<Vec<_>>(), ["symbols"]);
    assert_eq!(ui.docking_hidden["symbols"].anchor.as_deref(), Some("swatches"));

    let workspace = ui.custom_workspaces.iter().find(|w| w.name == "Migration Test").expect("custom workspace");
    assert_eq!(workspace.docking, ui.docking);
    assert_eq!(workspace.docking_hidden, ui.docking_hidden);
    assert!(workspace.floating_panels.is_empty() && workspace.docking_icons.is_empty());
}

#[test]
fn a_converted_file_saved_again_has_no_legacy_fields_and_loads_the_same() {
    let saved: Value = serde_json::from_str(SAVED_BY_0_16).unwrap();
    let ui = read(&saved).ui;
    let again = serde_json::to_value(&ui).unwrap();
    for key in ["floating_panels", "docking_icons", "docking_collapsed"] {
        assert!(again.get(key).is_none(), "{key} written");
    }
    for key in ["floatingPanels", "dockingIcons", "dockingCollapsed"] {
        assert!(again["custom_workspaces"][0].get(key).is_none(), "{key} written");
    }
    let reread = read(&again);
    assert!(reread.intact);
    assert_eq!(reread.ui.docking, ui.docking);
    assert_eq!(reread.ui.docking_hidden, ui.docking_hidden);
    assert_eq!(reread.ui.custom_workspaces, ui.custom_workspaces);
    let ids = |ui: &UiState| ui.docking.iter().flat_map(|l| l.floating.iter().map(|w| w.id)).collect::<Vec<_>>();
    assert_eq!(ids(&reread.ui), ids(&ui), "floating windows keep their ids");
}

#[test]
fn a_file_from_before_the_shared_dock_converts_and_remembered_changes_too() {
    let ui = read(&json!({
        "dock_tab": "Libraries", "dock_collapsed": true, "dock_width": 400.0, "workspace": "Essentials",
        "floating_panels": [{"panels": ["layers"], "active": 0, "pos": [10.0, 20.0]}],
        "docking_icons": ["properties"],
        "workspace_changes": [{
            "name": "Essentials", "dockTab": "Layers",
            "floatingPanels": [{"panels": ["swatches", "brushes"], "active": 1, "pos": [50.0, 60.0], "width": 260.0, "collapsed": true}],
        }],
    }))
    .ui;
    let layout = ui.docking.as_ref().unwrap();
    let columns = dock(layout);
    assert_eq!(groups(&columns[0]), [vec!["libraries"]]);
    assert!(columns[0].iconic, "collapsed dock");
    assert_eq!(columns[0].width, 400.0);
    assert_eq!(groups(&columns[1])[0], ["properties"]);
    assert_eq!(window_groups(&layout.floating[0]), [vec!["layers"]]);
    assert_eq!(layout.floating[0].rect, [10.0, 20.0, 300.0, 400.0]);

    let changed = ui.workspace_changes[0].docking.as_ref().unwrap();
    let columns = dock(changed);
    assert_eq!(groups(&columns[0]), [vec!["properties", "layers", "libraries"]]);
    assert_eq!(columns[0].groups[0].active, 1);
    assert!(!columns[0].iconic);
    assert!(groups(&columns[1]).contains(&vec!["symbols", "patternOptions"]));
    assert_eq!(window_groups(&changed.floating[0]), [vec!["swatches", "brushes"]]);
    assert!(matches!(changed.floating[0].root, Node::Tabs { active: 1, .. }));
    assert!(changed.floating[0].minimized);
    assert_eq!(changed.floating[0].rect, [50.0, 60.0, 260.0, 400.0]);

    // Nothing to convert: the flags still describe the workspace's default layout.
    let ui = read(&json!({"dock_tab": "Layers", "dock_collapsed": true, "dock_width": 400.0})).ui;
    assert!(ui.docking.is_none());
    assert_eq!(ui.dock_width, Some(400.0));
}

#[test]
fn unreadable_docking_data_is_dropped_and_reported() {
    let saved: Value = serde_json::from_str(SAVED_BY_0_16).unwrap();
    for (key, value) in [
        ("docking", json!({"root": {"FutureNode": {}}, "floating": []})),
        ("docking_hidden", json!(["malformed"])),
        ("floating_panels", json!([{"panels": "color"}])),
        ("docking_icons", json!(7)),
    ] {
        let mut broken = saved.clone();
        broken[key] = value.clone();
        broken["custom_workspaces"][0]["docking"] = json!(false);
        let read = read(&broken);
        assert!(!read.intact, "{key}");
        assert_eq!(read.ui.workspace, "Migration Test", "the rest is kept");
        assert_eq!(read.ui.custom_workspaces.len(), 1);
        assert!(docking::valid(read.ui.docking.as_ref().unwrap()), "{key}: what is left still converts");
    }
    assert!(UiState::read_preferences(b"[1, 2]").is_none());
    assert!(UiState::read_preferences(br#"{"toolbar": "invalid"}"#).is_none(), "other fields stay strict");
}

/// A hand-edited file: unknown and repeated panels, sizes out of range, more groups than any
/// layout holds.
#[test]
fn untrusted_legacy_fields_convert_without_failing() {
    let mut floating: Vec<Value> =
        (0..2000).map(|k| json!({"panels": ["nosuch", "color"], "active": k, "pos": [1e39, -1e39], "column": k % 3})).collect();
    floating.insert(0, json!({"panels": ["stroke"], "active": 99, "pos": [0.0, 0.0], "width": 1e30, "docked": 4000000000u32}));
    let ui = read(&json!({
        "dock_width": -5.0,
        "floating_panels": floating,
        "docking_icons": ["nosuch", "glyphs", "tabs"],
        "docking_collapsed": ["properties", "nosuch"],
        "docking": {"root": {"Tabs": {"panels": ["properties", "properties", "nosuch"], "active": 9}}, "floating": []},
    }))
    .ui;
    let layout = ui.docking.as_ref().unwrap();
    let columns = dock(layout);
    assert_eq!(groups(&columns[0]), [vec!["properties"]]);
    assert!(columns[0].groups[0].minimized);
    assert!(columns[0].width.is_finite());
    assert!(groups(&columns[1]).last().is_some_and(|g| g == &["tabs"]), "a panel outside the rows dropped on the icon column");
    assert_eq!(groups(columns.last().unwrap()), [vec!["stroke"]]);
    assert!(columns.last().unwrap().width <= COLUMN_MAX);
    assert_eq!(layout.floating.len(), 1);
    assert!(layout.floating[0].rect.iter().all(|v| v.is_finite()));
}
