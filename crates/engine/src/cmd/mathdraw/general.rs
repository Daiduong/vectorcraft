//! General commands: colours (convert to black, change to one colour), transforms about a
//! reference point, and labels set as equations at math coordinates.
//!
//! (Create Outlines for type is `type.createOutlines`; equations are outlines already.)

use std::collections::HashSet;

use serde_json::{Map, Value, json};
use vectorcraft_color::Color;
use vectorcraft_doc::{Node, NodeId, NodeKind};
use vectorcraft_geom::{Affine, Point};

use super::graph::{common, draw};
use super::*;
use crate::cmd::colorcmds::{Recolor, Scope};
use crate::cmd::edit::selected_roots;

/// The largest scale factor `mathdraw.transform` takes.
const MAX_SCALE: f64 = 10_000.0;
/// The steepest shear, in degrees.
const MAX_SHEAR: f64 = 89.0;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "mathdraw.toBlack",
            "Convert to Black",
            [],
            None,
            "{ids?: [object ids] (default the selection), stroke?: bool (default true), fill?: bool (default true)} → {changed: paints changed}; the colours of the strokes and fills (of type and inside groups too) become black, K 100 % in a CMYK document; what has no stroke or fill keeps none",
            has_doc,
            to_black
        ),
        cmd!(
            "mathdraw.recolor",
            "Change Color Tone",
            [],
            None,
            "{color: \"#rrggbb\" (or another colour form), ids?: [object ids] (default the selection), stroke?: bool (default true), fill?: bool (default true)} → {changed: paints changed}; the colours of the strokes and fills (of type and inside groups too) become `color`; what has no stroke or fill keeps none",
            has_doc,
            recolor
        ),
        cmd!(
            "mathdraw.transform",
            "Transform about Reference Point",
            [],
            None,
            "{ids?: [object ids] (default the selection), center?: [x, y] pt (default the centre of a small magenta-filled dot selected, which stays; else the centre of the objects' bounds), scale?: [sx, sy] factors, shear?: degrees (horizontal, below ±89), rotate?: degrees counter-clockwise, reflect?: {axisAngle: degrees} (across the line through the centre at that angle), keepTextSize?: bool (objects holding type or equations only move, as their centre does), copy?: bool (transform copies and keep the originals)} → {ids: [transformed objects or copies], center: [x, y]}; scale, then shear, rotate and reflect, about the centre; stroke weights stay",
            has_doc,
            transform
        ),
        cmd!(
            "mathdraw.label",
            "Math Label",
            [],
            None,
            "{latex: LaTeX math, at: [u, v] math coordinates, size?: pt (default the style's label size), anchor?: \"center\" (default) | \"baseline-start\" | \"left\" | \"right\" | \"top\" | \"bottom\" | \"top-left\" | \"top-right\" | \"bottom-left\" | \"bottom-right\" (the point of the equation put at `at`: its box's, or the start of its baseline), color?: \"#rrggbb\", origin?: [x, y] pt, unit?: pt per math unit, makeReference?: bool (default true), artboard?: index} → {id, reference?}; an equation (math.edit edits it); needs a math font",
            has_doc,
            label
        ),
    ]
}

// ---------- colours ----------

/// `stroke` and `fill` (both default true; not both false), and the objects (`ids`, else the
/// selection: some).
fn paint_scope(s: &Session, p: &Value, cmd: &str) -> Result<(Scope, Vec<NodeId>)> {
    if !p.is_object() && !p.is_null() {
        return Err(bad(cmd, "the parameters must be an object"));
    }
    let stroke = flag_param(p, "stroke", cmd)?.unwrap_or(true);
    let fill = flag_param(p, "fill", cmd)?.unwrap_or(true);
    if !stroke && !fill {
        return Err(bad(cmd, "`stroke` and `fill` are both false: there is nothing to change"));
    }
    let ids = checked_ids_or_selection(s, p, cmd)?;
    if ids.is_empty() {
        return Err(bad(cmd, "select objects (or give `ids`)"));
    }
    Ok((Scope { fill, stroke, images: false, patterns: false }, ids))
}

/// Map the colours of the strokes and fills of `ids` in `scope` to `color`, as one undo step.
fn paint_all(s: &mut Session, label: &str, scope: Scope, ids: &[NodeId], color: Color) -> Result<Value> {
    let to = move |_: Color| color;
    let changed = s.edit(label, |d, _| Ok(Recolor::new(scope, &to).run(d, ids)))?;
    Ok(json!({ "changed": changed }))
}

fn to_black(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.toBlack";
    let (scope, ids) = paint_scope(s, p, C)?;
    let black = match s.doc()?.doc.color_mode {
        ColorMode::Cmyk => Color::cmyk(0.0, 0.0, 0.0, 1.0),
        ColorMode::Rgb => Color::BLACK,
    };
    paint_all(s, "Convert to Black", scope, &ids, black)
}

