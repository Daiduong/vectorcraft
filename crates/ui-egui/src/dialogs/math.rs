//! Type › Insert Equation… and Edit Equation (a double-click on an equation with the Selection
//! tool, or `math.edit` with no parameters): the LaTeX source of a formula, its size in points and
//! whether it is set in display style. OK runs `math.create` (a new equation on the active
//! artboard) or `math.edit` (the equation again, in place), as one undo step; the Preview box
//! shows the result on the canvas while the fields change and Cancel rolls it back.
//!
//! The source is checked as it is typed: a formula the parser rejects shows its message under the
//! box, and the canvas keeps the last formula that did parse. Errors of the command itself (no
//! math font, an empty formula) show in the status bar and leave the dialog open.
//!
//! Fields: `source` (LaTeX math), `size` (pt), `display`, `preview`, and the private `__id` (the
//! equation being edited; absent when inserting a new one), `__artboard` (where a new equation
//! goes) and `__good` (the last source that parsed, which the preview runs).

use serde_json::{Value, json};
use vectorcraft_doc::math::DEFAULT_MATH_SIZE;

use super::{DialogSpec, form};
use crate::state::Dialog;
use crate::theme::Tokens;
use crate::{VectorcraftApp, widgets};

/// The dialog kind of Insert Equation / Edit Equation.
pub const KIND: &str = "math";

const CREATE: &str = "math.create";
const EDIT: &str = "math.edit";

/// The base font sizes `math.create` and `math.edit` accept, in points.
const SIZE_RANGE: std::ops::RangeInclusive<f64> = 0.5..=2000.0;

/// The width of the source box.
const SOURCE_W: f32 = 360.0;

pub(super) const SPEC: DialogSpec = DialogSpec {
    heading: |d| if editing(d) { tl!("Edit Equation").into() } else { tl!("Insert Equation").into() },
    body,
    confirm,
    preview: true,
    min_width: 400.0,
    ..DialogSpec::FORM
};

/// Open the dialog for `{id?}`: Edit Equation on equation `id`, or Insert Equation without one
/// (what a tool's double-click and `ui.dialog` callers send).
pub fn open(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    match p.get("id").filter(|v| !v.is_null()) {
        None => open_create(app),
        Some(v) => match v.as_u64() {
            Some(id) => open_edit(app, Some(id)),
            None => Err(format!("object ids are non-negative integers, not {v}")),
        },
    }
}

/// Open Insert Equation: an empty formula in display style at the default size.
pub fn open_create(app: &mut VectorcraftApp) -> Result<Value, String> {
    let mut fields = json!({"source": "", "size": DEFAULT_MATH_SIZE, "display": true, "preview": true});
    if let Some(v) = app.view() {
        fields["__artboard"] = json!(v.artboard);
    }
    app.ui.dialog = Some(Dialog::new(KIND, fields));
    Ok(Value::Null)
}

/// Open Edit Equation on equation `id` (default: the selected one, or the one containing the
/// selected object), with its current source, size and style: what `math.edit` answers with no
/// changes asked.
pub fn open_edit(app: &mut VectorcraftApp, id: Option<u64>) -> Result<Value, String> {
    let q = app.session.execute(EDIT, &id.map_or_else(|| json!({}), |id| json!({ "id": id }))).map_err(|e| e.to_string())?;
    let fields = json!({"__id": q["id"], "source": q["source"], "size": q["size"], "display": q["display"], "preview": true});
    app.ui.dialog = Some(Dialog::new(KIND, fields));
    Ok(Value::Null)
}

/// Is the dialog editing an equation (rather than inserting one)?
fn editing(d: &Dialog) -> bool {
    d.fields.get("__id").is_some_and(|v| !v.is_null())
}

/// The command and its parameters from the fields.
fn command(d: &Dialog) -> (&'static str, Value) {
    let mut p = json!({
        "source": d.str("source"),
        "size": d.f64("size", DEFAULT_MATH_SIZE).clamp(*SIZE_RANGE.start(), *SIZE_RANGE.end()),
        "display": d.bool("display"),
    });
    if editing(d) {
        p["id"] = d.fields.get("__id").cloned().unwrap_or(Value::Null);
        (EDIT, p)
    } else {
        if let Some(a) = d.fields.get("__artboard") {
            p["artboard"] = a.clone();
        }
        (CREATE, p)
    }
}

