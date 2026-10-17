//! Barcode panel: the symbology, the data and the sizing of a barcode, and the button that draws
//! one.
//!
//! Every field here ends in `barcode.create` or `barcode.set`, so the panel and an agent driving
//! the same document over MCP take exactly the same path. A barcode object selected on the canvas
//! is what the panel edits — the fields show its settings and the button rewrites it — and with
//! nothing barcode-shaped selected the fields are a draft and the button draws a new one.

use egui::Ui;
use serde_json::json;
use vectorcraft_barcode::{BarcodeSpec, Ecc, Symbology};
use vectorcraft_doc::NodeId;

use crate::VectorcraftApp;
use crate::state::BarcodeBatch;
use crate::widgets;

/// The panel id, as the dock, the Window menu and `window.panel` name it.
pub const ID: &str = "barcode";

/// The symbology the panel starts on: the retail article number, which is what most packaging
/// needs.
const DEFAULT_SYMBOLOGY: Symbology = Symbology::Ean13;

/// A fresh draft: OCR-B for the human readable line when the machine has it, since that is the
/// face GS1 §5.2.4 recommends for it, and the document's own font otherwise.
fn draft_font(spec: &mut BarcodeSpec) {
    if spec.font.is_none() {
        spec.font = standard_font();
    }
}

/// The face the digits are set in when the panel does not name one: the machine's OCR-B, which is
/// the one the barcode standards name for the human readable line, or `None` for the document's own
/// font when the machine has no OCR-B.
pub(crate) fn standard_font() -> Option<String> {
    vectorcraft_text::FontDb::global().ocr_b()
}

/// The barcode object the selection names, or the one the selection is inside.
fn selected(app: &VectorcraftApp) -> Option<(NodeId, BarcodeSpec)> {
    let st = app.session.active()?;
    for id in st.selection.objects.iter().copied() {
        let mut cur = Some(id);
        while let Some(c) = cur {
            if let Some(spec) = st.doc.node(c).and_then(|n| n.extras().barcode.as_deref()) {
                return Some((c, spec.clone()));
            }
            cur = st.doc.parent_of(c);
        }
    }
    None
}

/// Whether this frame's edits are drawn on the selected object at once rather than waiting for
/// [Update].
///
/// Everything but the value itself is: a type, a size, a checkbox drawn only when the button is
/// pressed reads as a broken control. A value being typed is not — a half-typed number would redraw
/// the object on every keystroke and leave the canvas carrying the last one that happened to encode
/// — so it waits, except when the type changed and brought its own example value along with it.
pub(super) fn applies_at_once(spec: &BarcodeSpec, before: &BarcodeSpec) -> bool {
    spec.symbology != before.symbology || (spec != before && spec.data == before.data)
}

/// A draft for `symbology`: the example data that type is usually shown with, and the standard
/// font for its digits.
fn draft(symbology: Symbology) -> BarcodeSpec {
    let mut spec = BarcodeSpec { data: symbology.sample_data().to_string(), symbology, ..BarcodeSpec::default() };
    draft_font(&mut spec);
    spec
}

