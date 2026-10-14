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
//! [`MathDrawStyle`] (`mathdraw.style`, `mathdraw.setStyle`). A command that draws several things
//! puts them in one group ([`insert_output`]); the selection is kept, with a reference segment it
//! drew added to it, so the next command draws in the same frame.

mod graph;
mod style;

use serde_json::Value;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{
    Appearance, AppearanceItem, ColorMode, Dash, Document, FillLayer, LineRef, MathDrawStyle, Node, NodeId, NodeKind, StrokeLayer,
};
use vectorcraft_geom::{Affine, BezPath, CubicBez, ParamCurve, PathData, Point, Rect, shapes};

use super::pathops::shape_node;
use super::*;

pub fn specs() -> Vec<CommandSpec> {
    let mut v = style::specs();
    v.extend(graph::specs());
    v
}

/// The longest unit (points per math unit) a frame takes.
const MAX_UNIT: f64 = vectorcraft_doc::mathdraw::limits::STD_LENGTH.1;

/// The largest math coordinate a parameter takes (the drawing must also stay on the canvas).
pub(crate) const MAX_MATH: f64 = 1.0e6;

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
/// border drawn with the style's dot border line. The standard dot is black with a white border;
/// a hollow one (an open end) white with a black border.
pub(crate) fn dot(d: &mut Document, at: Point, style: &MathDrawStyle, fill: Color, border: Color) -> Node {
    let r = style.dot_size / 2.0;
    let path = shapes::ellipse(Rect::new(at.x - r, at.y - r, at.x + r, at.y + r));
    let mut n = shape_node(d, path, None);
    let mut look = line_look(style, style.dot_border, border);
    look.items.insert(0, AppearanceItem::Fill(FillLayer::new(Paint::solid(fill))));
    n.appearance = look.into();
    n
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

// ---------- parameters ----------

/// An optional flag parameter.
pub(crate) fn flag_param(p: &Value, key: &str, cmd: &str) -> Result<Option<bool>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_bool().map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be true or false"))),
    }
}

/// An optional finite number parameter.
pub(crate) fn number_param(p: &Value, key: &str, cmd: &str) -> Result<Option<f64>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_f64().filter(|n| n.is_finite()).map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be a number"))),
    }
}

/// An optional text parameter.
pub(crate) fn text_param<'a>(p: &'a Value, key: &str, cmd: &str) -> Result<Option<&'a str>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_str().map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be text"))),
    }
}

/// An optional range `[a, b]` of math coordinates, `a < b`.
pub(crate) fn range_param(p: &Value, key: &str, cmd: &str) -> Result<Option<(f64, f64)>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => range_value(v).map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be [low, high] with low < high, within ±{MAX_MATH}"))),
    }
}

fn range_value(v: &Value) -> Option<(f64, f64)> {
    finite_numbers::<2>(v).filter(|[a, b]| a < b && a.abs() <= MAX_MATH && b.abs() <= MAX_MATH).map(|[a, b]| (a, b))
}

/// An optional count parameter (a whole number), clamped into `range`.
pub(crate) fn count_param(p: &Value, key: &str, default: usize, range: std::ops::RangeInclusive<usize>, cmd: &str) -> Result<usize> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(default),
        Some(v) => v
            .as_u64()
            .map(|n| usize::try_from(n).unwrap_or(usize::MAX).clamp(*range.start(), *range.end()))
            .ok_or_else(|| bad(cmd, format!("`{key}` must be a whole number from {} to {}", range.start(), range.end()))),
    }
}

/// An optional list of math points `[[u, v], …]` (at most `max`).
pub(crate) fn points_param(p: &Value, key: &str, max: usize, cmd: &str) -> Result<Option<Vec<(f64, f64)>>> {
    let Some(v) = p.get(key).filter(|v| !v.is_null()) else { return Ok(None) };
    let err = || bad(cmd, format!("`{key}` must be a list of [u, v] points within ±{MAX_MATH} (at most {max})"));
    let a = v.as_array().filter(|a| a.len() <= max).ok_or_else(err)?;
    a.iter()
        .map(|e| finite_numbers::<2>(e).filter(|[u, v]| u.abs() <= MAX_MATH && v.abs() <= MAX_MATH).map(|[u, v]| (u, v)).ok_or_else(err))
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

// ---------- output ----------

/// Put what a command drew into the document, on top in `parent`: one node as it is, several in a
/// group named `name`. Fails, changing nothing, when the drawing would reach off the canvas.
/// Returns the id of the node inserted.
pub(crate) fn insert_output(d: &mut Document, parent: Option<NodeId>, name: &str, mut nodes: Vec<Node>, cmd: &str) -> Result<NodeId> {
    let limit = crate::MAX_COORD;
    for n in &nodes {
        if let Some(b) = n.geometric_bounds()
            && ![b.x0, b.y0, b.x1, b.y1].iter().all(|v| v.is_finite() && v.abs() <= limit)
        {
            return Err(bad(cmd, format!("the drawing would reach off the canvas (beyond ±{limit} pt): use a smaller range or unit")));
        }
    }
    let node = match nodes.len() {
        0 => return Err(bad(cmd, "there is nothing to draw")),
        1 => nodes.pop().ok_or_else(|| bad(cmd, "there is nothing to draw"))?,
        _ => {
            let mut g = Node::group(d.alloc_id(), nodes.into_iter().map(std::sync::Arc::new).collect());
            g.name = Some(name.to_string());
            g
        }
    };
    Ok(d.insert(parent, usize::MAX, node)?)
}

// ---------- paths read as functions ----------

/// Most samples a path is read with ([`PathFunction`]).
const MAX_PATH_SAMPLES: usize = 40_000;

/// A sample of a path read as a function: its math point and where on which segment it lies.
#[derive(Clone, Copy, Debug)]
struct Sample {
    x: f64,
    y: f64,
    seg: usize,
    t: f64,
}

/// Whether a local extremum is a maximum or a minimum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Extremum {
    Max,
    Min,
}

