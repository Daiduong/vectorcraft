//! Headless frames of the Appearance panel and the panels that follow its active item.

use serde_json::{Value, json};
use vectorcraft_doc::Unit;
use vectorcraft_engine::Session;

use super::*;

pub(super) fn app_with_rect() -> VectorcraftApp {
    let mut app = VectorcraftApp::new(Session::new(), Default::default());
    run(&mut app, "file.new", json!({"width": 200, "height": 200}));
    let id = run(&mut app, "shape.rectangle", json!({"x": 10, "y": 10, "width": 100, "height": 100}))["id"].clone();
    run(&mut app, "select.set", json!({ "ids": [id] }));
    app
}

pub(super) fn run(app: &mut VectorcraftApp, id: &str, p: Value) -> Value {
    app.session.execute(id, &p).unwrap_or_else(|e| panic!("{id}: {e}"))
}

/// Every text drawn while `f` runs one headless frame.
fn frame_texts(app: &mut VectorcraftApp, f: impl FnMut(&mut VectorcraftApp, &mut egui::Ui)) -> Vec<String> {
    let ctx = egui::Context::default();
    frame_in(&ctx, app, f)
}

fn frame_in(ctx: &egui::Context, app: &mut VectorcraftApp, f: impl FnMut(&mut VectorcraftApp, &mut egui::Ui)) -> Vec<String> {
    frame_events(ctx, app, vec![], f).into_iter().map(|(s, _)| s).collect()
}

/// One headless frame with input `events`: every text drawn, with its screen rect.
pub(super) fn frame_events(
    ctx: &egui::Context,
    app: &mut VectorcraftApp,
    events: Vec<egui::Event>,
    f: impl FnMut(&mut VectorcraftApp, &mut egui::Ui),
) -> Vec<(String, egui::Rect)> {
    frame_raw(ctx, app, egui::RawInput { events, ..Default::default() }, f)
}

/// One headless frame with input `raw`: every text drawn, with its screen rect.
pub(super) fn frame_raw(
    ctx: &egui::Context,
    app: &mut VectorcraftApp,
    raw: egui::RawInput,
    mut f: impl FnMut(&mut VectorcraftApp, &mut egui::Ui),
) -> Vec<(String, egui::Rect)> {
    let mut out = ctx.run_ui(raw, |ui| f(app, ui));
    out.textures_delta.clear();
    fn collect(s: &egui::Shape, out: &mut Vec<(String, egui::Rect)>) {
        match s {
            egui::Shape::Text(t) => out.push((t.galley.text().to_string(), t.visual_bounding_rect())),
            egui::Shape::Vec(v) => v.iter().for_each(|s| collect(s, out)),
            _ => {}
        }
    }
    let mut texts = vec![];
    for c in &out.shapes {
        collect(&c.shape, &mut texts);
    }
    texts
}

/// Click at `pos` over three frames of `f` (hover, press, release).
pub(super) fn click(ctx: &egui::Context, app: &mut VectorcraftApp, pos: egui::Pos2, mut f: impl FnMut(&mut VectorcraftApp, &mut egui::Ui)) {
    let button = |pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
    for e in [egui::Event::PointerMoved(pos), button(true), button(false)] {
        frame_events(ctx, app, vec![e], &mut f);
    }
}

/// The rect of the first drawn text equal to `s`.
pub(super) fn text_rect(texts: &[(String, egui::Rect)], s: &str) -> egui::Rect {
    texts.iter().find(|(t, _)| t == s).unwrap_or_else(|| panic!("`{s}` not drawn: {texts:?}")).1
}