pub fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let target = selected(app);
    let mut spec = match &target {
        Some((_, spec)) => spec.clone(),
        None => app.ui.barcode.clone(),
    };
    // What the object holds before this frame's controls are drawn: the rows edit `spec`, and the
    // difference is what the selected object is redrawn from below.
    let before = spec.clone();

    let width = ui.available_width();
    symbology_row(ui, &mut spec, width);
    data_row(app, ui, &mut spec, width);
    match vectorcraft_barcode::encode(&spec) {
        Ok(code) => {
            let (w, h) = code.size_mm();
            let note = format!("{w:.2} × {h:.2} mm");
            widgets::dim_label(ui, &note);
        }
        Err(e) => {
            ui.colored_label(crate::theme::Tokens::get(ui.ctx()).error, format!("{e}"));
        }
    }
    ui.separator();
    size_rows(ui, &mut spec, width);
    ui.separator();
    if spec.symbology.is_matrix() {
        matrix_rows(ui, &mut spec, width);
        ui.separator();
    }
    hri_rows(ui, &mut spec, width);

    ui.separator();
    ui.horizontal(|ui| {
        let label = if target.is_some() { tl!("Update") } else { tl!("Create") };
        if ui.button(label).clicked() {
            let params = serde_json::to_value(&spec).unwrap_or(json!({}));
            match &target {
                Some((id, _)) => {
                    let mut p = params;
                    p["id"] = json!(id.0);
                    app.run("barcode.set", p).ok();
                }
                None => {
                    app.run("barcode.create", params).ok();
                }
            }
        }
        if target.is_some() && ui.button(tl!("New")).clicked() {
            app.ui.barcode = draft(spec.symbology);
        }
        if ui.button(tl!("Reset")).clicked() {
            app.ui.barcode = draft(DEFAULT_SYMBOLOGY);
        }
    });

    // Folded to start with, and left where it was left: most visits draw one code, and the section
    // is tall. It stays wherever the selection is, since a batch makes new codes rather than editing
    // the selected one.
    ui.add_space(4.0);
    let open = app.ui.barcode_batch_open;
    if widgets::section_toggle(ui, "Batch", open) {
        app.ui.barcode_batch_open = !open;
    }
    if open {
        batch_rows(app, ui, &spec, width);
    }

    // The selected object follows the panel as it is edited. A control that looks live but leaves
    // the art alone reads as broken, and [Update] then has nothing left to do. Everything but the
    // value itself is drawn at once — one edit for the undo stack, the same as the draft is — while
    // a value being typed waits for [Update] or the field's Enter, so a half-typed number never
    // redraws the object.
    if let Some((id, _)) = &target
        && applies_at_once(&spec, &before)
    {
        let mut p = serde_json::to_value(&spec).unwrap_or(json!({}));
        p["id"] = json!(id.0);
        app.run("barcode.set", p).ok();
    }

    if target.is_none() {
        // The draft the panel keeps is what was typed, once every row has had its say.
        app.ui.barcode = spec;
    }
}

fn symbology_row(ui: &mut Ui, spec: &mut BarcodeSpec, width: f32) {
    // The rows name the standard each symbology belongs to, the way the barcode tools a packaging
    // shop already uses do: "EAN-13" alone does not say whether it is the Chinese, Japanese or
    // American number.
    let labels: Vec<String> = Symbology::all().iter().map(|s| format!("{} · {}", s.label(), s.origin())).collect();
    let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
    let shown = format!("{} · {}", spec.symbology.label(), spec.symbology.origin());
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Type:"));
        let rest = width - 46.0;
        let before = spec.symbology;
        if let Some(i) = widgets::dropdown_names(ui, "barcode-type", &shown, &labels, rest)
            && let Some(s) = Symbology::all().get(i)
        {
            spec.symbology = *s;
        }
        // Changing the type brings that type's own example data along, so the panel starts on a
        // value the new symbology can carry instead of on the one the last type was working with.
        if spec.symbology != before {
            spec.data = spec.symbology.sample_data().to_string();
        }
    });
    widgets::dim_label(ui, spec.symbology.standard());
}

fn data_row(app: &mut VectorcraftApp, ui: &mut Ui, spec: &mut BarcodeSpec, width: f32) {
    let mut asked = false;
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Data:"));
        let field = (width - 46.0 - 56.0).max(60.0);
        if let Some(v) = widgets::text_field(ui, "barcode-data", Some(spec.data.as_str()), field, 1) {
            spec.data = v;
            // A new value is a new question: the last answer no longer applies to it.
            app.ui.barcode_note.clear();
        }
        asked = ui.button(tl!("Check")).clicked();
    });
    widgets::dim_label(ui, spec.symbology.hint());
    if asked {
        let params = serde_json::to_value(&*spec).unwrap_or(json!({}));
        app.ui.barcode_note = match app.run("barcode.check", params) {
            Ok(out) if out["ok"] == json!(true) => format!("{} {}", tl!("Valid:"), out["data"].as_str().unwrap_or("")),
            Ok(out) => out["message"].as_str().unwrap_or("").to_string(),
            Err(e) => e,
        };
    }
    if !app.ui.barcode_note.is_empty() {
        let t = crate::theme::Tokens::get(ui.ctx());
        let ok = app.ui.barcode_note.starts_with(tl!("Valid:"));
        ui.colored_label(if ok { t.success } else { t.error }, app.ui.barcode_note.clone());
    }
}

