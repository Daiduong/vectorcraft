//! Math drawing (`mathdraw.*`): coordinate axes, graphs, marks and constructions for math
//! textbooks and exams, after a math illustration tool's feature set.
//!
//! Everything is drawn in a coordinate frame ([`Frame`]): a doc-space origin and the length of one
//! math unit. It comes from the command's `origin` and `unit`, else from the **reference segment**
//! (a magenta straight segment in the selection: its bounding box's bottom-left corner is the
//! origin and its width the unit), else from the centre of the artboard with the style's standard
//! length as the unit, where a magenta reference segment is drawn too ([`resolve_frame`]).
//!
//! What the commands draw are plain paths and groups, painted from the document's
//! [`MathDrawStyle`] (`mathdraw.style`, `mathdraw.setStyle`).
// The frame and drawing helpers below are for the drawing commands (axes, graphs, marks), which
// come next; until they do, only the tests use them.
#![cfg_attr(not(test), expect(dead_code, reason = "helpers for the mathdraw drawing commands"))]

mod style;

use serde_json::Value;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{
    Appearance, AppearanceItem, ColorMode, Dash, Document, FillLayer, LineRef, MathDrawStyle, Node, NodeId, NodeKind, StrokeLayer,
};
use vectorcraft_geom::{Affine, BezPath, CubicBez, PathData, Point, Rect, shapes};

use super::pathops::shape_node;
use super::*;

pub fn specs() -> Vec<CommandSpec> {
    style::specs()
}

/// The longest unit (points per math unit) a frame takes.
const MAX_UNIT: f64 = vectorcraft_doc::mathdraw::limits::STD_LENGTH.1;

/// A math coordinate frame on the page: math `(u, v)` (y up) sits at
/// `(origin.x + u·unit, origin.y − v·unit)` in the document (y down).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Frame {
    /// Where math (0, 0) is, in document coordinates.
    pub origin: Point,
    /// Points per math unit.
    pub unit: f64,
}

impl Frame {
    /// The document point of math `(u, v)`.
    pub fn to_doc(self, u: f64, v: f64) -> Point {
        Point::new(self.origin.x + u * self.unit, self.origin.y - v * self.unit)
    }

    /// The math coordinates of document point `p`.
    pub fn to_math(self, p: Point) -> (f64, f64) {
        ((p.x - self.origin.x) / self.unit, (self.origin.y - p.y) / self.unit)
    }

    /// Math → document, as an affine map.
    pub fn affine(self) -> Affine {
        Affine::new([self.unit, 0.0, 0.0, -self.unit, self.origin.x, self.origin.y])
    }
}

/// Where a frame came from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum FrameSource {
    /// The command's `origin` and `unit`.
    Params,
    /// The reference segment in the selection.
    Reference(NodeId),
    /// The artboard's centre and the standard length.
    Artboard,
}

/// A resolved frame, and what using it does to the document ([`FramePlan::apply`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FramePlan {
    pub frame: Frame,
    pub source: FrameSource,
    /// A lone selected segment that isn't magenta yet, which becomes the reference.
    recolor: Option<NodeId>,
    /// Draw a reference segment for a frame made from the artboard.
    make_reference: bool,
}

impl FramePlan {
    /// Inside the command's edit: turn the segment the frame came from magenta, or draw the
    /// reference segment of a frame made from the artboard (in `parent`, on top). Returns the
    /// reference segment, if there is one.
    pub fn apply(&self, d: &mut Document, parent: Option<NodeId>) -> Result<Option<NodeId>> {
        let magenta = magenta_in(d);
        if let Some(id) = self.recolor {
            let n = d.node_mut(id).ok_or(EngineError::NoNode(id))?;
            let mut look = (*n.appearance).clone();
            match look.stroke_mut() {
                Some(s) => s.paint = Paint::solid(magenta),
                None => look.set_stroke(Paint::solid(magenta)),
            }
            n.appearance = look.into();
        }
        if self.make_reference {
            let a = self.frame.origin;
            let path = shapes::line(a, Point::new(a.x + self.frame.unit, a.y));
            let style = d.math_draw.clone();
            let mut n = shape_node(d, path, None);
            n.appearance = line_look(&style, LineRef::Solid(4), magenta).into();
            return Ok(Some(d.insert(parent, usize::MAX, n)?));
        }
        Ok(match self.source {
            FrameSource::Reference(id) => Some(id),
            _ => None,
        })
    }
}