#[test]
fn stroke_panel_shows_the_active_rows_weight() {
    let mut app = app_with_rect();
    run(&mut app, "appearance.addStroke", json!({})); // [Fill, Stroke 1 pt, Stroke 1 pt]
    run(&mut app, "stroke.set", json!({"item": 1, "weight": 6}));
    let six = Unit::Points.format(6.0);
    let texts = frame_texts(&mut app, stroke::show);
    assert!(!texts.contains(&six), "top stroke shown: {texts:?}");
    // Clicking the lower stroke row in the Appearance panel targets it.
    let ctx = egui::Context::default();
    frame_in(&ctx, &mut app, appearance::show);
    appearance::select_row(&mut app, &ctx, appearance::Sel::Item(1));
    assert_eq!(app.session.appearance_item(), Some(1));
    assert!(frame_texts(&mut app, stroke::show).contains(&six));
    let (_, stroke) = current_paints(&app);
    assert_eq!(stroke, current_stroke(&app).unwrap().paint);
    // The Appearance panel and the proxies draw with the row active; the object row clears it.
    frame_in(&ctx, &mut app, |app, ui| {
        appearance::show(app, ui);
        crate::toolbar::show(app, ui);
        transparency::show(app, ui);
        properties::show(app, ui);
    });
    appearance::select_row(&mut app, &ctx, appearance::Sel::None);
    assert_eq!(app.session.appearance_item(), None);
    assert!(!frame_texts(&mut app, stroke::show).contains(&six));
}

/// Cmd+/ is Add New Fill and Cmd+Alt+/ is Add New Stroke by default. On Windows and Linux the keys
/// are Ctrl+/ and Ctrl+Alt+/, and on macOS they are Command+/ and Option+Command+/. No other
/// command, tool or panel has either key by default.
#[test]
fn cmd_slash_is_add_new_fill_and_cmd_alt_slash_add_new_stroke() {
    use crate::shortcut_editor::{command_for_key, default_command_shortcut};
    let mac = cfg!(target_os = "macos");
    let (cmd, alt) = (egui::Modifiers::COMMAND, egui::Modifiers::ALT);
    let mut app = VectorcraftApp::new(Session::new(), Default::default());
    for (id, key, label, mods, shown) in [
        ("appearance.addFill", "Cmd+/", "Add New Fill", cmd, if mac { "⌘/" } else { "Ctrl+/" }),
        ("appearance.addStroke", "Cmd+Alt+/", "Add New Stroke", cmd | alt, if mac { "⌥⌘/" } else { "Ctrl+Alt+/" }),
    ] {
        assert_eq!(default_command_shortcut(id), Some(key));
        assert_eq!(command_for_key(key), Some(id));
        let chord = crate::shortcuts::parse(key).expect("parses");
        assert_eq!((chord.modifiers, chord.logical_key), (mods, egui::Key::Slash));
        assert_eq!(crate::menus::pretty_shortcut(key), shown);
        let listed = app.run("shortcuts.list", json!({ "query": id })).unwrap();
        let rows: Vec<_> = listed.as_array().expect("rows").iter().map(|r| (r["label"].clone(), r["group"].clone(), r["default"].clone())).collect();
        assert_eq!(rows, [(json!(label), json!("Window › Appearance"), json!(key))], "Keyboard Shortcuts lists {id}");
    }
    assert_eq!(app.run("shortcuts.conflicts", json!({})).unwrap(), json!([]), "no other default has either key");
}