fn size_rows(ui: &mut Ui, spec: &mut BarcodeSpec, width: f32) {
    let field = (width - 96.0).clamp(50.0, 120.0);
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Magnification:"));
        if let Some(v) = widgets::mixed_field(ui, "barcode-magnification", Some(spec.magnification * 100.0), "%", 0, field) {
            spec.magnification = (v / 100.0).clamp(0.25, 10.0);
        }
    });
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Module:"));
        let shown = spec.module_mm.unwrap_or_else(|| spec.symbology.nominal_module_mm());
        if let Some(v) = widgets::mixed_field(ui, "barcode-module", Some(shown), " mm", 3, field) {
            spec.module_mm = (v.is_finite() && v > 0.0).then_some(v);
        }
    });
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Height:"));
        let shown = spec.height_mm.unwrap_or_else(|| spec.symbology.nominal_height_mm());
        if let Some(v) = widgets::mixed_field(ui, "barcode-height", Some(shown), " mm", 2, field) {
            spec.height_mm = (v.is_finite() && v > 0.0).then_some(v);
        }
    });
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Quiet Zone:"));
        if let Some(v) = widgets::mixed_field(ui, "barcode-quiet", Some(spec.quiet_extra), "", 0, field) {
            spec.quiet_extra = v.clamp(0.0, 100.0);
        }
    });
    if matches!(spec.symbology, Symbology::Code39 | Symbology::Itf14 | Symbology::Itf25) {
        ui.horizontal(|ui| {
            widgets::dim_label(ui, tl!("Width Ratio:"));
            let nominal = spec.ratio.unwrap_or(if spec.symbology == Symbology::Code39 { 3.0 } else { 2.5 });
            if let Some(v) = widgets::mixed_field(ui, "barcode-ratio", Some(nominal), "", 2, field) {
                spec.ratio = Some(v.clamp(2.0, 3.0));
            }
        });
    }
}

fn matrix_rows(ui: &mut Ui, spec: &mut BarcodeSpec, width: f32) {
    let field = (width - 96.0).clamp(50.0, 120.0);
    let levels: Vec<&str> = Ecc::all().iter().map(|e| e.label()).collect();
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Error Correction:"));
        if let Some(i) = widgets::dropdown_names(ui, "barcode-ecc", spec.ecc.label(), &levels, field)
            && let Some(e) = Ecc::all().get(i)
        {
            spec.ecc = *e;
        }
    });
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Version:"));
        let shown = spec.version.map_or_else(|| tl!("Automatic").to_string(), |v| v.to_string());
        let options: Vec<String> = std::iter::once(tl!("Automatic").to_string()).chain((1..=40).map(|v| v.to_string())).collect();
        let refs: Vec<&str> = options.iter().map(String::as_str).collect();
        if let Some(i) = widgets::dropdown_names(ui, "barcode-version", &shown, &refs, field) {
            spec.version = (i > 0).then_some(i as u8);
        }
    });
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Mask:"));
        let shown = spec.mask.map_or_else(|| tl!("Automatic").to_string(), |v| v.to_string());
        let options: Vec<String> = std::iter::once(tl!("Automatic").to_string()).chain((0..8).map(|v| v.to_string())).collect();
        let refs: Vec<&str> = options.iter().map(String::as_str).collect();
        if let Some(i) = widgets::dropdown_names(ui, "barcode-mask", &shown, &refs, field) {
            spec.mask = (i > 0).then_some((i - 1) as u8);
        }
    });
}

