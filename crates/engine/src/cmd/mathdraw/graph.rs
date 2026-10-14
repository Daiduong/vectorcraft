//! Graphs of functions (2D): coordinate axes, points, the graphs of `y = f(x)`, parametric and
//! piecewise functions, circles, and graphs read from curves already drawn (their extrema,
//! integral, derivative, and arithmetic of several).
//!
//! Each command draws in the frame [`resolve_frame`] gives (the `origin`, `unit`,
//! `makeReference` and `artboard` parameters), is one undo step, and answers the ids of what it
//! drew: `id` (the group, or the one object), and per part (`curve`, `axes`, `dots`…).
//!
//! Curves fitted from expressions use [`vectorcraft_math::fit`]: `y = f(x)` is cut where it leaves
//! the band `yClip` (default −10…10), so asymptotes don't run off the page. Curves read from paths
//! use [`PathFunction`].

use serde_json::{Map, Value, json};
use vectorcraft_color::Color;
use vectorcraft_doc::mathdraw::AxisLabelKind;
use vectorcraft_doc::{Document, LineCap, LineJoin, LineRef, MathDrawStyle, Node, NodeId, NodeKind, StrokeLayer};
use vectorcraft_geom::{CubicBez, PathData, Point, Rect, SubPath, Vec2, shapes};
use vectorcraft_math::{Expr, FitOptions, fit_function, fit_parametric, fit_samples, parse_expr};

use super::*;

/// Most steps of a graph (its precision).
const MAX_PRECISION: usize = 2000;
/// Samples of a graph read from curves (integral, derivative, arithmetic): default and range.
const SAMPLES: (usize, usize, usize) = (100, 8, 4000);
/// Most points, circles or pieces a command takes.
const MAX_ITEMS: usize = 10_000;
const MAX_PIECES: usize = 8;
/// The band a function's graph is drawn in when `yClip` isn't given.
const DEFAULT_CLIP: (f64, f64) = (-10.0, 10.0);
/// The note answered for a label left out for want of a math font.
const NO_FONT: &str = "skipped (no math font)";

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "mathdraw.axes",
            "Coordinate Axes",
            [],
            None,
            "{xRange?: [a, b] math units (default [-2, 2]), yRange?: [a, b] (default [-2, 2]), fitSelection?: bool (the ranges the given ones don't set span the selected objects, with the origin), line?: line (default the style's axis line), color?: \"#rrggbb\", origin?: [x, y] pt, unit?: pt per math unit, makeReference?: bool (default true), artboard?: index} → {id, axes: [x, y], arrows: [x, y], labels: [x, y, O] ids or \"skipped (no math font)\", reference?}; x and y axes with arrowheads (the style's arrow type and ratio) and the labels x, y, O set in the math font at the style's label size and offsets",
            has_doc,
            axes
        ),
        cmd!(
            "mathdraw.points",
            "Points by Coordinates",
            [],
            None,
            "{coords: [[u, v], …] math coordinates, guides?: bool (default true: dashed lines to both axes), dot?: diameter pt (default the style's dot size), line?: line of the guides (default dashed3), color?: \"#rrggbb\", origin?, unit?, makeReference?, artboard?} → {id, dots: [ids], guides: [ids], reference?}",
            has_doc,
            points
        ),
        cmd!(
            "mathdraw.graph",
            "Graph of a Function",
            [],
            None,
            "{f: expression in x (\\sin x, x^2, \\frac{1}{x}…), xRange: [a, b], precision?: steps 1–2000 (default 20; one cubic per step, exact up to degree 3), yClip?: [low, high] (default [-10, 10]: the graph is cut where it leaves it), axes?: bool (draw axes covering the graph), label?: bool (\"y=f\" set at the curve's right end), labelAt?: \"end\", line?: line (default solid3), color?: \"#rrggbb\", origin?, unit?, makeReference?, artboard?} → {id, curve, axes?, label?: id or why it was skipped, reference?}",
            has_doc,
            graph
        ),
        cmd!(
            "mathdraw.parametric",
            "Parametric Curve",
            [],
            None,
            "{x: expression in t, y: expression in t, tRange: [a, b], precision?: steps 1–2000 (default 20), axes?: bool, label?: bool (\"(x, y)=(x(t), y(t))\" at the curve's end), labelAt?: \"end\", line?: line (default solid3), color?: \"#rrggbb\", origin?, unit?, makeReference?, artboard?} → {id, curve, axes?, label?, reference?}",
            has_doc,
            parametric
        ),
        cmd!(
            "mathdraw.circles",
            "Circles by Center",
            [],
            None,
            "{centers: [[u, v], …] math coordinates, radius: math units, line?: line (default solid3), color?: \"#rrggbb\", origin?, unit?, makeReference?, artboard?} → {id, circles: [ids], reference?}",
            has_doc,
            circles
        ),
        cmd!(
            "mathdraw.extrema",
            "Mark Maxima and Minima",
            [],
            None,
            "{id?: the curve (default the first open stroked curve selected), guides?: bool (dashed lines from each point to both axes), line?: line of the guides (default dashed3), color?: \"#rrggbb\", origin?, unit?, makeReference?, artboard?} → {id, maxima: [[x, y]…], minima: [[x, y]…], dots: [ids], guides: [ids], reference?}; the curve is read as y(x) in the frame",
            has_doc,
            extrema
        ),
        cmd!(
            "mathdraw.integral",
            "Integral Graph",
            [],
            None,
            "{id?: the curve (default the selected one), precision?: samples 8–4000 (default 100), c0At?: \"left\" (default: 0 at the curve's left end) | \"right\", line?: line (default solid3), color?: \"#rrggbb\", origin?, unit?, makeReference?, artboard?} → {id, reference?}; the graph of the curve's integral, the curve read as y(x)",
            has_doc,
            integral
        ),
        cmd!(
            "mathdraw.derivative",
            "Derivative Graph",
            [],
            None,
            "{id?: the curve (default the selected one), precision?: samples 8–4000 (default 100), line?: line (default solid3), color?: \"#rrggbb\", origin?, unit?, makeReference?, artboard?} → {id, reference?}; the graph of the curve's derivative, the curve read as y(x)",
            has_doc,
            derivative
        ),
        cmd!(
            "mathdraw.combine",
            "Graph Arithmetic",
            [],
            None,
            "{expr: expression in x and the curves f/g/h (f*g, f+h, x\\sqrt{f}+\\cos x…), ids?: {f?, g?, h?: curve ids} (default the selected curves by stroke: black f, blue g, yellow h), precision?: samples 8–4000 (default 100), line?: line (default solid3), color?: \"#rrggbb\", origin?, unit?, makeReference?, artboard?} → {id, range: [a, b], reference?}; drawn over the x range the curves used share",
            has_doc,
            combine
        ),
        cmd!(
            "mathdraw.piecewise",
            "Piecewise Function Graph",
            [],
            None,
            "{pieces: [{f: expression in x, range: [a, b], precision?: 1–2000 (default 20), leftEnd?|rightEnd?: \"open\" | \"closed\" | {value: v}}, …] (1–8), axes?: bool, yClip?: [low, high] (default [-10, 10]), line?: line (default solid3), color?: \"#rrggbb\", origin?, unit?, makeReference?, artboard?} → {id, curves: [ids], dots: [ids], closed: [[x, y]…], open: [[x, y]…], axes?, reference?}; an end \"closed\" is a filled dot, \"open\" a hollow one, {value: v} a filled dot at v with a hollow one where a piece meeting there ends elsewhere",
            has_doc,
            piecewise
        ),
    ]
}