/// Cmd+/ and Cmd+Alt+/, pressed through the shortcut handler, add a fill and a stroke to the
/// selected objects as the Appearance panel's Add New Fill and Add New Stroke buttons do, on a path
/// and on type. With nothing selected the buttons are off, and the keys leave the objects as they are.
#[test]
fn cmd_slash_adds_a_fill_and_cmd_alt_slash_a_stroke_as_the_panel_buttons_do() {
    fn press(app: &mut VectorcraftApp, modifiers: egui::Modifiers) {
        let ctx = egui::Context::default();
        let ev = egui::Event::Key { key: egui::Key::Slash, physical_key: None, pressed: true, repeat: false, modifiers };
        let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| crate::shortcuts::handle(app, ui.ctx()));
        out.textures_delta.clear();
    }
    /// The Appearance panel's button named `label`: the center of its rect and whether it is on.
    fn button(ctx: &egui::Context, app: &mut VectorcraftApp, label: &str) -> (egui::Pos2, bool) {
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| appearance::show(app, ui));
        out.textures_delta.clear();
        let nodes = &out.platform_output.accesskit_update.as_ref().expect("an accessibility tree").nodes;
        let (_, n) = nodes
            .iter()
            .find(|(_, n)| n.label() == Some(label) && n.role() == egui::accesskit::Role::Button)
            .unwrap_or_else(|| panic!("no button named {label:?}"));
        let b = n.bounds().expect("bounds");
        (egui::pos2((b.x0 + b.x1) as f32 * 0.5, (b.y0 + b.y1) as f32 * 0.5), !n.is_disabled())
    }
    /// A rectangle and a line of type, with the rectangle (`type_too`: both) selected.
    fn two_objects(type_too: bool) -> (VectorcraftApp, Vec<Value>) {
        let mut app = app_with_rect();
        let rect = first_selected(&app).expect("the rectangle").id.0;
        let text = run(&mut app, "text.create", json!({"x": 10, "y": 150, "text": "Hi"}))["id"].clone();
        let ids = if type_too { vec![json!(rect), text.clone()] } else { vec![json!(rect)] };
        run(&mut app, "select.set", json!({ "ids": ids }));
        (app, vec![json!(rect), text])
    }
    let stacks = |app: &VectorcraftApp, ids: &[Value]| -> Vec<vectorcraft_doc::Appearance> {
        let d = &app.session.active().expect("a document").doc;
        ids.iter().map(|id| (*d.node(vectorcraft_doc::NodeId(id.as_u64().expect("an id"))).expect("the object").appearance).clone()).collect()
    };
    let cmd = egui::Modifiers::COMMAND;
    for (type_too, mods, label, fill) in [
        (false, cmd, "Add New Fill", true),
        (false, cmd | egui::Modifiers::ALT, "Add New Stroke", false),
        (true, cmd, "Add New Fill", true),
        (true, cmd | egui::Modifiers::ALT, "Add New Stroke", false),
    ] {
        let (mut keyed, ids) = two_objects(type_too);
        let before = stacks(&keyed, &ids);
        press(&mut keyed, mods);
        let (mut clicked, _) = two_objects(type_too);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let (at, on) = button(&ctx, &mut clicked, label);
        assert!(on, "{label} is on with a selection");
        click(&ctx, &mut clicked, at, appearance::show);
        let after = stacks(&keyed, &ids);
        assert_eq!(after, stacks(&clicked, &ids), "{label}: the key adds what the button adds");
        let selected = if type_too { 2 } else { 1 };
        for (i, (a, b)) in after.iter().zip(&before).enumerate() {
            let added = if i < selected { 1 } else { 0 };
            assert_eq!(a.items.len(), b.items.len() + added, "{label}, object {i}: {a:?}");
            if added == 1 {
                assert_eq!(a.items.last().map(|it| it.is_fill()), Some(fill), "{label}: on top of the stack");
            }
        }
    }
    // With nothing selected, both buttons are off and neither key changes the objects.
    let (mut app, ids) = two_objects(true);
    run(&mut app, "select.none", json!({}));
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    assert!(!button(&ctx, &mut app, "Add New Fill").1 && !button(&ctx, &mut app, "Add New Stroke").1);
    let before = stacks(&app, &ids);
    press(&mut app, cmd);
    press(&mut app, cmd | egui::Modifiers::ALT);
    assert_eq!(stacks(&app, &ids), before);
}

#[test]
fn transparency_panel_reads_the_active_item() {
    let mut app = app_with_rect();
    run(&mut app, "transparency.set", json!({"item": 0, "opacity": 40}));
    assert_eq!(current_transparency(&app).unwrap().0, 1.0);
    run(&mut app, "appearance.setActiveItem", json!({"index": 0}));
    assert!((current_transparency(&app).unwrap().0 - 0.4).abs() < 1e-6);
    assert!(frame_texts(&mut app, transparency::show).iter().any(|t| t.starts_with("40")));
}