fn hri_rows(ui: &mut Ui, spec: &mut BarcodeSpec, width: f32) {
    if spec.symbology.is_matrix() {
        return;
    }
    if widgets::check(ui, tl!("Human Readable Line"), spec.text, true) {
        spec.text = !spec.text;
    }
    if widgets::check(ui, tl!("Outline the digits"), spec.outline_text, spec.text) {
        spec.outline_text = !spec.outline_text;
    }
    if widgets::check(ui, tl!("White background box"), spec.background, true) {
        spec.background = !spec.background;
    }
    let families = vectorcraft_text::FontDb::global().families();
    // The first entry names the face the engine will actually set the digits in when the spec does
    // not, rather than a "Document Default" that a machine with OCR-B would silently ignore.
    let standard = standard_font();
    let first = standard.clone().unwrap_or_else(|| tl!("Document Default").to_string());
    let shown = spec.font.clone().unwrap_or_else(|| first.clone());
    let options: Vec<String> = std::iter::once(first.clone()).chain(families.iter().cloned()).collect();
    let refs: Vec<&str> = options.iter().map(String::as_str).collect();
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Font:"));
        if let Some(i) = widgets::dropdown_names(ui, "barcode-font", &shown, &refs, width - 46.0) {
            spec.font = (i > 0).then(|| options.get(i).cloned().unwrap_or_default());
        }
    });
    if families.is_empty() {
        widgets::dim_label(ui, tl!("No fonts are installed."));
    } else if standard.is_none() {
        widgets::dim_label(ui, tl!("Install OCR-B for the standard digits."));
    }
}

/// Many codes at once: a list pasted in from a spreadsheet, drawn as a grid across the artboard so
/// that a run of cartons is one gesture rather than one per carton.
///
/// Its own section: a batch makes new objects rather than editing the one on the canvas, so it does
/// not belong beside the button that rewrites the selection — and it stays where it is whatever is
/// selected, since a batch leaves the codes it drew selected.
fn batch_rows(app: &mut VectorcraftApp, ui: &mut Ui, spec: &BarcodeSpec, width: f32) {
    let mut batch = app.ui.barcode_batch.clone();
    if let Some(v) = widgets::text_field(ui, "barcode-batch-data", Some(batch.data.as_str()), width, 5) {
        batch.data = v;
    }
    let field = (width - 132.0).clamp(40.0, 90.0);
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Max per Row:"));
        if let Some(v) = widgets::mixed_field(ui, "barcode-batch-columns", Some(f64::from(batch.columns)), "", 0, field) {
            batch.columns = v.clamp(1.0, 200.0) as u32;
        }
        widgets::dim_label(ui, tl!("Gap:"));
        if let Some(v) = widgets::mixed_field(ui, "barcode-batch-gap", Some(batch.gap), " mm", 1, field) {
            batch.gap = v.clamp(0.0, 1000.0);
        }
    });
    let count = batch.data.lines().filter(|l| !l.trim().is_empty()).count();
    widgets::dim_label(ui, &format!("{} {count}", tl!("Codes:")));
    let mut run = false;
    ui.horizontal(|ui| {
        run = ui.add_enabled(count > 0, egui::Button::new(tl!("Create Batch"))).clicked();
        if ui.button(tl!("Clear")).clicked() {
            batch = BarcodeBatch::default();
        }
    });
    if run {
        let mut p = serde_json::to_value(spec).unwrap_or(json!({}));
        p["data"] = json!(batch.data);
        p["columns"] = json!(batch.columns);
        p["gapX"] = json!(batch.gap);
        p["gapY"] = json!(batch.gap);
        app.ui.barcode_batch_note = match app.run("barcode.createBatch", p) {
            Ok(out) => {
                let drawn = out["count"].as_u64().unwrap_or(0);
                let skipped = out["skipped"].as_array().map_or(0, Vec::len);
                if skipped == 0 {
                    format!("{} {drawn}", tl!("Drawn:"))
                } else {
                    let first = out["skipped"].as_array().and_then(|a| a.first()).and_then(|v| v["message"].as_str()).unwrap_or("");
                    format!("{} {drawn}, {skipped} — {first}", tl!("Drawn:"))
                }
            }
            Err(e) => e,
        };
    }
    if !app.ui.barcode_batch_note.is_empty() {
        let t = crate::theme::Tokens::get(ui.ctx());
        let ok = app.ui.barcode_batch_note.starts_with(tl!("Drawn:"));
        ui.colored_label(if ok { t.success } else { t.error }, app.ui.barcode_batch_note.clone());
    }
    app.ui.barcode_batch = batch;
}

/// Items of the panel's (≡) menu.
pub fn menu(app: &mut VectorcraftApp, ui: &mut Ui) -> bool {
    let mut any = false;
    if ui.button(tl!("Reset")).clicked() {
        app.ui.barcode = draft(DEFAULT_SYMBOLOGY);
        any = true;
    }
    any
}