// ---------- shared ----------

/// What every drawing command takes besides its own parameters.
struct Common {
    plan: FramePlan,
    line: Option<LineRef>,
    color: Color,
    style: MathDrawStyle,
    parent: Option<NodeId>,
}

fn common(s: &Session, p: &Value, cmd: &str) -> Result<Common> {
    if !p.is_object() && !p.is_null() {
        return Err(bad(cmd, "the parameters must be an object"));
    }
    let line = line_param(p, "line", cmd)?;
    let color = color_param(p, "color", cmd)?.unwrap_or(Color::BLACK);
    let plan = resolve_frame(s, p, cmd)?;
    let st = s.doc()?;
    Ok(Common { plan, line, color, style: st.doc.math_draw.clone(), parent: st.target_parent()? })
}

/// One undo step labelled `label`: the frame's reference segment (if it makes one), then what
/// `build` draws, inserted on top as one object ([`insert_output`], a group named `name`). A
/// reference segment drawn is added to the selection. Answers what `build` answers with the `id`
/// of what was inserted and the `reference` segment's.
fn draw(
    s: &mut Session,
    c: &Common,
    label: &str,
    name: &str,
    cmd: &str,
    build: impl FnOnce(&mut Document, &Frame) -> Result<(Vec<Node>, Map<String, Value>)>,
) -> Result<Value> {
    let (plan, parent) = (c.plan, c.parent);
    s.edit(label, |d, sel| {
        let reference = plan.apply(d, parent)?;
        let (nodes, mut out) = build(d, &plan.frame)?;
        let id = insert_output(d, parent, name, nodes, cmd)?;
        if let (FrameSource::Artboard, Some(r)) = (plan.source, reference) {
            sel.add(r);
        }
        out.insert("id".into(), json!(id.0));
        if let Some(r) = reference {
            out.insert("reference".into(), json!(r.0));
        }
        Ok(Value::Object(out))
    })
}

fn ids(nodes: &[Node]) -> Value {
    json!(nodes.iter().map(|n| n.id.0).collect::<Vec<_>>())
}

fn pairs(pts: &[Point]) -> Value {
    json!(pts.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>())
}

/// Whether a math font is there to set labels with.
fn have_math_font() -> bool {
    vectorcraft_math::find_math_face(vectorcraft_text::FontDb::global()).is_some()
}

/// An expression of the graph language as LaTeX for a label (`*` as a centred dot).
fn latex(src: &str) -> String {
    src.replace('*', "\\cdot ")
}

/// An expression parameter, read in the variables `vars`.
fn expr_param(p: &Value, key: &str, vars: &[&str], cmd: &str) -> Result<(String, Expr)> {
    let src = text_param(p, key, cmd)?.ok_or_else(|| bad(cmd, format!("missing `{key}`: an expression in {}", vars.join(", "))))?;
    let e = parse_expr(src, vars).map_err(|e| bad(cmd, format!("`{key}`: {e}")))?;
    Ok((src.to_string(), e))
}

/// `labelAt`: only "end" is known.
fn label_at_param(p: &Value, cmd: &str) -> Result<()> {
    match text_param(p, "labelAt", cmd)? {
        None | Some("end") => Ok(()),
        Some(_) => Err(bad(cmd, "`labelAt` must be \"end\"")),
    }
}

/// Graph values beyond what the page can hold (in `frame`) as NaN, which breaks the curve there.
fn on_page(frame: &Frame, pts: &mut [(f64, f64)]) {
    let limit = crate::MAX_COORD / 2.0;
    for (x, y) in pts.iter_mut() {
        let q = frame.to_doc(*x, *y);
        if !(q.x.abs() <= limit && q.y.abs() <= limit) {
            *y = f64::NAN;
        }
    }
}