/// A path read as the graph of a function `y(x)` in a frame's math coordinates: how the
/// extremum, integral, derivative and combined graphs read the curves they are given.
///
/// Each segment of each subpath is sampled at evenly spaced parameters, and each subpath is one
/// *piece* with its samples in order of x: as drawn when x only grows (or only shrinks: then
/// reversed), else sorted (which reads a curve that turns back on itself, not a function, as well
/// as it can be read). Between two samples the value is the drawn segment's own, its x solved by
/// bisection, so a curve `mathdraw.graph` drew reads back exactly (between samples of different
/// segments that don't meet, linearly). Where no piece reaches, as across a vertical asymptote
/// that split a graph, there is no value.
#[derive(Clone, Debug)]
pub(crate) struct PathFunction {
    /// The segments, in math coordinates.
    segs: Vec<CubicBez>,
    /// Each piece's samples, by increasing x (no two at the same x).
    pieces: Vec<Vec<Sample>>,
}

impl PathFunction {
    /// `n` (a path or a compound path) read in `frame`; `None` for another object, or a path with
    /// no extent in x.
    pub fn from_node(n: &Node, frame: &Frame) -> Option<Self> {
        let paths: Vec<&PathData> = match &n.kind {
            NodeKind::Path { path, .. } => vec![path],
            NodeKind::Compound { children, .. } => children.iter().filter_map(|c| c.path_data()).collect(),
            _ => return None,
        };
        Self::from_paths(&paths, frame)
    }

    /// The subpaths of `paths` (document coordinates) read in `frame`.
    pub fn from_paths(paths: &[&PathData], frame: &Frame) -> Option<Self> {
        let to_math = |p: Point| {
            let (u, v) = frame.to_math(p);
            Point::new(u, v)
        };
        let total: usize = paths.iter().flat_map(|p| &p.subpaths).map(|sp| sp.segment_count()).sum();
        // Samples per segment: dense for a few segments, at least the ends for very many.
        let per = (MAX_PATH_SAMPLES / total.max(1)).clamp(1, 64);
        let mut segs = vec![];
        let mut pieces = vec![];
        for sp in paths.iter().flat_map(|p| &p.subpaths) {
            let mut samples = vec![];
            for i in 0..sp.segment_count() {
                let c = sp.segment(i);
                let c = CubicBez::new(to_math(c.p0), to_math(c.p1), to_math(c.p2), to_math(c.p3));
                let seg = segs.len();
                segs.push(c);
                for j in 0..=per {
                    let t = j as f64 / per as f64;
                    let q = c.eval(t);
                    if q.x.is_finite() && q.y.is_finite() {
                        samples.push(Sample { x: q.x, y: q.y, seg, t });
                    }
                }
            }
            let rising = samples.windows(2).all(|w| matches!(w, [a, b] if b.x >= a.x));
            let falling = samples.windows(2).all(|w| matches!(w, [a, b] if b.x <= a.x));
            if falling && !rising {
                samples.reverse();
            } else if !rising {
                samples.sort_by(|a, b| a.x.total_cmp(&b.x));
            }
            samples.dedup_by(|b, a| b.x == a.x);
            if samples.len() >= 2 {
                pieces.push(samples);
            }
        }
        (!pieces.is_empty()).then_some(Self { segs, pieces })
    }

    /// The x range the path covers, `(low, high)`.
    pub fn range(&self) -> (f64, f64) {
        let lo = self.pieces.iter().filter_map(|p| p.first()).map(|s| s.x).fold(f64::INFINITY, f64::min);
        let hi = self.pieces.iter().filter_map(|p| p.last()).map(|s| s.x).fold(f64::NEG_INFINITY, f64::max);
        (lo, hi)
    }