fn recolor(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.recolor";
    let color = color_param(p, "color", C)?.ok_or_else(|| bad(C, "missing `color`, such as \"#0000ff\""))?;
    let (scope, ids) = paint_scope(s, p, C)?;
    paint_all(s, "Change Color Tone", scope, &ids, color)
}

// ---------- transform about a reference point ----------

/// The centre of `n` when it is a small dot filled magenta (the centre of a transform): a round
/// path less than `max_r` in radius.
fn centre_dot(n: &Node, max_r: f64) -> Option<Point> {
    if !matches!(n.kind, NodeKind::Path { .. }) {
        return None;
    }
    let fill = n.appearance.fill().filter(|f| f.visible)?;
    if !paint_is_magenta(&fill.paint) {
        return None;
    }
    let b = n.geometric_bounds()?;
    let (w, h) = (b.width(), b.height());
    (w > 0.0 && (w - h).abs() <= 0.05 * w.max(h) && w.max(h) / 2.0 < max_r).then(|| b.center())
}

/// Whether `n` is or holds type or an equation.
fn holds_type(n: &Node) -> bool {
    let mut found = false;
    n.walk(&mut |c| found |= matches!(c.kind, NodeKind::Text(_)) || c.extras().math.is_some());
    found
}

/// The transform's matrix about the origin: scale, then shear, rotate and reflect (angles
/// counter-clockwise as seen; the page's y runs down).
fn matrix(scale: Option<[f64; 2]>, shear: Option<f64>, rotate: Option<f64>, reflect: Option<f64>) -> Affine {
    let mut m = Affine::IDENTITY;
    if let Some([sx, sy]) = scale {
        m = Affine::scale_non_uniform(sx, sy) * m;
    }
    if let Some(a) = shear {
        m = Affine::new([1.0, 0.0, -a.to_radians().tan(), 1.0, 0.0, 0.0]) * m;
    }
    if let Some(a) = rotate {
        m = Affine::rotate(-a.to_radians()) * m;
    }
    if let Some(a) = reflect {
        let t = -a.to_radians();
        m = Affine::rotate(t) * Affine::scale_non_uniform(1.0, -1.0) * Affine::rotate(-t) * m;
    }
    m
}

fn transform(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.transform";
    if !p.is_object() && !p.is_null() {
        return Err(bad(C, "the parameters must be an object"));
    }
    let center = match p.get("center").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(
            finite_numbers::<2>(v)
                .filter(|c| c.iter().all(|v| v.abs() <= crate::MAX_COORD))
                .map(|[x, y]| Point::new(x, y))
                .ok_or_else(|| bad(C, "`center` must be [x, y] on the canvas"))?,
        ),
    };
    let scale = match p.get("scale").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(
            finite_numbers::<2>(v)
                .filter(|c| c.iter().all(|k| *k != 0.0 && k.abs() <= MAX_SCALE && k.abs() >= 1.0 / MAX_SCALE))
                .ok_or_else(|| bad(C, format!("`scale` must be [sx, sy], factors not 0, between 1/{MAX_SCALE} and {MAX_SCALE} in size")))?,
        ),
    };
    let shear = number_param(p, "shear", C)?;
    if shear.is_some_and(|a| a.abs() >= MAX_SHEAR) {
        return Err(bad(C, format!("`shear` must be an angle below ±{MAX_SHEAR} degrees")));
    }
    let rotate = number_param(p, "rotate", C)?;
    let reflect = match p.get("reflect").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(
            v.as_object()
                .filter(|o| o.len() == 1)
                .and_then(|o| o.get("axisAngle"))
                .and_then(Value::as_f64)
                .filter(|a| a.is_finite())
                .ok_or_else(|| bad(C, "`reflect` must be {axisAngle: degrees}"))?,
        ),
    };
    if [rotate, reflect].iter().flatten().any(|a| a.abs() > 1e6) {
        return Err(bad(C, "`rotate` and `reflect.axisAngle` must be within ±1000000 degrees"));
    }
    let keep_text = flag_param(p, "keepTextSize", C)?.unwrap_or(false);
    let copy = flag_param(p, "copy", C)?.unwrap_or(false);
    if scale.is_none() && shear.is_none() && rotate.is_none() && reflect.is_none() {
        return Err(bad(C, "give `scale`, `shear`, `rotate` or `reflect`"));
    }
    let mut ids = checked_ids_or_selection(s, p, C)?;
    let st = s.doc()?;
    let mut dot = None;
    let center = match center {
        Some(c) => c,
        None => {
            let max_r = st.doc.math_draw.dot_size * 1.25 / 2.0;
            let found = selected_roots(s)?.into_iter().find_map(|id| st.doc.node(id).and_then(|n| centre_dot(n, max_r)).map(|c| (id, c)));
            match found {
                Some((id, c)) => {
                    // The dot marks the centre: it stays where it is.
                    ids.retain(|i| *i != id);
                    dot = Some(id);
                    c
                }
                None => ids
                    .iter()
                    .filter_map(|id| st.doc.node(*id).and_then(|n| n.geometric_bounds()))
                    .reduce(|a, b| a.union(b))
                    .map(|b| b.center())
                    .ok_or_else(|| bad(C, "select objects to transform (or give `ids`)"))?,
            }
        }
    };
    if ids.is_empty() {
        return Err(bad(C, "select objects to transform (or give `ids`)"));
    }
    let xf = Affine::translate(center.to_vec2()) * matrix(scale, shear, rotate, reflect) * Affine::translate(-center.to_vec2());
    let moved_only: HashSet<NodeId> =
        if keep_text { ids.iter().filter(|id| st.doc.node(**id).is_some_and(holds_type)).copied().collect() } else { HashSet::new() };
    // What each object gets: type moves by where its centre goes.
    let of = |n: &Node| -> Affine {
        match n.geometric_bounds().filter(|_| moved_only.contains(&n.id)) {
            Some(b) => Affine::translate(xf * b.center() - b.center()),
            None => xf,
        }
    };
    let out = s.edit("Transform about Reference Point", |d, sel| {
        if copy {
            let copies = crate::cmd::edit::copy_above(d, &ids, |_, n, c| c.transform(of(n), false))?;
            sel.set(copies.iter().copied().chain(dot));
            return Ok(copies);
        }
        for id in &ids {
            let n = d.node_mut(*id).ok_or(EngineError::NoNode(*id))?;
            let a = of(n);
            n.transform(a, false);
        }
        Ok(ids.clone())
    })?;
    Ok(json!({ "ids": out.iter().map(|i| i.0).collect::<Vec<_>>(), "center": [center.x, center.y] }))
}