/// The math bounds of fitted runs' end points.
fn runs_bounds(runs: &[Vec<CubicBez>]) -> Option<Rect> {
    runs.iter().flatten().flat_map(|c| [c.p0, c.p3]).map(|p| Rect::from_points(p, p)).reduce(|a, b| a.union(b))
}

/// Axis ranges covering math bounds `b` and the origin, with a margin.
fn covering(b: Rect) -> ((f64, f64), (f64, f64)) {
    let pad = |lo: f64, hi: f64| {
        let (lo, hi) = (lo.min(0.0), hi.max(0.0));
        let m = if hi - lo > 1e-9 { 0.1 * (hi - lo) } else { 1.0 };
        (lo - m, hi + m)
    };
    (pad(b.x0, b.x1), pad(b.y0, b.y1))
}

/// A dashed guide from math `a` to `b` (none when they are the same point).
fn guide(d: &mut Document, frame: &Frame, a: (f64, f64), b: (f64, f64), style: &MathDrawStyle, line: LineRef, color: Color) -> Option<Node> {
    let (pa, pb) = (frame.to_doc(a.0, a.1), frame.to_doc(b.0, b.1));
    ((pa - pb).hypot() > 1e-9).then(|| stroked(d, shapes::line(pa, pb), style, line, color))
}

/// The guides from math point `at` to both axes.
fn guides_to_axes(d: &mut Document, frame: &Frame, at: (f64, f64), style: &MathDrawStyle, line: LineRef, color: Color) -> Vec<Node> {
    [(at.0, 0.0), (0.0, at.1)].into_iter().filter_map(|to| guide(d, frame, at, to, style, line, color)).collect()
}

// ---------- axes ----------

/// An arrowhead of `kind` at `tip`, pointing along unit vector `dir`, for a line `w` wide: its
/// path, whether it is filled (else stroked), and how far back from the tip the line stops.
fn arrowhead(kind: u8, tip: Point, dir: Vec2, w: f64) -> (PathData, bool, f64) {
    let n = Vec2::new(-dir.y, dir.x);
    let at = |back: f64, side: f64| tip - dir * back + n * side;
    let poly = |pts: &[Point], closed: bool| PathData::single(SubPath::polyline(pts, closed));
    match kind {
        // An open V of two strokes.
        1 => {
            let (l, h) = (8.0 * w, 3.5 * w);
            (poly(&[at(l, h), tip, at(l, -h)], false), false, 0.0)
        }
        // A slim triangle.
        2 => {
            let l = 10.0 * w;
            (poly(&[tip, at(l, 2.5 * w), at(l, -2.5 * w)], true), true, 0.5 * l)
        }
        // A triangle with its back notched in (swallowtail), the textbook arrow.
        _ => {
            let l = 10.0 * w;
            (poly(&[tip, at(l, 4.0 * w), at(0.7 * l, 0.0), at(l, -4.0 * w)], true), true, 0.5 * l)
        }
    }
}

/// The coordinate axes over math ranges `x` and `y`, drawn with `line`: a group named "Axes" of
/// the two axes, their arrowheads and the labels x, y and O (left out without a math font), and
/// what it holds.
fn build_axes(
    d: &mut Document,
    style: &MathDrawStyle,
    frame: &Frame,
    (x, y): ((f64, f64), (f64, f64)),
    line: LineRef,
    color: Color,
    cmd: &str,
) -> Result<(Node, Map<String, Value>)> {
    let (width, _) = style.stroke(line);
    let w = width * style.arrow_ratio;
    let (mut axes, mut arrows, mut labels) = (vec![], vec![], vec![]);
    let ends = [(frame.to_doc(x.0, 0.0), frame.to_doc(x.1, 0.0)), (frame.to_doc(0.0, y.0), frame.to_doc(0.0, y.1))];
    for (from, tip) in ends {
        let len = (tip - from).hypot();
        if len.is_nan() || len <= 1e-9 {
            return Err(bad(cmd, "an axis has no length"));
        }
        let dir = (tip - from) / len;
        let (head, filled, inset) = arrowhead(style.arrow_type, tip, dir, w);
        axes.push(stroked(d, shapes::line(from, tip - dir * inset.min(len)), style, line, color));
        let mut a = shape_node(d, head, None);
        a.appearance = if filled {
            Appearance { items: vec![AppearanceItem::Fill(FillLayer::new(Paint::solid(color)))], ..Appearance::default() }
        } else {
            let mut st = StrokeLayer::new(Paint::solid(color), w);
            st.cap = LineCap::Round;
            st.join = LineJoin::Round;
            Appearance { items: vec![AppearanceItem::Stroke(st)], ..Appearance::default() }
        }
        .into();
        arrows.push(a);
    }
    let mut out = Map::new();
    out.insert("axes".into(), ids(&axes));
    out.insert("arrows".into(), ids(&arrows));
    if have_math_font() {
        let at = [(AxisLabelKind::X, ends[0].1, "x"), (AxisLabelKind::Y, ends[1].1, "y"), (AxisLabelKind::Origin, frame.to_doc(0.0, 0.0), "O")];
        for (kind, anchor, text) in at {
            let source = if style.label(kind).italic { text.to_string() } else { format!("\\mathrm{{{text}}}") };
            // Offsets are y up.
            let (ox, oy) = style.label_offset(kind);
            let mut l = crate::cmd::math::equation_on_baseline(d, &source, style.label_size, anchor + Vec2::new(ox, -oy), color, cmd)?;
            l.name = Some(text.to_string());
            labels.push(l);
        }
        out.insert("labels".into(), ids(&labels));
    } else {
        out.insert("labels".into(), json!(NO_FONT));
    }
    let children: Vec<Node> = axes.into_iter().zip(arrows).flat_map(|(a, b)| [a, b]).chain(labels).collect();
    let mut g = Node::group(d.alloc_id(), children.into_iter().map(std::sync::Arc::new).collect());
    g.name = Some("Axes".into());
    out.insert("id".into(), json!(g.id.0));
    Ok((g, out))
}

