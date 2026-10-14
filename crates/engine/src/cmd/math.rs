//! Equations: Insert Equation and Edit Equation.
//!
//! An equation is a group carrying a [`MathSpec`]; its children (one filled path per glyph or
//! rule of the typeset LaTeX formula) are generated from the spec, so they are outlines with no
//! font behind them. Editing the formula regenerates them in place, keeping any move or scale the
//! user applied to the group since it was generated: the group's bounds against the spec's give
//! the scale, and its centre stays where it is.

use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::math::DEFAULT_MATH_SIZE;
use vectorcraft_doc::{Appearance, AppearanceItem, Document, FillLayer, MathSpec, Node, NodeId};
use vectorcraft_geom::{Affine, PathData, Point, Rect};
use vectorcraft_math::{MathError, MathOptions, find_math_face, typeset};
use vectorcraft_text::FontDb;

use super::pathops::shape_node;
use super::*;

/// The base font sizes an equation can be set at, in points.
const SIZE_RANGE: std::ops::RangeInclusive<f64> = 0.5..=2000.0;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "math.create",
            "Insert Equation",
            [],
            None,
            "{source: LaTeX math, size?: base font size in pt (0.5–2000, default 24), display?: bool (display style, default true), x?, y?: where the equation is centred (default the centre of the artboard), artboard?: index of the artboard to centre on (default the first)} → {id, bounds: [x0, y0, x1, y1]}; a group of outline paths, selected, that keeps its source for math.edit",
            has_doc,
            create
        ),
        cmd!(
            "math.edit",
            "Edit Equation",
            [],
            None,
            "{id?, source?: LaTeX math, size?: pt, display?: bool} typeset the selected equation (or the one containing the selected object, or `id`) again, keeping its position and scale; no source, size or display → the current {id, source, size, display}",
            has_doc,
            edit
        ),
    ]
}

/// An optional number parameter: `None` when absent (or null), an error when it is not a finite number.
fn number(p: &Value, key: &str, cmd: &str) -> Result<Option<f64>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_f64().filter(|n| n.is_finite()).map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be a number"))),
    }
}

/// An optional text parameter.
fn text<'a>(p: &'a Value, key: &str, cmd: &str) -> Result<Option<&'a str>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_str().map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be text"))),
    }
}

/// An optional flag parameter.
fn flag(p: &Value, key: &str, cmd: &str) -> Result<Option<bool>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_bool().map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be true or false"))),
    }
}

/// The base font size parameter, checked.
fn size_param(p: &Value, cmd: &str) -> Result<Option<f64>> {
    match number(p, "size", cmd)? {
        Some(n) if !SIZE_RANGE.contains(&n) => Err(bad(cmd, format!("`size` must be between {} and {} pt", SIZE_RANGE.start(), SIZE_RANGE.end()))),
        n => Ok(n),
    }
}

/// A coordinate parameter, checked.
fn coord_param(p: &Value, key: &str, cmd: &str) -> Result<Option<f64>> {
    match number(p, key, cmd)? {
        Some(n) if n.abs() > crate::MAX_COORD => Err(bad(cmd, format!("`{key}` is out of range"))),
        n => Ok(n),
    }
}

/// Typeset `spec`: its outlines in layout space (y down, the top of the ink at 0) with the ink's bounds.
fn typeset_spec(spec: &MathSpec, cmd: &str) -> Result<(Vec<vectorcraft_geom::BezPath>, Rect)> {
    let db = FontDb::global();
    let face = find_math_face(db).ok_or_else(|| EngineError::Other("No math font available".into()))?;
    let layout = typeset(db, &face, &spec.source, MathOptions { size_pt: spec.size, display: spec.display }).map_err(|e| match e {
        MathError::NoMathFont => EngineError::Other("No math font available".into()),
        e => bad(cmd, e.to_string()),
    })?;
    let b = layout.bounds;
    if ![b.x0, b.y0, b.x1, b.y1].iter().all(|v| v.is_finite()) {
        return Err(bad(cmd, "the equation has no usable size"));
    }
    Ok((layout.to_paths(db), b))
}

/// What a new equation's shapes look like: filled black, no stroke.
fn default_look() -> Appearance {
    Appearance { items: vec![AppearanceItem::Fill(FillLayer::new(Paint::solid(Color::BLACK)))], ..Appearance::default() }
}

/// The nodes for `paths`, each moved by `xf`, painted as `look`.
fn shapes(d: &mut Document, paths: &[vectorcraft_geom::BezPath], xf: Affine, look: &Appearance) -> Vec<Arc<Node>> {
    paths
        .iter()
        .filter_map(|bp| {
            let path = PathData::from_bezpath(bp).transformed(xf);
            if path.is_empty() {
                return None;
            }
            let mut n = shape_node(d, path, None);
            n.appearance = look.clone().into();
            Some(Arc::new(n))
        })
        .collect()
}

fn children_bounds(children: &[Arc<Node>]) -> Option<Rect> {
    children.iter().filter_map(|c| c.geometric_bounds()).reduce(|a, b| a.union(b))
}