#[test]
fn item_effect_rows_toggle_their_eye() {
    let mut app = app_with_rect();
    run(&mut app, "effect.apply", json!({"effect": "distort.roughen", "item": 1}));
    let ctx = egui::Context::default();
    // The stroke row's disclosure is open: its Opacity sub-row and its effect show under it.
    super::set_pstate(&ctx, "ap-open-1", true);
    let texts = frame_events(&ctx, &mut app, vec![], appearance::show);
    let label = text_rect(&texts, "Roughen");
    // The eye sits in the first column of the effect's row (the label is indented one level).
    let eye = egui::pos2(label.left() - 12.0 - 24.0 - appearance::EYE_W / 2.0, label.center().y);
    click(&ctx, &mut app, eye, appearance::show);
    let visible = |app: &VectorcraftApp| current_stroke_effects(app)[0].visible;
    assert!(!visible(&app));
    click(&ctx, &mut app, eye, appearance::show);
    assert!(visible(&app));
    // Clicking the effect's row selects it and makes its stroke the active item.
    click(&ctx, &mut app, egui::pos2(label.right() + 40.0, label.center().y), appearance::show);
    assert_eq!(app.session.appearance_item(), Some(1));
}

fn current_stroke_effects(app: &VectorcraftApp) -> Vec<vectorcraft_doc::Effect> {
    first_selected(app).unwrap().appearance.items[1].effects().clone()
}

#[test]
fn blend_separators_follow_the_groups() {
    use vectorcraft_color::BlendMode;
    let starts: Vec<BlendMode> =
        (0..BlendMode::ALL.len()).filter(|i| crate::widgets::blend_separator_before(*i)).map(|i| BlendMode::ALL[i]).collect();
    assert_eq!(starts, [BlendMode::Darken, BlendMode::Lighten, BlendMode::Overlay, BlendMode::Difference, BlendMode::Hue]);
    assert!(starts.iter().all(|m| m.group() > 0) && !crate::widgets::blend_separator_before(BlendMode::ALL.len()));
}

#[test]
fn item_opacity_popup_sets_the_items_opacity() {
    let mut app = app_with_rect();
    let ctx = egui::Context::default();
    super::set_pstate(&ctx, "ap-open-1", true);
    let texts = frame_events(&ctx, &mut app, vec![], appearance::show);
    // The stroke's Opacity sub-row (listed first; the object's own Opacity row is last).
    let link = texts.iter().find(|(t, _)| t == "Opacity:").expect("item opacity row").1;
    click(&ctx, &mut app, link.center(), appearance::show);
    let texts = frame_events(&ctx, &mut app, vec![], appearance::show);
    assert!(texts.iter().any(|(t, _)| t == "Normal"), "the popup shows the blend mode: {texts:?}");
    // Type 50 into the popup's opacity field.
    let field = text_rect(&texts, "100%");
    click(&ctx, &mut app, field.center(), appearance::show);
    let key = |key, modifiers| egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers };
    frame_events(&ctx, &mut app, vec![key(egui::Key::A, egui::Modifiers::COMMAND), egui::Event::Text("50".into())], appearance::show);
    frame_events(&ctx, &mut app, vec![key(egui::Key::Enter, Default::default())], appearance::show);
    let n = first_selected(&app).unwrap();
    assert_eq!(n.appearance.items[1].opacity(), 0.5);
    assert_eq!((n.appearance.items[0].opacity(), n.opacity), (1.0, 1.0));
}

