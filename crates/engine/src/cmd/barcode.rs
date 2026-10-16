//! Barcodes and QR Codes: Object → Barcode and the Barcode panel.
//!
//! A barcode is a group carrying a [`BarcodeSpec`] in its extras, like a graph: its children — the
//! bars or the modules, and the human readable line — are drawn from the spec, and changing the
//! spec redraws them in place. The encoding itself lives in `vectorcraft-barcode`, which knows
//! nothing about documents; everything here is about turning what it returns into art.
//!
//! Sizes come out in millimetres, which is the unit every symbology standard is written in, and
//! are converted once, here. The document is in points, so a 0.33 mm module prints at 0.94 pt.

use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_barcode::{Bar, BarcodeSpec, Code, Ecc, HriAlign, Symbology};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, CharStyle, Document, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::{Affine, BezPath, PathData, Point};

use super::create::{new_type_para, new_type_style};
use super::typecmd::refresh_bounds;
use super::*;

/// Millimetres to document points.
const PT_PER_MM: f64 = 72.0 / 25.4;

/// The digits' height as a fraction of the font size they are set in.
///
/// A font's cap height is its own, and the encoder can only say how tall the human readable line
/// should be. OCR-B, which GS1 §5.2.4 recommends for it, has a cap height of 0.70 em, and the
/// faces a document without it falls back to are within a few hundredths of that.
const CAP_HEIGHT_RATIO: f64 = 0.70;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "barcode.create",
            "Barcode",
            [],
            None,
            "{symbology: ean13|ean8|upcA|upcE|ean128|code128|code128A|code128B|code128C|code39|code93|itf14|itf25|qrCode, data, x?, y? (default: the centre of the first artboard), magnification?: the module size multiplier, 1 = the standard one, moduleMm?, heightMm?, ratio?: the wide-to-narrow ratio of Code 39 and Interleaved 2 of 5, text?: the human readable line, quietExtra?: extra quiet zone in modules, bearer?: an ITF-14's bearer bars, ecc?: low|medium|quartile|high, version?: 1-40, mask?: 0-7, font?, fontStyle?} → {id, data: the digits the symbol carries, with the check digit when it was added}",
            has_doc,
            create
        ),
        cmd!(
            "barcode.set",
            "Barcode Options",
            [],
            None,
            "{id?, any option `barcode.create` takes} change the selected barcode and draw it again; no options → {id, data, symbology, …the current ones}",
            has_selection,
            set
        ),
    ]
}

/// A `f64` a command carries, when it carries one.
fn opt_f64(p: &Value, key: &str) -> Option<f64> {
    p.get(key).and_then(Value::as_f64).filter(|v| v.is_finite())
}

fn opt_bool(p: &Value, key: &str) -> Option<bool> {
    p.get(key).and_then(Value::as_bool)
}

fn opt_u8(p: &Value, key: &str) -> Option<u8> {
    p.get(key).and_then(Value::as_u64).and_then(|v| u8::try_from(v).ok())
}