/// Generate the children of equation group `id` from `spec`, centred on `at` and scaled by `scale`
/// (its x and y factors), and record where they ended up in the spec.
fn generate(d: &mut Document, id: NodeId, mut spec: MathSpec, at: Option<Point>, scale: (f64, f64), cmd: &str) -> Result<()> {
    let (paths, ink) = typeset_spec(&spec, cmd)?;
    let n = d.node(id).ok_or(EngineError::NoNode(id))?;
    // A recoloured equation stays recoloured: the first shape's paints are every shape's.
    let look = n.children().and_then(|c| c.first()).map_or_else(default_look, |c| (*c.appearance).clone());
    let at = at.or_else(|| n.children().and_then(|c| children_bounds(c)).map(|b| b.center())).unwrap_or(Point::ZERO);
    let xf = Affine::translate(at.to_vec2()) * Affine::scale_non_uniform(scale.0, scale.1) * Affine::translate(-ink.center().to_vec2());
    let children = shapes(d, &paths, xf, &look);
    if children.is_empty() {
        return Err(bad(cmd, "the equation draws nothing"));
    }
    // The ink box at the unscaled size, where the group stands: its size against the group's later
    // size is the scale the user has applied, however many times the formula has been set since.
    spec.bounds = Rect::from_center_size(at, (ink.width(), ink.height()));
    let n = d.node_mut(id).ok_or(EngineError::NoNode(id))?;
    if let Some(ch) = n.children_mut() {
        *ch = children;
    }
    n.extras_mut().math = Some(Box::new(spec));
    Ok(())
}

fn create(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "math.create";
    let source = text(p, "source", C)?.ok_or_else(|| bad(C, "missing text `source`"))?;
    let spec = MathSpec {
        source: source.to_string(),
        size: size_param(p, C)?.unwrap_or(DEFAULT_MATH_SIZE),
        display: flag(p, "display", C)?.unwrap_or(true),
        ..MathSpec::default()
    };
    let (x, y) = (coord_param(p, "x", C)?, coord_param(p, "y", C)?);
    let st = s.doc()?;
    let at = if let (Some(x), Some(y)) = (x, y) {
        Point::new(x, y)
    } else {
        let index = match p.get("artboard").filter(|v| !v.is_null()) {
            None => 0,
            Some(v) => v.as_u64().and_then(|i| usize::try_from(i).ok()).ok_or_else(|| bad(C, "`artboard` must be an artboard index"))?,
        };
        let centre = st.doc.artboards.get(index).map_or(Point::ZERO, |a| a.rect.center());
        Point::new(x.unwrap_or(centre.x), y.unwrap_or(centre.y))
    };
    let parent = st.target_parent()?;
    let (id, bounds) = s.edit("Insert Equation", |d, sel| {
        let gid = d.alloc_id();
        let mut g = Node::group(gid, vec![]);
        g.name = Some("Equation".into());
        d.insert(parent, usize::MAX, g)?;
        generate(d, gid, spec, Some(at), (1.0, 1.0), C)?;
        sel.set([gid]);
        let bounds = d.node(gid).and_then(|n| n.geometric_bounds()).unwrap_or(Rect::ZERO);
        Ok((gid, bounds))
    })?;
    Ok(json!({ "id": id.0, "bounds": [bounds.x0, bounds.y0, bounds.x1, bounds.y1] }))
}

/// The selected equation (or the one containing the selected object, or `id`).
fn target(s: &Session, p: &Value, cmd: &str) -> Result<NodeId> {
    let st = s.doc()?;
    if let Some(v) = p.get("id").filter(|v| !v.is_null())
        && v.as_u64().is_none()
    {
        return Err(bad(cmd, "`id` must be an object id"));
    }
    let start = id_param(p, "id").into_iter().chain(st.selection.objects.iter().copied());
    for id in start {
        let mut cur = Some(id);
        while let Some(c) = cur {
            if st.doc.node(c).is_some_and(|n| n.extras().math.is_some()) {
                return Ok(c);
            }
            cur = st.doc.parent_of(c);
        }
    }
    Err(bad(cmd, "select an equation"))
}

fn edit(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "math.edit";
    let (source, size, display) = (text(p, "source", C)?, size_param(p, C)?, flag(p, "display", C)?);
    let id = target(s, p, C)?;
    let mut spec = s.doc()?.doc.node(id).and_then(|n| n.extras().math.as_deref().cloned()).ok_or(EngineError::NoNode(id))?;
    if source.is_none() && size.is_none() && display.is_none() {
        return Ok(json!({ "id": id.0, "source": spec.source, "size": spec.size, "display": spec.display }));
    }
    // The group as it stands against its unscaled ink box: the user's scale (and where it was moved to).
    let placed = spec.bounds;
    let now = s.doc()?.doc.node(id).and_then(|n| n.children().and_then(|c| children_bounds(c)));
    let (at, scale) = match now {
        Some(now) if placed.width() > 1e-9 && placed.height() > 1e-9 && now.width() > 1e-9 && now.height() > 1e-9 => {
            (Some(now.center()), (now.width() / placed.width(), now.height() / placed.height()))
        }
        Some(now) => (Some(now.center()), (1.0, 1.0)),
        None => (Some(placed.center()), (1.0, 1.0)),
    };
    if let Some(src) = source {
        spec.source = src.to_string();
    }
    spec.size = size.unwrap_or(spec.size);
    spec.display = display.unwrap_or(spec.display);
    s.edit("Edit Equation", |d, _| generate(d, id, spec, at, scale, C))?;
    Ok(json!({ "id": id.0 }))
}