/// The math bounds of the selected objects other than the frame's reference segment.
fn selection_bounds(s: &Session, plan: &FramePlan) -> Result<Option<Rect>> {
    let st = s.doc()?;
    let reference = match plan.source {
        FrameSource::Reference(id) => Some(id),
        _ => None,
    };
    let b = st
        .selection
        .objects
        .iter()
        .filter(|id| Some(**id) != reference)
        .filter_map(|id| st.doc.node(*id).and_then(|n| n.geometric_bounds()))
        .reduce(|a, b| a.union(b));
    Ok(b.map(|b| {
        let (u0, v0) = plan.frame.to_math(Point::new(b.x0, b.y1));
        let (u1, v1) = plan.frame.to_math(Point::new(b.x1, b.y0));
        Rect::new(u0, v0, u1, v1)
    }))
}

fn axes(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.axes";
    let c = common(s, p, C)?;
    let (mut x, mut y) = (range_param(p, "xRange", C)?, range_param(p, "yRange", C)?);
    if flag_param(p, "fitSelection", C)?.unwrap_or(false)
        && let Some(b) = selection_bounds(s, &c.plan)?
    {
        // The selection's span, and the origin the axes cross at.
        let span = |lo: f64, hi: f64| {
            let (lo, hi) = (lo.min(0.0), hi.max(0.0));
            if hi - lo > 1e-9 { (lo, hi) } else { (lo - 1.0, hi + 1.0) }
        };
        x = x.or(Some(span(b.x0, b.x1)));
        y = y.or(Some(span(b.y0, b.y1)));
    }
    let ranges = (x.unwrap_or((-2.0, 2.0)), y.unwrap_or((-2.0, 2.0)));
    let line = c.line.unwrap_or(c.style.axis_line);
    draw(s, &c, "Coordinate Axes", "Axes", C, |d, frame| {
        let (g, mut out) = build_axes(d, &c.style, frame, ranges, line, c.color, C)?;
        out.remove("id");
        Ok((vec![g], out))
    })
}

// ---------- points and circles ----------

fn points(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.points";
    let coords = points_param(p, "coords", MAX_ITEMS, C)?.ok_or_else(|| bad(C, "missing `coords`: [[u, v], …]"))?;
    if coords.is_empty() {
        return Err(bad(C, "`coords` has no points"));
    }
    let with_guides = flag_param(p, "guides", C)?.unwrap_or(true);
    let size = number_param(p, "dot", C)?;
    let (lo, hi) = vectorcraft_doc::mathdraw::limits::DOT;
    if size.is_some_and(|d| !(lo..=hi).contains(&d)) {
        return Err(bad(C, format!("`dot` must be a diameter from {lo} to {hi} pt")));
    }
    let mut c = common(s, p, C)?;
    if let Some(size) = size {
        c.style.dot_size = size;
    }
    let line = c.line.unwrap_or(LineRef::Dashed(3));
    draw(s, &c, "Points by Coordinates", "Points", C, |d, frame| {
        let guides: Vec<Node> =
            if with_guides { coords.iter().flat_map(|&at| guides_to_axes(d, frame, at, &c.style, line, c.color)).collect() } else { vec![] };
        let dots: Vec<Node> = coords.iter().map(|&(u, v)| dot(d, frame.to_doc(u, v), &c.style, c.color, Color::WHITE)).collect();
        let mut out = Map::new();
        out.insert("dots".into(), ids(&dots));
        out.insert("guides".into(), ids(&guides));
        Ok((guides.into_iter().chain(dots).collect(), out))
    })
}

fn circles(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.circles";
    let centers = points_param(p, "centers", MAX_ITEMS, C)?.ok_or_else(|| bad(C, "missing `centers`: [[u, v], …]"))?;
    if centers.is_empty() {
        return Err(bad(C, "`centers` has no points"));
    }
    let r = number_param(p, "radius", C)?.ok_or_else(|| bad(C, "missing `radius` (math units)"))?;
    if !(r > 0.0 && r <= MAX_MATH) {
        return Err(bad(C, format!("`radius` must be above 0 and at most {MAX_MATH}")));
    }
    let c = common(s, p, C)?;
    let line = c.line.unwrap_or(LineRef::Solid(3));
    draw(s, &c, "Circles by Center", "Circles", C, |d, frame| {
        let nodes: Vec<Node> = centers
            .iter()
            .map(|&(u, v)| {
                let (a, b) = (frame.to_doc(u - r, v + r), frame.to_doc(u + r, v - r));
                stroked(d, shapes::ellipse(Rect::from_points(a, b)), &c.style, line, c.color)
            })
            .collect();
        let mut out = Map::new();
        out.insert("circles".into(), ids(&nodes));
        Ok((nodes, out))
    })
}

// ---------- graphs of expressions ----------

/// What the graph and parametric commands share: precision, axes and label.
struct CurveOptions {
    precision: usize,
    axes: bool,
    label: bool,
    /// The x range the axes cover (else the curve's).
    x_span: Option<(f64, f64)>,
}

fn curve_options(p: &Value, cmd: &str) -> Result<CurveOptions> {
    label_at_param(p, cmd)?;
    Ok(CurveOptions {
        precision: count_param(p, "precision", 20, 1..=MAX_PRECISION, cmd)?,
        axes: flag_param(p, "axes", cmd)?.unwrap_or(false),
        label: flag_param(p, "label", cmd)?.unwrap_or(false),
        x_span: None,
    })
}