#[test]
fn opacity_popup_blend_dropdown_sets_the_blend_mode() {
    let mut app = app_with_rect();
    let ctx = egui::Context::default();
    let texts = frame_events(&ctx, &mut app, vec![], appearance::show);
    // The object's own Opacity row (the last one).
    let link = texts.iter().rev().find(|(t, _)| t == "Opacity:").expect("object opacity row").1;
    click(&ctx, &mut app, link.center(), appearance::show);
    let texts = frame_events(&ctx, &mut app, vec![], appearance::show);
    // Opening the dropdown inside the popup keeps the popup open: the list shows.
    click(&ctx, &mut app, text_rect(&texts, "Normal").center(), appearance::show);
    let texts = frame_events(&ctx, &mut app, vec![], appearance::show);
    click(&ctx, &mut app, text_rect(&texts, "Multiply").center(), appearance::show);
    assert_eq!(first_selected(&app).unwrap().blend, vectorcraft_color::BlendMode::Multiply);
    // The popup is still open after choosing.
    let texts = frame_events(&ctx, &mut app, vec![], appearance::show);
    assert!(texts.iter().any(|(t, _)| t == "Multiply"), "{texts:?}");
    // A frame without the panel closes it, as egui closes the popups it remembers.
    frame_events(&ctx, &mut app, vec![], |_, _| {});
    let texts = frame_events(&ctx, &mut app, vec![], appearance::show);
    assert!(!texts.iter().any(|(t, _)| t == "Multiply"), "{texts:?}");
}

#[test]
fn object_row_names_the_selection_like_the_control_bar() {
    let mut app = app_with_rect();
    let a = first_selected(&app).unwrap().id.0;
    let ctx = egui::Context::default();
    assert!(frame_in(&ctx, &mut app, appearance::show).contains(&"Rectangle".to_string()));
    let b = run(&mut app, "shape.rectangle", json!({"x": 120, "y": 10, "width": 50, "height": 50}))["id"].clone();
    // Objects with one appearance list it under "Mixed Objects".
    run(&mut app, "select.set", json!({"ids": [a, b]}));
    let texts = frame_in(&ctx, &mut app, appearance::show);
    assert!(texts.contains(&"Mixed Objects".to_string()) && texts.contains(&"Stroke:".to_string()), "{texts:?}");
    // Objects that differ list nothing.
    run(&mut app, "select.set", json!({"ids": [b]}));
    run(&mut app, "transparency.set", json!({"opacity": 50}));
    run(&mut app, "select.set", json!({"ids": [a, b]}));
    let texts = frame_in(&ctx, &mut app, appearance::show);
    assert!(texts.contains(&"Mixed Appearances".to_string()) && !texts.contains(&"Stroke:".to_string()), "{texts:?}");
    // Type objects are "Type" in both places.
    let t = run(&mut app, "text.create", json!({"x": 10, "y": 150, "text": "Hi"}))["id"].clone();
    run(&mut app, "select.set", json!({ "ids": [t] }));
    assert_eq!(appearance::object_label(&app), "Type");
    assert!(frame_in(&ctx, &mut app, appearance::show).contains(&"Type".to_string()));
    let fonts = egui::Context::default();
    crate::theme::install_fonts(&fonts);
    assert!(frame_in(&fonts, &mut app, crate::chrome::control_bar).contains(&"Type".to_string()));
}

/// The Appearance panel's button named `label`: the center of its rect (`ctx` needs accesskit on).
fn panel_button(ctx: &egui::Context, app: &mut VectorcraftApp, label: &str) -> egui::Pos2 {
    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| appearance::show(app, ui));
    out.textures_delta.clear();
    let nodes = &out.platform_output.accesskit_update.as_ref().expect("an accessibility tree").nodes;
    let (_, n) = nodes
        .iter()
        .find(|(_, n)| n.label() == Some(label) && n.role() == egui::accesskit::Role::Button)
        .unwrap_or_else(|| panic!("no button named {label:?}"));
    let b = n.bounds().expect("bounds");
    egui::pos2((b.x0 + b.x1) as f32 * 0.5, (b.y0 + b.y1) as f32 * 0.5)
}

