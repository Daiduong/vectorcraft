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

/// OCR-B, when the machine has it installed.
pub(crate) fn standard_font() -> Option<String> {
    vectorcraft_text::FontDb::global().has_family("OCR-B").then(|| "OCR-B".to_string())
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

    let width = ui.available_width();
    symbology_row(ui, &mut spec, width);
    data_row(ui, &mut spec, width);
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

    if target.is_none() {
        app.ui.barcode = spec;
    }
}

fn symbology_row(ui: &mut Ui, spec: &mut BarcodeSpec, width: f32) {
    let labels: Vec<&str> = Symbology::all().iter().map(|s| s.label()).collect();
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Type:"));
        let rest = width - 46.0;
        let before = spec.symbology;
        if let Some(i) = widgets::dropdown_names(ui, "barcode-type", spec.symbology.label(), &labels, rest)
            && let Some(s) = Symbology::all().get(i)
        {
            spec.symbology = *s;
        }
        // Changing the type brings that type's own example data along, so the panel does not sit
        // on a number the new symbology cannot carry.
        if spec.symbology != before && spec.data == before.sample_data() {
            spec.data = spec.symbology.sample_data().to_string();
        }
    });
}

fn data_row(ui: &mut Ui, spec: &mut BarcodeSpec, width: f32) {
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Data:"));
        if let Some(v) = widgets::text_field(ui, "barcode-data", Some(spec.data.as_str()), width - 46.0, 1) {
            spec.data = v;
        }
    });
    widgets::dim_label(ui, spec.symbology.hint());
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
    let families = vectorcraft_text::FontDb::global().families();
    let shown = spec.font.clone().unwrap_or_else(|| tl!("Document Default").to_string());
    let mut options: Vec<String> = std::iter::once(tl!("Document Default").to_string()).collect();
    options.extend(families.iter().cloned());
    let refs: Vec<&str> = options.iter().map(String::as_str).collect();
    ui.horizontal(|ui| {
        widgets::dim_label(ui, tl!("Font:"));
        if let Some(i) = widgets::dropdown_names(ui, "barcode-font", &shown, &refs, width - 46.0) {
            spec.font = (i > 0).then(|| options.get(i).cloned().unwrap_or_default());
        }
    });
    if !families.is_empty() && standard_font().is_none() {
        widgets::dim_label(ui, tl!("Install OCR-B for the standard digits."));
    }
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