/// A label `source` set at the label size beside doc point `end` (to its right, a little below),
/// or why there is none.
fn end_label(d: &mut Document, style: &MathDrawStyle, end: Point, source: &str, color: Color, cmd: &str) -> std::result::Result<Node, String> {
    if !have_math_font() {
        return Err(NO_FONT.into());
    }
    let size = style.label_size;
    let origin = end + Vec2::new(0.25 * size, 0.25 * size);
    crate::cmd::math::equation_on_baseline(d, source, size, origin, color, cmd).map_err(|e| format!("skipped ({e})"))
}

/// The drawing of a fitted curve: axes covering it (if asked), the curve, its label (if asked).
fn curve_drawing(
    d: &mut Document,
    c: &Common,
    frame: &Frame,
    runs: &[Vec<CubicBez>],
    opt: &CurveOptions,
    label: &str,
    cmd: &str,
) -> Result<(Vec<Node>, Map<String, Value>)> {
    let mut nodes = vec![];
    let mut out = Map::new();
    if opt.axes {
        let b = runs_bounds(runs).unwrap_or(Rect::ZERO);
        let b = opt.x_span.map_or(b, |(a, z)| Rect::new(a, b.y0, z, b.y1));
        let (g, info) = build_axes(d, &c.style, frame, covering(b), c.style.axis_line, c.color, cmd)?;
        out.insert("axes".into(), Value::Object(info));
        nodes.push(g);
    }
    let curve = stroked(d, curve_path(runs, frame), &c.style, c.line.unwrap_or(LineRef::Solid(3)), c.color);
    out.insert("curve".into(), json!(curve.id.0));
    nodes.push(curve);
    if opt.label
        && let Some(end) = runs.last().and_then(|r| r.last()).map(|c| c.p3)
    {
        match end_label(d, &c.style, frame.to_doc(end.x, end.y), label, c.color, cmd) {
            Ok(l) => {
                out.insert("label".into(), json!(l.id.0));
                nodes.push(l);
            }
            Err(why) => {
                out.insert("label".into(), json!(why));
            }
        }
    }
    Ok((nodes, out))
}

fn clip_param(p: &Value, cmd: &str) -> Result<(f64, f64)> {
    Ok(range_param(p, "yClip", cmd)?.unwrap_or(DEFAULT_CLIP))
}

fn graph(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.graph";
    let (src, f) = expr_param(p, "f", &["x"], C)?;
    let range = range_param(p, "xRange", C)?.ok_or_else(|| bad(C, "missing `xRange`: [a, b]"))?;
    let opt = CurveOptions { x_span: Some(range), ..curve_options(p, C)? };
    let clip = clip_param(p, C)?;
    let c = common(s, p, C)?;
    let runs = fit_function(&f, "x", range, opt.precision, FitOptions { clip: Some(clip) });
    if runs.is_empty() {
        return Err(bad(C, "the function has no values in `xRange` within `yClip`"));
    }
    let label = format!("y={}", latex(&src));
    draw(s, &c, "Graph of a Function", "Graph", C, |d, frame| curve_drawing(d, &c, frame, &runs, &opt, &label, C))
}

fn parametric(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.parametric";
    let (xs, x) = expr_param(p, "x", &["t"], C)?;
    let (ys, y) = expr_param(p, "y", &["t"], C)?;
    let range = range_param(p, "tRange", C)?.ok_or_else(|| bad(C, "missing `tRange`: [a, b]"))?;
    let opt = curve_options(p, C)?;
    let c = common(s, p, C)?;
    let runs = fit_parametric(&x, &y, "t", range, opt.precision);
    if runs.is_empty() {
        return Err(bad(C, "the curve has no points in `tRange`"));
    }
    let label = format!("(x,y)=\\left({},{}\\right)", latex(&xs), latex(&ys));
    draw(s, &c, "Parametric Curve", "Parametric Curve", C, |d, frame| curve_drawing(d, &c, frame, &runs, &opt, &label, C))
}

// ---------- piecewise ----------

/// How a piece of a piecewise graph ends.
#[derive(Clone, Copy, Debug, PartialEq)]
enum End {
    /// A hollow dot: the end isn't the function's.
    Open,
    /// A filled dot.
    Closed,
    /// A filled dot at this value, with hollow ones where pieces ending here end elsewhere.
    Value(f64),
}

fn end_value(v: &Value, key: &str, cmd: &str) -> Result<Option<End>> {
    let err = || bad(cmd, format!("`{key}` must be \"open\", \"closed\" or {{value: number}}"));
    match v {
        Value::Null => Ok(None),
        Value::String(s) if s == "open" => Ok(Some(End::Open)),
        Value::String(s) if s == "closed" => Ok(Some(End::Closed)),
        Value::Object(o) => match (o.len(), o.get("value").and_then(Value::as_f64)) {
            (1, Some(v)) if v.is_finite() && v.abs() <= MAX_MATH => Ok(Some(End::Value(v))),
            _ => Err(err()),
        },
        _ => Err(err()),
    }
}

struct Piece {
    f: Expr,
    range: (f64, f64),
    precision: usize,
    /// The left and right ends.
    ends: [Option<End>; 2],
}

fn piece_value(v: &Value, i: usize, cmd: &str) -> Result<Piece> {
    let o = v.as_object().ok_or_else(|| bad(cmd, format!("`pieces[{i}]` must be {{f, range, …}}")))?;
    if let Some(k) = o.keys().find(|k| !["f", "range", "precision", "leftEnd", "rightEnd"].contains(&k.as_str())) {
        return Err(bad(cmd, format!("`pieces[{i}]` has an unknown key `{k}` (it takes f, range, precision, leftEnd, rightEnd)")));
    }
    let (_, f) = expr_param(v, "f", &["x"], cmd).map_err(|e| match e {
        EngineError::BadParams { msg, .. } => bad(cmd, format!("pieces[{i}]: {msg}")),
        e => e,
    })?;
    let range = range_param(v, "range", cmd)?.ok_or_else(|| bad(cmd, format!("`pieces[{i}]` is missing `range`: [a, b]")))?;
    let precision = count_param(v, "precision", 20, 1..=MAX_PRECISION, cmd)?;
    let end = |k: &str| o.get(k).map_or(Ok(None), |e| end_value(e, k, cmd));
    Ok(Piece { f, range, precision, ends: [end("leftEnd")?, end("rightEnd")?] })
}