/// The frame a command draws in (see the module docs): `origin: [x, y]` and `unit` win over the
/// selection's reference segment, which wins over the artboard (`artboard`: index, default the
/// first) with the style's standard length; `makeReference: false` draws no reference segment
/// for the last.
pub(crate) fn resolve_frame(s: &Session, p: &Value, cmd: &str) -> Result<FramePlan> {
    let origin = match p.get("origin").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(
            finite_numbers::<2>(v)
                .filter(|[x, y]| x.abs() <= crate::MAX_COORD && y.abs() <= crate::MAX_COORD)
                .map(|[x, y]| Point::new(x, y))
                .ok_or_else(|| bad(cmd, "`origin` must be [x, y] on the canvas"))?,
        ),
    };
    let unit = match p.get("unit").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(
            v.as_f64()
                .filter(|u| u.is_finite() && *u > 0.0 && *u <= MAX_UNIT)
                .ok_or_else(|| bad(cmd, format!("`unit` must be a length above 0 and at most {MAX_UNIT} pt")))?,
        ),
    };
    let make_reference = match p.get("makeReference").filter(|v| !v.is_null()) {
        None => true,
        Some(v) => v.as_bool().ok_or_else(|| bad(cmd, "`makeReference` must be true or false"))?,
    };
    let artboard = match p.get("artboard").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(v.as_u64().and_then(|i| usize::try_from(i).ok()).ok_or_else(|| bad(cmd, "`artboard` must be an artboard index"))?),
    };
    let st = s.doc()?;
    let d = &st.doc;
    let with = |frame: Frame| Frame { origin: origin.unwrap_or(frame.origin), unit: unit.unwrap_or(frame.unit) };
    if let (Some(origin), Some(unit)) = (origin, unit) {
        return Ok(FramePlan { frame: Frame { origin, unit }, source: FrameSource::Params, recolor: None, make_reference: false });
    }
    // The selected straight segments (in selected groups too), in selection order.
    let mut segments: Vec<(NodeId, Rect, bool)> = vec![];
    for id in &st.selection.objects {
        let Some(n) = d.node(*id) else { continue };
        n.walk(&mut |c| {
            if let Some(b) = straight_segment(c) {
                segments.push((c.id, b, is_magenta(c)));
            }
        });
    }
    let reference = segments.iter().find(|s| s.2).or(match segments.as_slice() {
        [only] => Some(only),
        _ => None,
    });
    if let Some(&(id, b, magenta)) = reference {
        if b.width() <= 1e-9 || b.width() > MAX_UNIT {
            return Err(bad(cmd, "the reference segment must be wider than 0 (its width is the unit)"));
        }
        let frame = with(Frame { origin: Point::new(b.x0, b.y1), unit: b.width() });
        return Ok(FramePlan { frame, source: FrameSource::Reference(id), recolor: (!magenta).then_some(id), make_reference: false });
    }
    let centre = match artboard {
        Some(i) => d.artboards.get(i).map(|a| a.rect.center()).ok_or_else(|| bad(cmd, format!("there is no artboard {i}")))?,
        None => d.artboards.first().map_or(Point::ZERO, |a| a.rect.center()),
    };
    let frame = with(Frame { origin: centre, unit: d.math_draw.std_length });
    Ok(FramePlan { frame, source: FrameSource::Artboard, recolor: None, make_reference })
}

/// The bounds of `n` when it is a straight segment: an open path of two anchors whose handles
/// (if any) lie on the line between them.
fn straight_segment(n: &Node) -> Option<Rect> {
    let NodeKind::Path { path, .. } = &n.kind else { return None };
    let [sp] = path.subpaths.as_slice() else { return None };
    let [a, b] = sp.anchors.as_slice() else { return None };
    if sp.closed {
        return None;
    }
    let chord = b.p - a.p;
    let len2 = chord.hypot2();
    if !len2.is_finite() || len2 <= 0.0 {
        return None;
    }
    let on_line = |h: Point| (h - a.p).cross(chord).abs() <= 1e-6 * len2;
    (on_line(a.h_out) && on_line(b.h_in)).then(|| Rect::from_points(a.p, b.p))
}