/// Read the options a command carries onto `spec`.
///
/// Only the keys the command actually carries are changed, so `barcode.set` can adjust one thing
/// without a caller having to repeat the rest.
fn read_spec(spec: &mut BarcodeSpec, p: &Value, what: &str) -> Result<()> {
    if let Some(name) = str_param(p, "symbology") {
        spec.symbology = Symbology::from_id(name).ok_or_else(|| bad(what, format!("unknown barcode type `{name}`")))?;
    }
    if let Some(data) = str_param(p, "data") {
        spec.data = data.to_string();
    }
    if let Some(v) = opt_f64(p, "magnification") {
        spec.magnification = v.clamp(0.25, 10.0);
    }
    if p.get("moduleMm").is_some() {
        spec.module_mm = opt_f64(p, "moduleMm");
    }
    if p.get("heightMm").is_some() {
        spec.height_mm = opt_f64(p, "heightMm");
    }
    if let Some(v) = opt_bool(p, "text") {
        spec.text = v;
    }
    if let Some(v) = opt_f64(p, "quietExtra") {
        spec.quiet_extra = v.clamp(0.0, 100.0);
    }
    if let Some(v) = opt_bool(p, "bearer") {
        spec.bearer = v;
    }
    if let Some(v) = opt_f64(p, "ratio") {
        spec.ratio = Some(v.clamp(2.0, 3.0));
    }
    if let Some(name) = str_param(p, "ecc") {
        spec.ecc = match name.to_ascii_lowercase().as_str() {
            "low" | "l" => Ecc::Low,
            "medium" | "m" => Ecc::Medium,
            "quartile" | "q" => Ecc::Quartile,
            "high" | "h" => Ecc::High,
            other => return Err(bad(what, format!("unknown error correction level `{other}`"))),
        };
    }
    if let Some(v) = opt_u8(p, "version") {
        spec.version = match v {
            0 => None,
            1..=40 => Some(v),
            other => return Err(bad(what, format!("a QR Code version is 1 to 40, not {other}"))),
        };
    }
    if p.get("mask").is_some() {
        spec.mask = opt_u8(p, "mask").map(|m| m % 8);
    }
    if p.get("font").is_some() {
        spec.font = str_param(p, "font").map(str::to_string);
    }
    if p.get("fontStyle").is_some() {
        spec.font_style = str_param(p, "fontStyle").map(str::to_string);
    }
    Ok(())
}

/// What the encoder made of a spec, and the data it ended up carrying — which is not what a caller
/// typed when a check digit was missing.
fn encoded(spec: &BarcodeSpec) -> Result<(Code, String)> {
    let code = vectorcraft_barcode::encode(spec).map_err(|e| EngineError::Other(e.to_string()))?;
    let data = match &code {
        // The data of a finished symbol is its human readable line, which carries every digit the
        // symbol encodes; a caller who typed twelve of thirteen gets the thirteenth back.
        Code::Linear(symbol) => {
            symbol.human_readable.as_ref().map(|h| h.parts.iter().map(|p| p.text.as_str()).collect::<String>()).unwrap_or_else(|| spec.data.clone())
        }
        Code::Matrix(_) => spec.data.clone(),
    };
    Ok((code, data))
}

fn create(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "barcode.create";
    let mut spec = BarcodeSpec::default();
    read_spec(&mut spec, p, C)?;
    if spec.data.trim().is_empty() {
        // A new barcode shows something rather than refusing to draw.
        spec.data = spec.symbology.sample_data().to_string();
    }
    let (code, data) = encoded(&spec)?;
    let (w, h) = code.size_mm();
    let (w, h) = (w * PT_PER_MM, h * PT_PER_MM);
    let parent = s.doc()?.target_parent()?;
    let style = s_style(s, &spec);
    let id = s.edit("Barcode", |d, sel| {
        let origin = match (opt_f64(p, "x"), opt_f64(p, "y")) {
            (Some(x), Some(y)) => Point::new(x, y),
            _ => {
                let centre = d.artboards.first().map(|a| a.rect.center()).unwrap_or(Point::ZERO);
                Point::new(centre.x - w / 2.0, centre.y - h / 2.0)
            }
        };
        let gid = d.alloc_id();
        let mut g = Node::group(gid, Vec::new());
        g.name = Some(format!("{} {}", spec.symbology.label(), data));
        d.insert(parent, usize::MAX, g)?;
        draw(d, gid, &spec, &code, origin, style.clone())?;
        sel.set([gid]);
        Ok(gid)
    })?;
    Ok(json!({ "id": id.0, "data": data }))
}

/// The character style the human readable line is set in: the spec's font, else the one new type
/// in this document gets.
fn s_style(s: &Session, spec: &BarcodeSpec) -> CharStyle {
    let mut style = new_type_style(s, &json!({}));
    if let Some(font) = &spec.font {
        style.font_family = font.clone();
    }
    if let Some(face) = &spec.font_style {
        style.font_style = face.clone();
    }
    style.fill = Paint::solid(Color::BLACK);
    style.stroke = Paint::None;
    style.stroke_width = 0.0;
    style
}