/// `f`'s value at `x`, else its limit there from inside the piece (`inward` +1: from the right).
fn limit(f: &Expr, x: f64, inward: f64, span: f64) -> f64 {
    [0.0, 1e-9, 1e-7, 1e-5].iter().map(|k| f.eval1(x + inward * k * span)).find(|y| y.is_finite()).unwrap_or(f64::NAN)
}

/// The dots of a piecewise graph: math points, filled or hollow.
fn piecewise_dots(pieces: &[Piece]) -> Vec<(Point, bool)> {
    let mut dots: Vec<(Point, bool)> = vec![];
    let near = |a: f64, b: f64| (a - b).abs() <= 1e-9 * (1.0 + a.abs().max(b.abs()));
    let mut add = |at: Point, filled: bool| {
        if !(at.x.is_finite() && at.y.is_finite()) {
            return;
        }
        match dots.iter_mut().find(|(q, _)| near(q.x, at.x) && near(q.y, at.y)) {
            // A filled dot wins over a hollow one at the same place.
            Some(d) => d.1 |= filled,
            None => dots.push((at, filled)),
        }
    };
    // Each end: its x, the piece's limit there.
    let ends = |p: &Piece| {
        let span = p.range.1 - p.range.0;
        [(p.range.0, limit(&p.f, p.range.0, 1.0, span)), (p.range.1, limit(&p.f, p.range.1, -1.0, span))]
    };
    for p in pieces {
        for ((x, l), end) in ends(p).into_iter().zip(p.ends) {
            match end {
                None => {}
                Some(End::Open) => add(Point::new(x, l), false),
                Some(End::Closed) => add(Point::new(x, l), true),
                Some(End::Value(v)) => {
                    add(Point::new(x, v), true);
                    if !near(l, v) {
                        add(Point::new(x, l), false);
                    }
                    // The other pieces ending here without an end of their own.
                    for q in pieces {
                        for ((qx, ql), qend) in ends(q).into_iter().zip(q.ends) {
                            if qend.is_none() && near(qx, x) && !near(ql, v) {
                                add(Point::new(qx, ql), false);
                            }
                        }
                    }
                }
            }
        }
    }
    dots
}

fn piecewise(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.piecewise";
    let list = p.get("pieces").filter(|v| !v.is_null()).ok_or_else(|| bad(C, "missing `pieces`: [{f, range, …}, …]"))?;
    let list = list
        .as_array()
        .filter(|a| !a.is_empty() && a.len() <= MAX_PIECES)
        .ok_or_else(|| bad(C, format!("`pieces` must be a list of 1 to {MAX_PIECES} pieces")))?;
    let pieces = list.iter().enumerate().map(|(i, v)| piece_value(v, i, C)).collect::<Result<Vec<_>>>()?;
    let with_axes = flag_param(p, "axes", C)?.unwrap_or(false);
    let clip = clip_param(p, C)?;
    let c = common(s, p, C)?;
    let mut all_runs = vec![];
    for (i, piece) in pieces.iter().enumerate() {
        let runs = fit_function(&piece.f, "x", piece.range, piece.precision, FitOptions { clip: Some(clip) });
        if runs.is_empty() {
            return Err(bad(C, format!("pieces[{i}] has no values in its range within `yClip`")));
        }
        all_runs.push(runs);
    }
    let dots = piecewise_dots(&pieces);
    draw(s, &c, "Piecewise Function Graph", "Piecewise Graph", C, |d, frame| {
        let mut nodes = vec![];
        let mut out = Map::new();
        if with_axes {
            let b =
                all_runs.iter().filter_map(|r| runs_bounds(r)).chain(dots.iter().map(|(q, _)| Rect::from_points(*q, *q))).reduce(|a, b| a.union(b));
            let (g, info) = build_axes(d, &c.style, frame, covering(b.unwrap_or(Rect::ZERO)), c.style.axis_line, c.color, C)?;
            out.insert("axes".into(), Value::Object(info));
            nodes.push(g);
        }
        let line = c.line.unwrap_or(LineRef::Solid(3));
        let curves: Vec<Node> = all_runs.iter().map(|runs| stroked(d, curve_path(runs, frame), &c.style, line, c.color)).collect();
        let dot_nodes: Vec<Node> = dots
            .iter()
            .map(|&(q, filled)| {
                let at = frame.to_doc(q.x, q.y);
                if filled { dot(d, at, &c.style, c.color, Color::WHITE) } else { dot(d, at, &c.style, Color::WHITE, c.color) }
            })
            .collect();
        let (closed, open): (Vec<_>, Vec<_>) = dots.iter().partition(|(_, filled)| *filled);
        out.insert("curves".into(), ids(&curves));
        out.insert("dots".into(), ids(&dot_nodes));
        out.insert("closed".into(), pairs(&closed.iter().map(|(q, _)| *q).collect::<Vec<_>>()));
        out.insert("open".into(), pairs(&open.iter().map(|(q, _)| *q).collect::<Vec<_>>()));
        nodes.extend(curves);
        nodes.extend(dot_nodes);
        Ok((nodes, out))
    })
}

// ---------- graphs read from curves ----------