/// Drag from `a` to `b` in the Appearance panel: press, move past the drag threshold, move, release.
fn drag_in_panel(ctx: &egui::Context, app: &mut VectorcraftApp, a: egui::Pos2, b: egui::Pos2) {
    let button = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
    for e in [
        vec![egui::Event::PointerMoved(a)],
        vec![button(a, true)],
        vec![egui::Event::PointerMoved(a + egui::vec2(0.0, 6.0))],
        vec![egui::Event::PointerMoved(b)],
        vec![button(b, false)],
        vec![],
    ] {
        frame_events(ctx, app, e, appearance::show);
    }
}

/// A point on the row whose label is the `nth` drawn `label`, clear of its eye, swatch and fields.
fn row_point(ctx: &egui::Context, app: &mut VectorcraftApp, label: &str, nth: usize) -> egui::Pos2 {
    let texts = frame_events(ctx, app, vec![], appearance::show);
    let r = texts.iter().filter(|(t, _)| t == label).nth(nth).unwrap_or_else(|| panic!("no {label:?} #{nth}: {texts:?}")).1;
    egui::pos2(r.left() + 400.0, r.center().y)
}

/// The selected object's fills/strokes as (is fill, stroke weight).
fn stack_items(app: &VectorcraftApp) -> Vec<(bool, Option<f32>)> {
    first_selected(app)
        .unwrap()
        .appearance
        .items
        .iter()
        .map(|it| match it {
            vectorcraft_doc::AppearanceItem::Stroke(s) => (false, Some(s.width as f32)),
            _ => (true, None),
        })
        .collect()
}

fn drop_ctx() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    ctx
}

/// Fill and stroke rows dropped on Delete Selected Item are removed, on Duplicate Selected Item
/// copied, as rows dropped on the Layers panel's buttons are.
#[test]
fn rows_dropped_on_delete_are_removed_and_on_duplicate_copied() {
    // [Fill, Stroke]: dropping the stroke on Delete leaves the fill.
    let mut app = app_with_rect();
    let ctx = drop_ctx();
    let delete = panel_button(&ctx, &mut app, "Delete Selected Item");
    let stroke = row_point(&ctx, &mut app, "Stroke:", 0);
    drag_in_panel(&ctx, &mut app, stroke, delete);
    assert_eq!(stack_items(&app), [(true, None)]);
    // Dropping the fill on Duplicate adds one copy of it.
    let mut app = app_with_rect();
    let ctx = drop_ctx();
    let duplicate = panel_button(&ctx, &mut app, "Duplicate Selected Item");
    let fill = row_point(&ctx, &mut app, "Fill:", 0);
    drag_in_panel(&ctx, &mut app, fill, duplicate);
    let items = stack_items(&app);
    assert_eq!(items.iter().filter(|(f, _)| *f).count(), 2, "{items:?}");
    assert_eq!(items.len(), 3, "{items:?}");
}