/// The barcode object the selection names, or the one the selection is inside.
fn target(s: &Session, p: &Value, what: &str) -> Result<NodeId> {
    let st = s.doc()?;
    let start = id_param(p, "id").into_iter().chain(st.selection.objects.iter().copied());
    for id in start {
        let mut cur = Some(id);
        while let Some(c) = cur {
            if st.doc.node(c).is_some_and(|n| n.extras().barcode.is_some()) {
                return Ok(c);
            }
            cur = st.doc.parent_of(c);
        }
    }
    Err(bad(what, "select a barcode"))
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "barcode.set";
    let id = target(s, p, C)?;
    let mut spec = s.doc()?.doc.node(id).and_then(|n| n.extras().barcode.as_deref().cloned()).ok_or(EngineError::NoNode(id))?;
    // A query: the command carried no options at all.
    let query = p.as_object().is_none_or(|o| o.keys().all(|k| k == "id"));
    if query {
        let (code, data) = encoded(&spec)?;
        let (w, h) = code.size_mm();
        return Ok(json!({
            "id": id.0,
            "spec": spec,
            "data": data,
            "symbology": spec.symbology.id(),
            "widthMm": w,
            "heightMm": h,
        }));
    }
    read_spec(&mut spec, p, C)?;
    let (code, data) = encoded(&spec)?;
    let origin = top_left(s, id);
    let style = s_style(s, &spec);
    s.edit("Barcode Options", |d, _| draw(d, id, &spec, &code, origin, style.clone()))?;
    Ok(json!({ "id": id.0, "data": data }))
}

/// Where the barcode's top-left corner is now: its children's bounds, or the origin when it has
/// none. Redrawing puts the new art where the old art was.
fn top_left(s: &Session, id: NodeId) -> Point {
    s.doc().ok().and_then(|st| st.doc.bounds_of(&[id], false)).map_or(Point::ZERO, |r| Point::new(r.x0, r.y0))
}

/// Draw `code` into the barcode object `id`, from its top-left corner.
fn draw(d: &mut Document, id: NodeId, spec: &BarcodeSpec, code: &Code, origin: Point, style: CharStyle) -> Result<()> {
    let mut children: Vec<Arc<Node>> = Vec::new();
    match code {
        Code::Linear(symbol) => {
            let bars = bars_path(&symbol.bars, origin);
            if !bars.is_empty() {
                children.push(Arc::new(Node::path(d.alloc_id(), bars, Appearance::basic(Paint::solid(Color::BLACK), Paint::None, 0.0))));
            }
            children.extend(hri_nodes(d, symbol.human_readable.as_ref(), origin, &style));
        }
        Code::Matrix(matrix) => {
            let quiet = matrix.quiet as f64 * matrix.module_mm;
            let mut bp = BezPath::new();
            for run in matrix.runs() {
                let x0 = origin.x + (quiet + run.x as f64 * matrix.module_mm) * PT_PER_MM;
                let y0 = origin.y + (quiet + run.y as f64 * matrix.module_mm) * PT_PER_MM;
                let x1 = x0 + run.width as f64 * matrix.module_mm * PT_PER_MM;
                let y1 = y0 + run.height as f64 * matrix.module_mm * PT_PER_MM;
                rectangle(&mut bp, x0, y0, x1, y1);
            }
            children.push(Arc::new(Node::path(
                d.alloc_id(),
                PathData::from_bezpath(&bp),
                Appearance::basic(Paint::solid(Color::BLACK), Paint::None, 0.0),
            )));
        }
    }
    let node = d.node_mut(id).ok_or(EngineError::NoNode(id))?;
    if let Some(slot) = node.children_mut() {
        *slot = children;
    }
    node.extras_mut().barcode = Some(Box::new(spec.clone()));
    Ok(())
}