/// Whether `n` can be a curve to read: an open, stroked path or compound path.
fn is_curve(n: &Node) -> bool {
    let open = match &n.kind {
        NodeKind::Path { path, .. } => path.subpaths.iter().any(|sp| !sp.closed && sp.anchors.len() >= 2),
        NodeKind::Compound { children, .. } => {
            children.iter().filter_map(|c| c.path_data()).flat_map(|p| &p.subpaths).any(|sp| !sp.closed && sp.anchors.len() >= 2)
        }
        _ => false,
    };
    open && n.appearance.stroke().is_some_and(|s| s.visible)
}

/// The curves in the selection, in order: selected curves, and those in selected groups (but not
/// in the axes of the math drawing commands, a group named "Axes"), not the frame's reference.
fn selected_curves(s: &Session, plan: &FramePlan) -> Result<Vec<NodeId>> {
    fn walk(n: &Node, out: &mut Vec<NodeId>) {
        if is_curve(n) {
            out.push(n.id);
            return;
        }
        if let NodeKind::Group { children, .. } = &n.kind
            && n.name.as_deref() != Some("Axes")
        {
            for c in children {
                walk(c, out);
            }
        }
    }
    let st = s.doc()?;
    let reference = match plan.source {
        FrameSource::Reference(id) => Some(id),
        _ => None,
    };
    let mut out = vec![];
    for id in &st.selection.objects {
        if let Some(n) = st.doc.node(*id) {
            walk(n, &mut out);
        }
    }
    out.retain(|id| Some(*id) != reference && st.doc.node(*id).is_some_and(|n| !super::is_magenta(n)));
    out.dedup();
    Ok(out)
}

/// A curve given by id (`key` of `p`, checked) or else the first selected one that isn't a
/// straight segment (a straight one if that is all there is), read in `frame`.
fn target_function(s: &Session, p: &Value, key: &str, c: &Common, cmd: &str) -> Result<PathFunction> {
    let st = s.doc()?;
    let id = match p.get(key).filter(|v| !v.is_null()) {
        Some(v) => checked_id(s, v, cmd)?,
        None => {
            let curves = selected_curves(s, &c.plan)?;
            let straight = |id: &NodeId| st.doc.node(*id).is_some_and(|n| super::straight_segment(n).is_some());
            curves
                .iter()
                .find(|&&id| !straight(&id))
                .or(curves.first())
                .copied()
                .ok_or_else(|| bad(cmd, format!("select a curve (or give its `{key}`)")))?
        }
    };
    let n = st.doc.node(id).ok_or(EngineError::NoNode(id))?;
    PathFunction::from_node(n, &c.plan.frame).ok_or_else(|| bad(cmd, format!("object {} is not a curve that can be read as y(x)", id.0)))
}

fn extrema(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.extrema";
    let with_guides = flag_param(p, "guides", C)?.unwrap_or(false);
    let c = common(s, p, C)?;
    let f = target_function(s, p, "id", &c, C)?;
    let found = f.extrema();
    if found.is_empty() {
        return Err(bad(C, "the curve has no local maximum or minimum"));
    }
    let line = c.line.unwrap_or(LineRef::Dashed(3));
    draw(s, &c, "Mark Maxima and Minima", "Extrema", C, |d, frame| {
        let guides: Vec<Node> = if with_guides {
            found.iter().flat_map(|(q, _)| guides_to_axes(d, frame, (q.x, q.y), &c.style, line, c.color)).collect()
        } else {
            vec![]
        };
        let dots: Vec<Node> = found.iter().map(|(q, _)| dot(d, frame.to_doc(q.x, q.y), &c.style, c.color, Color::WHITE)).collect();
        let of = |k: Extremum| found.iter().filter(|(_, e)| *e == k).map(|(q, _)| *q).collect::<Vec<_>>();
        let mut out = Map::new();
        out.insert("maxima".into(), pairs(&of(Extremum::Max)));
        out.insert("minima".into(), pairs(&of(Extremum::Min)));
        out.insert("dots".into(), ids(&dots));
        out.insert("guides".into(), ids(&guides));
        Ok((guides.into_iter().chain(dots).collect(), out))
    })
}

/// `n` evenly spaced xs over `(a, b)`.
fn spaced(n: usize, (a, b): (f64, f64)) -> impl Iterator<Item = f64> {
    let n = n.max(2);
    (0..n).map(move |i| if i + 1 == n { b } else { a + (b - a) * i as f64 / (n - 1) as f64 })
}

/// Draw a graph given by samples (math coordinates) as one curve.
fn sampled_graph(s: &mut Session, c: &Common, mut pts: Vec<(f64, f64)>, label: &str, cmd: &str, extra: Map<String, Value>) -> Result<Value> {
    on_page(&c.plan.frame, &mut pts);
    let runs = fit_samples(&pts);
    if runs.is_empty() {
        return Err(bad(cmd, "the graph has no values to draw"));
    }
    let line = c.line.unwrap_or(LineRef::Solid(3));
    draw(s, c, label, label, cmd, |d, frame| Ok((vec![stroked(d, curve_path(&runs, frame), &c.style, line, c.color)], extra)))
}

/// The integral of `f` over `(a, b)`, 0 at `a`, at `n` points: Simpson's rule between them.
fn integral_samples(f: &PathFunction, n: usize, (a, b): (f64, f64)) -> Vec<(f64, f64)> {
    const SUB: usize = 8;
    let xs: Vec<f64> = spaced(n, (a, b)).collect();
    let mut out = Vec::with_capacity(xs.len());
    let mut acc = 0.0;
    for (i, &x) in xs.iter().enumerate() {
        let Some(prev) = i.checked_sub(1).and_then(|j| xs.get(j)) else {
            out.push((x, if f.at(x).is_finite() { 0.0 } else { f64::NAN }));
            continue;
        };
        let h = (x - prev) / SUB as f64;
        let part: f64 = (0..=SUB)
            .map(|k| {
                f.at(prev + h * k as f64)
                    * if k == 0 || k == SUB {
                        1.0
                    } else if k % 2 == 1 {
                        4.0
                    } else {
                        2.0
                    }
            })
            .sum::<f64>()
            * h
            / 3.0;
        // Where the curve has a gap, the integral has one too, and goes on from where it was.
        if part.is_finite() {
            acc += part;
            out.push((x, acc));
        } else {
            out.push((x, f64::NAN));
        }
    }
    out
}