/// Dragging one of the selected rows onto Delete removes every selected row; dragging a row outside
/// the selection removes only that row.
#[test]
fn dropping_a_selected_row_acts_on_the_selection() {
    let setup = || {
        // [Fill, Stroke 1 pt, Stroke 6 pt]
        let mut app = app_with_rect();
        run(&mut app, "appearance.addStroke", json!({}));
        run(&mut app, "stroke.set", json!({"item": 2, "weight": 6}));
        let ctx = drop_ctx();
        frame_events(&ctx, &mut app, vec![], appearance::show);
        // Both strokes selected, the top one active.
        appearance::select_row(&mut app, &ctx, appearance::Sel::Item(2));
        let owner = first_selected(&app).map(|n| n.id);
        super::set_pstate(&ctx, "ap-multi", (owner, vec![1usize, 2]));
        (app, ctx)
    };
    // The top stroke row (drawn first) is in the selection: both strokes go.
    let (mut app, ctx) = setup();
    let delete = panel_button(&ctx, &mut app, "Delete Selected Item");
    let top = row_point(&ctx, &mut app, "Stroke:", 0);
    drag_in_panel(&ctx, &mut app, top, delete);
    assert_eq!(stack_items(&app), [(true, None)]);
    // The fill is not: only it goes.
    let (mut app, ctx) = setup();
    let delete = panel_button(&ctx, &mut app, "Delete Selected Item");
    let fill = row_point(&ctx, &mut app, "Fill:", 0);
    drag_in_panel(&ctx, &mut app, fill, delete);
    assert_eq!(stack_items(&app), [(false, Some(1.0)), (false, Some(6.0))]);
    // Onto Duplicate, the selected strokes are both copied.
    let (mut app, ctx) = setup();
    let duplicate = panel_button(&ctx, &mut app, "Duplicate Selected Item");
    let top = row_point(&ctx, &mut app, "Stroke:", 0);
    drag_in_panel(&ctx, &mut app, top, duplicate);
    let items = stack_items(&app);
    assert_eq!(items.len(), 5, "{items:?}");
    assert_eq!(items.iter().filter(|(_, w)| *w == Some(6.0)).count(), 2, "{items:?}");
}

/// An effect row dropped on Delete removes that effect, on Duplicate copies it; the stroke it
/// belongs to stays.
#[test]
fn effect_rows_dropped_on_delete_or_duplicate() {
    for (button, left) in [("Delete Selected Item", 0), ("Duplicate Selected Item", 2)] {
        let mut app = app_with_rect();
        run(&mut app, "effect.apply", json!({"effect": "distort.roughen", "item": 1}));
        let ctx = drop_ctx();
        super::set_pstate(&ctx, "ap-open-1", true);
        let at = panel_button(&ctx, &mut app, button);
        let fx = row_point(&ctx, &mut app, "Roughen", 0);
        drag_in_panel(&ctx, &mut app, fx, at);
        assert_eq!(current_stroke_effects(&app).len(), left, "{button}");
        assert_eq!(stack_items(&app).len(), 2, "{button}: the fill and stroke stay");
    }
}

/// The Contents row can be neither copied nor removed: dropped on either button it changes nothing.
/// A row released outside the panel changes nothing either, and the next drop still works.
#[test]
fn contents_rows_and_drops_elsewhere_change_nothing() {
    let mut app = VectorcraftApp::new(Session::new(), Default::default());
    run(&mut app, "file.new", json!({"width": 100, "height": 100}));
    let a = run(&mut app, "shape.rectangle", json!({"x": 0, "y": 0, "width": 20, "height": 20}))["id"].clone();
    let b = run(&mut app, "shape.rectangle", json!({"x": 30, "y": 0, "width": 20, "height": 20}))["id"].clone();
    run(&mut app, "select.set", json!({"ids": [a, b]}));
    run(&mut app, "object.group", json!({}));
    run(&mut app, "appearance.addFill", json!({}));
    let ctx = drop_ctx();
    let before = first_selected(&app).unwrap().appearance.clone();
    for button in ["Delete Selected Item", "Duplicate Selected Item"] {
        let at = panel_button(&ctx, &mut app, button);
        let contents = row_point(&ctx, &mut app, "Contents", 0);
        drag_in_panel(&ctx, &mut app, contents, at);
        assert_eq!(first_selected(&app).unwrap().appearance, before, "{button}");
    }
    // Released far below the panel: nothing happens.
    let fill = row_point(&ctx, &mut app, "Fill:", 0);
    drag_in_panel(&ctx, &mut app, fill, egui::pos2(fill.x, 9000.0));
    assert_eq!(first_selected(&app).unwrap().appearance, before);
    let delete = panel_button(&ctx, &mut app, "Delete Selected Item");
    let fill = row_point(&ctx, &mut app, "Fill:", 0);
    drag_in_panel(&ctx, &mut app, fill, delete);
    assert!(first_selected(&app).unwrap().appearance.items.is_empty());
}