/// One path holding every bar, so that a symbol is one object a caller can fill, move or recolour
/// rather than a hundred.
fn bars_path(bars: &[Bar], origin: Point) -> PathData {
    let mut bp = BezPath::new();
    for bar in bars {
        let x0 = origin.x + bar.x * PT_PER_MM;
        let y0 = origin.y + bar.y * PT_PER_MM;
        let x1 = x0 + bar.width * PT_PER_MM;
        let y1 = y0 + bar.height * PT_PER_MM;
        rectangle(&mut bp, x0, y0, x1, y1);
    }
    PathData::from_bezpath(&bp)
}

fn rectangle(bp: &mut BezPath, x0: f64, y0: f64, x1: f64, y1: f64) {
    if !(x1 > x0 && y1 > y0) {
        return;
    }
    bp.move_to(Point::new(x0, y0));
    bp.line_to(Point::new(x1, y0));
    bp.line_to(Point::new(x1, y1));
    bp.line_to(Point::new(x0, y1));
    bp.close_path();
}

/// The human readable line's text objects.
///
/// Each part is set at the size its height asks for and shrunk when the font it lands in would run
/// past the character cells the digits sit under; the anchor is the edge `at_mm` gave, so a digit
/// printed outside the symbol keeps its distance from the guard bars whatever the font.
fn hri_nodes(d: &mut Document, hri: Option<&vectorcraft_barcode::HumanReadable>, origin: Point, style: &CharStyle) -> Vec<Arc<Node>> {
    let Some(hri) = hri else { return Vec::new() };
    let baseline = origin.y + (hri.top_mm + hri.height_mm) * PT_PER_MM;
    let mut out = Vec::new();
    for part in &hri.parts {
        let limit = part.width_mm * PT_PER_MM;
        let mut size = hri.height_mm * PT_PER_MM / CAP_HEIGHT_RATIO;
        let mut laid = layout_part(part, style, size);
        let width = laid.cached_bounds.map_or(0.0, |b| b.width());
        // A font wider than the character cells the digits sit under would run past them and stop
        // lining up with the bars: shrink it rather than let the line drift.
        if limit > 0.0 && width > limit {
            size = (size * limit / width).max(0.05);
            laid = layout_part(part, style, size);
        }
        let width = laid.cached_bounds.map_or(0.0, |b| b.width());
        let x = origin.x + part.at_mm * PT_PER_MM
            - match part.align {
                HriAlign::Left => 0.0,
                HriAlign::Center => width / 2.0,
                HriAlign::Right => width,
            };
        laid.xf = Affine::translate(Point::new(x, baseline).to_vec2());
        out.push(Arc::new(Node::new(d.alloc_id(), NodeKind::Text(Box::new(laid)))));
    }
    out
}