// ---------- labels ----------

/// Where on an equation's box `anchor` is, as fractions of its width and height from the top
/// left; `None` for the start of the baseline.
fn anchor_value(anchor: &str) -> Option<Option<(f64, f64)>> {
    Some(Some(match anchor {
        "baseline-start" => return Some(None),
        "center" => (0.5, 0.5),
        "left" => (0.0, 0.5),
        "right" => (1.0, 0.5),
        "top" => (0.5, 0.0),
        "bottom" => (0.5, 1.0),
        "top-left" => (0.0, 0.0),
        "top-right" => (1.0, 0.0),
        "bottom-left" => (0.0, 1.0),
        "bottom-right" => (1.0, 1.0),
        _ => return None,
    }))
}

fn label(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.label";
    let latex =
        text_param(p, "latex", C)?.filter(|t| !t.trim().is_empty()).ok_or_else(|| bad(C, "missing `latex`: the label as LaTeX math"))?.to_string();
    let at = p.get("at").filter(|v| !v.is_null()).ok_or_else(|| bad(C, "missing `at`: [u, v] math coordinates")).and_then(|v| {
        finite_numbers::<2>(v)
            .filter(|c| c.iter().all(|v| v.abs() <= MAX_MATH))
            .ok_or_else(|| bad(C, format!("`at` must be [u, v] within ±{MAX_MATH}")))
    })?;
    let size = number_param(p, "size", C)?;
    let (lo, hi) = vectorcraft_doc::mathdraw::limits::LABEL_SIZE;
    if size.is_some_and(|v| !(lo..=hi).contains(&v)) {
        return Err(bad(C, format!("`size` must be from {lo} to {hi} pt")));
    }
    let anchor = match text_param(p, "anchor", C)? {
        None => Some((0.5, 0.5)),
        Some(a) => anchor_value(a).ok_or_else(|| bad(C, "`anchor` must be \"center\", \"baseline-start\", \"left\", \"right\", \"top\", \"bottom\", \"top-left\", \"top-right\", \"bottom-left\" or \"bottom-right\""))?,
    };
    let c = common(s, p, C)?;
    let size = size.unwrap_or(c.style.label_size);
    draw(s, &c, "Math Label", "Label", C, |d, frame| {
        let [u, v] = at;
        let pt = frame.to_doc(u, v);
        let mut n = crate::cmd::math::equation_on_baseline(d, &latex, size, pt, c.color, C)?;
        if let Some((fx, fy)) = anchor
            && let Some(b) = n.geometric_bounds()
        {
            let from = Point::new(b.x0 + fx * b.width(), b.y0 + fy * b.height());
            n.transform(Affine::translate(pt - from), false);
        }
        Ok((vec![n], Map::new()))
    })
}