/// Whether `n`'s stroke is magenta (#ff00ff, or CMYK 0/100/0/0).
fn is_magenta(n: &Node) -> bool {
    let Some(s) = n.appearance.stroke().filter(|s| s.visible) else { return false };
    let Paint::Solid { color, .. } = &s.paint else { return false };
    let [r, g, b] = color.to_rgb_uncalibrated();
    (r - 1.0).abs() < 0.02 && g.abs() < 0.02 && (b - 1.0).abs() < 0.02
}

/// Magenta in the document's colour mode.
fn magenta_in(d: &Document) -> Color {
    match d.color_mode {
        ColorMode::Cmyk => Color::cmyk(0.0, 1.0, 0.0, 0.0),
        ColorMode::Rgb => Color::rgb(1.0, 0.0, 1.0),
    }
}

/// A stroke-only look: `line` of `style` in `color`.
pub(crate) fn line_look(style: &MathDrawStyle, line: LineRef, color: Color) -> Appearance {
    let (width, pattern) = style.stroke(line);
    let mut stroke = StrokeLayer::new(Paint::solid(color), width);
    if !pattern.is_empty() {
        stroke.dash = Some(Dash { pattern, offset: 0.0, align_corners: false });
    }
    Appearance { items: vec![AppearanceItem::Stroke(stroke)], ..Appearance::default() }
}

/// A path node for `path` stroked with `line` of `style` in `color`, unfilled.
pub(crate) fn stroked(d: &mut Document, path: PathData, style: &MathDrawStyle, line: LineRef, color: Color) -> Node {
    let mut n = shape_node(d, path, None);
    n.appearance = line_look(style, line, color).into();
    n
}

/// A dot at `at`: a circle of the style's dot size filled with `fill`, with a `border` coloured
/// border drawn with the style's dot border line.
pub(crate) fn dot(d: &mut Document, at: Point, style: &MathDrawStyle, fill: Color, border: Color) -> Node {
    let r = style.dot_size / 2.0;
    let path = shapes::ellipse(Rect::new(at.x - r, at.y - r, at.x + r, at.y + r));
    let mut n = shape_node(d, path, None);
    let mut look = line_look(style, style.dot_border, border);
    look.items.insert(0, AppearanceItem::Fill(FillLayer::new(Paint::solid(fill))));
    n.appearance = look.into();
    n
}

/// The standard dot: black with a white border.
pub(crate) fn black_dot(d: &mut Document, at: Point, style: &MathDrawStyle) -> Node {
    dot(d, at, style, Color::BLACK, Color::WHITE)
}

/// Curves fitted in math coordinates (`vectorcraft_math::fit`), on the page: a subpath per run.
pub(crate) fn curve_path(runs: &[Vec<CubicBez>], frame: &Frame) -> PathData {
    let mut bp: BezPath = vectorcraft_math::fit::to_bezpath(runs);
    bp.apply_affine(frame.affine());
    PathData::from_bezpath(&bp)
}

/// An optional colour parameter (`"#rrggbb"` and the other forms commands take).
pub(crate) fn color_param(p: &Value, key: &str, cmd: &str) -> Result<Option<Color>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => color_value(v).map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be a colour such as \"#000000\""))),
    }
}

/// An optional line parameter: `"solid1"` … `"solid5"`, `"dashed1"` … `"dashed5"`.
pub(crate) fn line_param(p: &Value, key: &str, cmd: &str) -> Result<Option<LineRef>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v
            .as_str()
            .and_then(LineRef::parse)
            .map(Some)
            .ok_or_else(|| bad(cmd, format!("`{key}` must be a line, \"solid1\"…\"solid5\" or \"dashed1\"…\"dashed5\""))),
    }
}