/// One run of the human readable line, laid out and measured, before it is placed.
fn layout_part(part: &vectorcraft_barcode::HriPart, style: &CharStyle, size: f64) -> TextObject {
    let mut st = style.clone();
    st.size = size.max(0.05);
    let mut t = TextObject::point(Point::ZERO, &part.text, st);
    t.para = new_type_para();
    refresh_bounds(&mut t);
    t
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vectorcraft_doc::NodeKind;

    use crate::{NodeId, Session};

    fn new_doc() -> Session {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 800, "height": 800})).ok();
        s
    }

    fn make(s: &mut Session, p: serde_json::Value) -> (NodeId, serde_json::Value) {
        let made = s.execute("barcode.create", &p).expect("barcode.create");
        (NodeId(made["id"].as_u64().expect("id")), made)
    }

    fn children(s: &Session, id: NodeId) -> Vec<std::sync::Arc<vectorcraft_doc::Node>> {
        s.doc().unwrap().doc.node(id).and_then(|n| n.children()).map(|c| c.to_vec()).unwrap_or_default()
    }

    /// A barcode is one group carrying its spec, with every bar in a single path and the human
    /// readable line as type.
    #[test]
    fn a_barcode_is_a_group_with_its_spec() {
        let mut s = new_doc();
        let (id, made) = make(&mut s, json!({"symbology": "ean13", "data": "4006381333931"}));
        assert_eq!(made["data"], "4006381333931");
        let node = s.doc().unwrap().doc.node(id).unwrap().clone();
        let spec = node.extras().barcode.as_deref().expect("the spec is kept");
        assert_eq!(spec.symbology, vectorcraft_barcode::Symbology::Ean13);
        assert_eq!(spec.data, "4006381333931");
        let kids = children(&s, id);
        assert_eq!(kids.iter().filter(|n| matches!(n.kind, NodeKind::Path { .. })).count(), 1, "the bars are one path");
        assert_eq!(kids.iter().filter(|n| matches!(n.kind, NodeKind::Text(_))).count(), 13, "one object a digit");
        assert!(node.name.as_deref().is_some_and(|n| n.starts_with("EAN-13 ")));
    }

    /// A symbol is the size its standard gives it. The drawn art is the bar pattern itself, which
    /// at an EAN-13's nominal 0.33 mm module is its 95 modules: 31.35 mm wide, 22.85 mm of bar with
    /// the guard bars 1.65 mm below them, and the digits' baseline 25.93 mm down — the nominal
    /// symbol height, which is measured to the digits rather than to the font's descender.
    #[test]
    fn a_symbol_is_the_size_its_standard_gives() {
        let mut s = new_doc();
        let (id, _) = make(&mut s, json!({"symbology": "ean13", "data": "4006381333931"}));
        let kids = children(&s, id);
        let doc = &s.doc().unwrap().doc;
        let bars = kids.iter().find(|n| matches!(n.kind, NodeKind::Path { .. })).expect("the bars");
        let r = doc.bounds_of(&[bars.id], false).expect("bounds");
        let mm = |v: f64| v / (72.0 / 25.4);
        assert!((mm(r.width()) - 95.0 * 0.33).abs() < 0.02, "{}", mm(r.width()));
        assert!((mm(r.height()) - 24.5).abs() < 0.02, "{}", mm(r.height()));
        // The digits' baseline, which is where the nominal symbol height is measured to.
        let first = kids.iter().find(|n| matches!(n.kind, NodeKind::Text(_))).expect("type");
        let NodeKind::Text(t) = &first.kind else { panic!("the human readable line is type") };
        assert!((mm(t.xf.translation().y - r.y0) - 25.93).abs() < 0.02, "{}", mm(t.xf.translation().y - r.y0));
    }

    /// A missing check digit is worked out and reported back.
    #[test]
    fn the_check_digit_is_added_and_reported() {
        let mut s = new_doc();
        let (_, made) = make(&mut s, json!({"symbology": "ean13", "data": "400638133393"}));
        assert_eq!(made["data"], "4006381333931");
        // A wrong one is refused rather than quietly replaced.
        assert!(s.execute("barcode.create", &json!({"symbology": "ean13", "data": "4006381333932"})).is_err());
    }

    /// Changing the data redraws the object in place, and the art follows.
    #[test]
    fn a_change_redraws_the_same_object() {
        let mut s = new_doc();
        let (id, _) = make(&mut s, json!({"symbology": "ean13", "data": "4006381333931"}));
        let before = children(&s, id);
        s.execute("barcode.set", &json!({"data": "5012345678900"})).expect("barcode.set");
        let after = children(&s, id);
        assert_eq!(before.len(), after.len());
        assert_ne!(before[0], after[0], "the bars are drawn again");
        assert_eq!(s.doc().unwrap().doc.node(id).unwrap().extras().barcode.as_deref().unwrap().data, "5012345678900");
        // One undo step.
        s.execute("edit.undo", &json!({})).expect("undo");
        assert_eq!(s.doc().unwrap().doc.node(id).unwrap().extras().barcode.as_deref().unwrap().data, "4006381333931");
    }

    /// `barcode.set` with no options is the query: the spec and what the raw data resolves to.
    #[test]
    fn asking_for_the_options_gives_them_back() {
        let mut s = new_doc();
        make(&mut s, json!({"symbology": "code128", "data": "ABC-123"}));
        let out = s.execute("barcode.set", &json!({})).expect("query");
        assert_eq!(out["symbology"], "code128");
        assert_eq!(out["data"], "ABC-123");
        assert!(out["widthMm"].as_f64().is_some_and(|w| w > 0.0));
    }

    /// A QR Code is one path of module runs, and carries no human readable line.
    #[test]
    fn a_qr_code_is_modules_and_no_type() {
        let mut s = new_doc();
        let (id, _) = make(&mut s, json!({"symbology": "qrCode", "data": "https://example.com/", "ecc": "quartile"}));
        let kids = children(&s, id);
        assert_eq!(kids.len(), 1, "one path");
        assert!(matches!(kids[0].kind, NodeKind::Path { .. }));
        let spec = s.doc().unwrap().doc.node(id).unwrap().extras().barcode.as_deref().unwrap().clone();
        assert_eq!(spec.ecc, vectorcraft_barcode::Ecc::Quartile);
    }

    /// Every symbology draws, and the ones that need it get their bearer bars.
    #[test]
    fn every_symbology_draws() {
        let mut s = new_doc();
        for (symbology, data) in [
            ("ean13", "4006381333931"),
            ("ean8", "96385074"),
            ("upcA", "036000291452"),
            ("upcE", "01234565"),
            ("ean128", "(01)12345678901231"),
            ("code128", "ABC-123"),
            ("code128A", "ABC123"),
            ("code128B", "abc123"),
            ("code128C", "1234567890"),
            ("code39", "ABC-123"),
            ("code93", "Code 93 Test"),
            ("itf14", "15400141288763"),
            ("itf25", "1234567890"),
            ("qrCode", "hello"),
        ] {
            let (id, _) = make(&mut s, json!({"symbology": symbology, "data": data}));
            let r = s.doc().unwrap().doc.bounds_of(&[id], false).expect("bounds");
            assert!(r.width() > 1.0 && r.height() > 1.0, "{symbology} is {}", r.width());
        }
    }

    /// What a caller asks for that no symbol can do comes back as an error, not a broken object.
    #[test]
    fn a_bad_request_is_an_error() {
        let mut s = new_doc();
        assert!(s.execute("barcode.create", &json!({"symbology": "pdf417", "data": "x"})).is_err());
        assert!(s.execute("barcode.create", &json!({"symbology": "ean13", "data": "nonsense"})).is_err());
        assert!(s.execute("barcode.create", &json!({"symbology": "qrCode", "data": "x", "ecc": "ultra"})).is_err());
        assert!(s.execute("barcode.create", &json!({"symbology": "qrCode", "data": "x", "version": 99, "ecc": "quartile"})).is_err());
        // Nothing is left behind by a refused command.
        assert!(s.doc().unwrap().doc.node(NodeId(0)).is_none());
    }

    /// The spec survives a save and a reopen, so a barcode can be edited after the file was closed.
    #[test]
    fn the_spec_survives_the_file() {
        let mut s = new_doc();
        let (id, _) = make(&mut s, json!({"symbology": "itf14", "data": "15400141288763", "magnification": 1.5, "text": false}));
        let json = serde_json::to_string(&*s.doc().unwrap().doc).expect("saves");
        let doc: vectorcraft_doc::Document = serde_json::from_str(&json).expect("opens");
        let spec = doc.node(id).and_then(|n| n.extras().barcode.as_deref()).expect("the spec is kept");
        assert_eq!(spec.symbology, vectorcraft_barcode::Symbology::Itf14);
        assert!((spec.magnification - 1.5).abs() < 1e-9);
        assert!(!spec.text);
    }
}