    /// The value at `x` (from the first piece that covers it), NaN where none does.
    pub fn at(&self, x: f64) -> f64 {
        for p in &self.pieces {
            let (Some(first), Some(last)) = (p.first(), p.last()) else { continue };
            if !(x >= first.x && x <= last.x) {
                continue;
            }
            let i = p.partition_point(|s| s.x < x);
            let Some(b) = p.get(i) else { continue };
            if b.x == x {
                return b.y;
            }
            let Some(a) = i.checked_sub(1).and_then(|i| p.get(i)) else { continue };
            return self.between(a, b, x);
        }
        f64::NAN
    }

    /// The value at `x` between neighbouring samples `a` and `b` (`a.x < x < b.x`).
    fn between(&self, a: &Sample, b: &Sample, x: f64) -> f64 {
        // The segment the two lie on, or one of them reaching the other at its end (a joint).
        let mut on = None;
        if a.seg == b.seg {
            on = Some((a.seg, a.t, b.t));
        } else {
            'find: for (s, o) in [(a, b), (b, a)] {
                let Some(c) = self.segs.get(s.seg) else { continue };
                for end in [0.0, 1.0] {
                    let q = c.eval(end);
                    if (q.x - o.x).abs() <= 1e-12 * (1.0 + o.x.abs()) && (q.y - o.y).abs() <= 1e-12 * (1.0 + o.y.abs()) {
                        on = Some(if s.seg == a.seg { (s.seg, a.t, end) } else { (s.seg, end, b.t) });
                        break 'find;
                    }
                }
            }
        }
        if let Some(y) = on.and_then(|(seg, t0, t1)| self.solve(seg, t0, t1, x)) {
            return y;
        }
        a.y + (b.y - a.y) * (x - a.x) / (b.x - a.x)
    }

    /// The y of segment `seg` where its x is `x`, between parameters `t0` (x below) and `t1`.
    fn solve(&self, seg: usize, mut t0: f64, mut t1: f64, x: f64) -> Option<f64> {
        let c = self.segs.get(seg)?;
        let f = |t: f64| c.eval(t).x - x;
        if f(t0) > 0.0 || f(t1) < 0.0 {
            return None;
        }
        for _ in 0..60 {
            let m = 0.5 * (t0 + t1);
            if m == t0 || m == t1 {
                break;
            }
            if f(m) < 0.0 {
                t0 = m;
            } else {
                t1 = m;
            }
        }
        let y = c.eval(0.5 * (t0 + t1)).y;
        y.is_finite().then_some(y)
    }

    /// The local maxima and minima, `(x, y)` with their kind, by increasing x within each piece:
    /// where the change between samples turns from rising to falling or back (a flat stretch
    /// between is skipped), refined by a golden-section search on the curve.
    pub fn extrema(&self) -> Vec<(Point, Extremum)> {
        let mut out: Vec<(Point, Extremum)> = vec![];
        for p in &self.pieces {
            let scale = p.iter().map(|s| s.y.abs()).fold(1.0, f64::max);
            let eps = 1e-12 * scale;
            // The sign of the last change that wasn't flat, and the index where it ended.
            let mut last: Option<(i8, usize)> = None;
            for i in 1..p.len() {
                let (Some(a), Some(b)) = (p.get(i - 1), p.get(i)) else { continue };
                let d = b.y - a.y;
                let sign = if d > eps {
                    1
                } else if d < -eps {
                    -1
                } else {
                    continue;
                };
                if let Some((was, end)) = last
                    && was != sign
                    && let Some(lo) = end.checked_sub(1).and_then(|j| p.get(j))
                {
                    let kind = if was > 0 { Extremum::Max } else { Extremum::Min };
                    let x = self.golden(lo.x, b.x, kind);
                    let y = self.at(x);
                    if y.is_finite() && !out.iter().any(|(q, _)| (q.x - x).abs() <= 1e-9 * (1.0 + x.abs())) {
                        out.push((Point::new(x, y), kind));
                    }
                }
                last = Some((sign, i));
            }
        }
        out
    }

    /// Where the curve is highest (or lowest) between `a` and `b`.
    fn golden(&self, mut a: f64, mut b: f64, kind: Extremum) -> f64 {
        const R: f64 = 0.618_033_988_749_894_9;
        let better = |u: f64, v: f64| match kind {
            Extremum::Max => u > v,
            Extremum::Min => u < v,
        };
        let value = |x: f64| {
            let y = self.at(x);
            if y.is_finite() {
                y
            } else if kind == Extremum::Max {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        };
        let (mut c, mut d) = (b - R * (b - a), a + R * (b - a));
        let (mut fc, mut fd) = (value(c), value(d));
        for _ in 0..80 {
            if (b - a).abs() <= 1e-12 * (1.0 + a.abs().max(b.abs())) {
                break;
            }
            if better(fc, fd) {
                (b, d, fd) = (d, c, fc);
                c = b - R * (b - a);
                fc = value(c);
            } else {
                (a, c, fc) = (c, d, fd);
                d = a + R * (b - a);
                fd = value(d);
            }
        }
        0.5 * (a + b)
    }
}