/// The derivative of `f` at `n` points over `(a, b)`: central differences (one-sided at the ends).
fn derivative_samples(f: &PathFunction, n: usize, (a, b): (f64, f64)) -> Vec<(f64, f64)> {
    let h = (b - a) * 1e-4;
    spaced(n, (a, b))
        .map(|x| {
            let (x0, x1) = ((x - h).max(a), (x + h).min(b));
            (x, (f.at(x1) - f.at(x0)) / (x1 - x0))
        })
        .collect()
}

fn integral(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.integral";
    let n = count_param(p, "precision", SAMPLES.0, SAMPLES.1..=SAMPLES.2, C)?;
    let right = match text_param(p, "c0At", C)? {
        None | Some("left") => false,
        Some("right") => true,
        Some(_) => return Err(bad(C, "`c0At` must be \"left\" or \"right\"")),
    };
    let c = common(s, p, C)?;
    let f = target_function(s, p, "id", &c, C)?;
    let mut pts = integral_samples(&f, n, f.range());
    if right && let Some(last) = pts.iter().rev().map(|q| q.1).find(|y| y.is_finite()) {
        pts.iter_mut().for_each(|q| q.1 -= last);
    }
    sampled_graph(s, &c, pts, "Integral Graph", C, Map::new())
}

fn derivative(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.derivative";
    let n = count_param(p, "precision", SAMPLES.0, SAMPLES.1..=SAMPLES.2, C)?;
    let c = common(s, p, C)?;
    let f = target_function(s, p, "id", &c, C)?;
    let pts = derivative_samples(&f, n, f.range());
    sampled_graph(s, &c, pts, "Derivative Graph", C, Map::new())
}

/// The colour role of a curve for graph arithmetic: f black, g blue (or cyan), h yellow.
fn color_role(n: &Node) -> Option<&'static str> {
    let s = n.appearance.stroke().filter(|s| s.visible)?;
    let Paint::Solid { color, .. } = &s.paint else { return None };
    let [r, g, b] = color.to_rgb_uncalibrated();
    if r.max(g).max(b) < 0.2 {
        Some("f")
    } else if r < 0.3 && b > 0.7 {
        Some("g")
    } else if r > 0.8 && g > 0.7 && b < 0.3 {
        Some("h")
    } else {
        None
    }
}

fn combine(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.combine";
    const VARS: [&str; 3] = ["f", "g", "h"];
    let (_, e) = expr_param(p, "expr", &["f", "g", "h", "x"], C)?;
    let used: Vec<&str> = VARS.into_iter().filter(|v| e.uses(v)).collect();
    if used.is_empty() {
        return Err(bad(C, "`expr` must use f, g or h"));
    }
    let n = count_param(p, "precision", SAMPLES.0, SAMPLES.1..=SAMPLES.2, C)?;
    let c = common(s, p, C)?;
    let mut chosen: Vec<(&str, NodeId)> = vec![];
    match p.get("ids").filter(|v| !v.is_null()) {
        Some(v) => {
            let o = v.as_object().ok_or_else(|| bad(C, "`ids` must be {f?, g?, h?: curve ids}"))?;
            for (k, v) in o {
                let role = VARS.into_iter().find(|r| r == k).ok_or_else(|| bad(C, format!("`ids` has an unknown key `{k}` (f, g or h)")))?;
                chosen.push((role, checked_id(s, v, C)?));
            }
        }
        None => {
            let st = s.doc()?;
            for id in selected_curves(s, &c.plan)? {
                if let Some(role) = st.doc.node(id).and_then(color_role)
                    && !chosen.iter().any(|(r, _)| *r == role)
                {
                    chosen.push((role, id));
                }
            }
        }
    }
    let mut fs: Vec<(&str, PathFunction)> = vec![];
    for v in &used {
        let (_, id) = chosen
            .iter()
            .find(|(r, _)| r == v)
            .ok_or_else(|| bad(C, format!("there is no curve {v}: select a black (f), blue (g) or yellow (h) curve, or give `ids`")))?;
        let n = s.doc()?.doc.node(*id).ok_or(EngineError::NoNode(*id))?;
        let f = PathFunction::from_node(n, &c.plan.frame)
            .ok_or_else(|| bad(C, format!("object {} ({v}) is not a curve that can be read as y(x)", id.0)))?;
        fs.push((v, f));
    }
    let (a, b) = fs.iter().map(|(_, f)| f.range()).fold((f64::NEG_INFINITY, f64::INFINITY), |(a, b), (lo, hi)| (a.max(lo), b.min(hi)));
    if a.is_nan() || b.is_nan() || a >= b {
        return Err(bad(C, "the curves have no x range in common"));
    }
    let pts: Vec<(f64, f64)> = spaced(n, (a, b))
        .map(|x| {
            let mut env: Vec<(&str, f64)> = fs.iter().map(|(v, f)| (*v, f.at(x))).collect();
            env.push(("x", x));
            (x, e.eval(&env))
        })
        .collect();
    let mut extra = Map::new();
    extra.insert("range".into(), json!([a, b]));
    sampled_graph(s, &c, pts, "Graph Arithmetic", C, extra)
}