/// The multi-line source box (monospace; Enter adds a line, it doesn't press OK).
fn source_box(ui: &mut egui::Ui, d: &mut Dialog) {
    let t = Tokens::get(ui.ctx());
    let mut s = d.str("source");
    let id = ui.id().with("math-source");
    widgets::take_dialog_focus(ui, id, &s);
    let r = egui::Frame::NONE
        .fill(t.input)
        .stroke(egui::Stroke::new(1.0, t.input_border))
        .corner_radius(egui::CornerRadius::same(3))
        .inner_margin(egui::Margin::symmetric(6, 3))
        .show(ui, |ui| {
            crate::theme::strong_selected_text(ui);
            ui.add(
                egui::TextEdit::multiline(&mut s)
                    .id(id)
                    .font(egui::TextStyle::Monospace)
                    .frame(egui::Frame::NONE)
                    .desired_width(SOURCE_W)
                    .desired_rows(4),
            )
        });
    if r.inner.changed() {
        d.fields.insert("source".into(), Value::String(s));
    }
    if r.inner.has_focus() {
        ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
    }
}

fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new(tl!("LaTeX:")).color(t.text_dim));
    ui.add_space(4.0);
    source_box(ui, d);
    // The parser is cheap and needs no font: a malformed formula is reported as it is typed.
    let source = d.str("source");
    let problem = if source.trim().is_empty() { None } else { vectorcraft_math::parse(&source).err() };
    ui.add_space(4.0);
    if let Some(e) = &problem {
        ui.add(egui::Label::new(egui::RichText::new(e.to_string()).color(t.error).size(12.0)).wrap());
        ui.add_space(4.0);
    }
    ui.add_space(6.0);
    widgets::label_row(ui, tl!("Size:"), 56.0, |ui| {
        let size = d.f64("size", DEFAULT_MATH_SIZE);
        if let Some(n) = widgets::range_field(ui, "math-size", size, SIZE_RANGE, " pt", 1, 80.0) {
            d.fields.insert("size".into(), json!(n));
        }
    });
    ui.add_space(4.0);
    form::check(ui, d, "display", tl!("Display style"));
    // The preview runs the last formula that parsed, so typing a formula never flashes errors on
    // the canvas; before the first one there is nothing to preview. Size and style follow the
    // fields as they change.
    if !source.trim().is_empty() && problem.is_none() {
        d.fields.insert("__good".into(), Value::String(source));
    }
    match d.fields.get("__good").and_then(Value::as_str).map(str::to_owned) {
        Some(good) => {
            let (cmd, mut p) = command(d);
            p["source"] = Value::String(good);
            form::preview(app, ui, d, if editing(d) { "Edit Equation" } else { "Insert Equation" }, cmd, p);
        }
        None => {
            ui.add_space(8.0);
            widgets::check(ui, tl!("Preview"), d.bool("preview"), false);
        }
    }
    false
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let (cmd, p) = command(d);
    form::commit_preview(app, cmd, p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_engine::Session;

    fn frame(app: &mut VectorcraftApp) {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| super::super::show(app, ui.ctx()));
        out.textures_delta.clear();
    }

    fn app() -> VectorcraftApp {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("file.new", json!({"width": 400, "height": 300})).unwrap();
        app
    }

    fn equations(app: &VectorcraftApp) -> usize {
        let mut n = 0;
        app.session.doc().unwrap().doc.walk(|node| n += usize::from(node.extras().math.is_some()));
        n
    }

    /// Whether the font database has a font to set formulas in (a system font; the repository ships none).
    fn has_font() -> bool {
        vectorcraft_math::find_math_face(vectorcraft_text::FontDb::global()).is_some()
    }

    #[test]
    fn the_menu_opens_insert_equation_with_defaults() {
        let mut app = app();
        crate::menus::invoke(&mut app, CREATE, json!({}));
        let d = app.ui.dialog.as_ref().expect("the dialog opens");
        assert_eq!(d.kind, KIND);
        assert!(!editing(d));
        assert_eq!((d.str("source"), d.f64("size", 0.0), d.bool("display")), (String::new(), DEFAULT_MATH_SIZE, true));
        frame(&mut app);
        assert!(app.ui.dialog.is_some(), "a frame keeps it open");
        assert_eq!(equations(&app), 0, "nothing is previewed before a formula parses");
    }

    #[test]
    fn an_empty_formula_is_an_error_and_the_dialog_stays_open() {
        let mut app = app();
        crate::menus::invoke(&mut app, CREATE, json!({}));
        frame(&mut app);
        assert!(crate::dialogs::confirm(&mut app).is_err());
        assert!(app.ui.dialog.is_some());
        assert_eq!(equations(&app), 0);
    }

    #[test]
    fn a_malformed_formula_does_not_reach_the_canvas() {
        let mut app = app();
        crate::menus::invoke(&mut app, CREATE, json!({}));
        app.ui.dialog.as_mut().unwrap().fields.insert("source".into(), json!(r"\frac{1"));
        frame(&mut app);
        assert!(!app.session.in_interaction(), "no preview ran");
        assert!(app.ui.dialog.as_ref().unwrap().fields.get("__good").is_none());
        assert!(crate::dialogs::confirm(&mut app).is_err());
        assert!(app.ui.dialog.is_some());
        crate::dialogs::cancel(&mut app);
        assert!(app.ui.dialog.is_none());
        assert_eq!(equations(&app), 0);
    }

    #[test]
    fn ok_inserts_the_equation_as_one_undo_step() {
        let mut app = app();
        crate::menus::invoke(&mut app, CREATE, json!({}));
        let d = app.ui.dialog.as_mut().unwrap();
        d.fields.insert("source".into(), json!(r"y=\sqrt{x}+\dfrac{1}{\sin x}"));
        d.fields.insert("size".into(), json!(36));
        frame(&mut app);
        let depth = app.session.doc().unwrap().history.undo.len();
        match crate::dialogs::confirm(&mut app) {
            Ok(_) => {
                assert!(has_font());
                assert!(app.ui.dialog.is_none());
                assert_eq!(equations(&app), 1);
                assert_eq!(app.session.doc().unwrap().history.undo.len(), depth + 1);
            }
            Err(e) => {
                // No math font on this machine: the error reaches the user and the dialog stays.
                assert!(!has_font(), "{e}");
                assert_eq!(e, "No math font available");
                assert!(app.ui.dialog.is_some());
                assert_eq!(equations(&app), 0);
            }
        }
    }

    #[test]
    fn cancel_rolls_the_preview_back() {
        if !has_font() {
            eprintln!("skipped: no font with a MATH table");
            return;
        }
        let mut app = app();
        crate::menus::invoke(&mut app, CREATE, json!({}));
        app.ui.dialog.as_mut().unwrap().fields.insert("source".into(), json!("x^2"));
        frame(&mut app);
        assert_eq!(equations(&app), 1, "previewed");
        crate::dialogs::cancel(&mut app);
        assert_eq!(equations(&app), 0);
    }

    #[test]
    fn edit_opens_on_the_equation_and_ok_sets_it_again() {
        if !has_font() {
            eprintln!("skipped: no font with a MATH table");
            return;
        }
        let mut app = app();
        let id = app.run(CREATE, json!({"source": "a+b", "size": 20, "display": false})).unwrap()["id"].as_u64().unwrap();
        // The menu command with nothing asked opens the dialog on the selected equation.
        crate::menus::invoke(&mut app, EDIT, json!({}));
        let d = app.ui.dialog.as_ref().expect("the dialog opens");
        assert!(editing(d));
        assert_eq!((d.str("source"), d.f64("size", 0.0), d.bool("display")), ("a+b".to_string(), 20.0, false));
        // The same through a tool's request for the equation `id`.
        crate::dialogs::open_tool_dialog(&mut app, KIND, json!({ "id": id }));
        assert!(editing(app.ui.dialog.as_ref().unwrap()));
        let d = app.ui.dialog.as_mut().unwrap();
        d.fields.insert("source".into(), json!(r"\frac{a}{b}"));
        d.fields.insert("size".into(), json!(30));
        frame(&mut app);
        crate::dialogs::confirm(&mut app).unwrap();
        assert!(app.ui.dialog.is_none());
        assert_eq!(equations(&app), 1);
        let spec = app.session.doc().unwrap().doc.node(vectorcraft_doc::NodeId(id)).and_then(|n| n.extras().math.as_deref().cloned()).unwrap();
        assert_eq!((spec.source.as_str(), spec.size), (r"\frac{a}{b}", 30.0));
    }

    #[test]
    fn editing_with_no_equation_selected_says_so() {
        let mut app = app();
        crate::menus::invoke(&mut app, EDIT, json!({}));
        assert!(app.ui.dialog.is_none());
        assert!(open_edit(&mut app, None).is_err());
        assert!(open(&mut app, &json!({"id": "x"})).is_err());
    }
}
