//! Space coordinates (3D): axes, points with their guides, parametric curves and circles in space,
//! drawn in a parallel projection of space onto the page ([`Frame3`]).
//!
//! The projection comes from the command's `origin`, `ex`, `ey` and `ez` (the page vectors of one
//! math unit along each axis), else from the **3D reference object** in the selection (a group of
//! three straight segments stroked cyan, magenta and yellow meeting at one point: that point is
//! the origin, and each segment, from there to its other end, one unit along x, y and z), else
//! from the centre of the artboard with the style's standard length as the unit and the textbook
//! view (x toward the lower left, y to the right, z up), where a reference object is drawn too
//! ([`resolve_frame3`]). Parameters given alongside a reference object override its parts.
//!
//! Each command is one undo step, draws its parts as one group, and answers the ids of what it
//! drew: `id` (the group, or the one object), per part, and `reference` (the reference object it
//! drew in or made). A reference object it made is added to the selection, so the next command
//! draws in the same projection.

use serde_json::{Map, Value, json};
use vectorcraft_color::Color;
use vectorcraft_doc::mathdraw::AxisLabelKind;
use vectorcraft_doc::{ColorMode, Document, LineRef, MathDrawStyle, Node, NodeId, NodeKind};
use vectorcraft_geom::{BezPath, CubicBez, PathData, Point, Vec2, shapes};
use vectorcraft_math::fit::fit_parametric_with;
use vectorcraft_math::{Expr, fit_samples};

use super::graph::{NO_FONT, axis_label, axis_with_arrow, expr_param, have_math_font, ids, label_at};
use super::*;

/// Most points `mathdraw.points3d` takes.
const MAX_ITEMS: usize = 10_000;
/// Most steps of a curve (its precision).
const MAX_PRECISION: usize = 2000;
/// Steps of a circle: a projected circle (an ellipse) is all but exact in 16 cubics.
const CIRCLE_STEPS: usize = 16;
/// The default view: one unit of x on the page, in units of the standard length (y down).
const DEFAULT_EX: (f64, f64) = (-0.64, 0.77);
/// The name of a reference object a command makes.
const REFERENCE_NAME: &str = "Space Reference";

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "mathdraw.axes3d",
            "Space Coordinate Axes",
            [],
            None,
            "{lengths?: [lx, ly, lz] math units each above 0 (default [2, 2, 2]: how far each axis reaches on its positive side), negative?: [nx, ny, nz] math units each 0 or more (default [0, 0, 0]: how far it reaches on the negative side), line?: line (default the style's axis line), color?: \"#rrggbb\", origin?: [x, y] pt, ex?|ey?|ez?: [dx, dy] pt (one unit along x, y, z on the page), unit?: pt per math unit of the default view, makeReference?: bool (default true), artboard?: index} → {id, axes: [x, y, z], arrows: [x, y, z], labels: [x, y, z, O] ids or \"skipped (no math font)\", reference?}; the x, y and z axes with arrowheads, and the labels x, y, z, O set in the math font at the style's label size: an axis nearer the page's horizontal takes the style's x label offset (under its tip), one nearer the vertical the y label offset (left of its tip), and O sits in the widest gap between the axes at the distance the style's O offset gives",
            has_doc,
            axes3d
        ),
        cmd!(
            "mathdraw.points3d",
            "Points by Space Coordinates",
            [],
            None,
            "{coords: [[x, y, z], …] math coordinates, guides?: bool (default true: dashed lines from each point down to the xy plane and from there to the x and y axes), planeGuides?: bool (default false: also the dashed lines to the yz and zx planes and on to the axes, the edges of the box from the origin to the point; implies guides), dot?: diameter pt (default the style's dot size), line?: line of the guides (default dashed3), color?: \"#rrggbb\", origin?, ex?|ey?|ez?, unit?, makeReference?, artboard?} → {id, dots: [ids], guides: [ids], reference?}",
            has_doc,
            points3d
        ),
        cmd!(
            "mathdraw.parametric3d",
            "Space Parametric Curve",
            [],
            None,
            "{x: expression in t, y: expression in t, z: expression in t, tRange: [a, b], precision?: steps 1–2000 (default 40), line?: line (default solid3), color?: \"#rrggbb\", origin?, ex?|ey?|ez?, unit?, makeReference?, artboard?} → {id, curve, reference?}; the curve (x(t), y(t), z(t)) in the projection",
            has_doc,
            parametric3d
        ),
        cmd!(
            "mathdraw.circle3d",
            "Space Circle",
            [],
            None,
            "{center: [x, y, z], normal: [x, y, z] (the circle's plane is perpendicular to it), radius: math units, shadow?: {planeNormal: [x, y, z], planePoint?: [x, y, z] (default the origin), lightDir?: [x, y, z] (default planeNormal)} (also draw the circle's shadow cast by parallel light along lightDir onto that plane), shadowLine?: line of the shadow (default dashed3), line?: line (default solid3), color?: \"#rrggbb\", origin?, ex?|ey?|ez?, unit?, makeReference?, artboard?} → {id, circle, shadow?, reference?}; the circle in the projection (an ellipse)",
            has_doc,
            circle3d
        ),
    ]
}

// ---------- vectors in space ----------

/// A math vector or point in space.
type V3 = [f64; 3];

fn dot3(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn add3(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(a: V3, k: f64) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn norm3(a: V3) -> f64 {
    dot3(a, a).sqrt()
}

/// `a` scaled to length 1, `None` for a vector too short (or not finite) to have a direction.
fn unit3(a: V3) -> Option<V3> {
    let n = norm3(a);
    (n.is_finite() && n > 1e-12).then(|| scale3(a, 1.0 / n))
}

/// Two unit vectors that, with unit vector `n`, make a right-handed orthonormal basis.
fn plane_basis(n: V3) -> (V3, V3) {
    // The axis least along `n`, made perpendicular to it.
    let a = if n[0].abs() <= n[1].abs() && n[0].abs() <= n[2].abs() {
        [1.0, 0.0, 0.0]
    } else if n[1].abs() <= n[2].abs() {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let u = unit3(cross3(a, n)).unwrap_or([1.0, 0.0, 0.0]);
    (u, cross3(n, u))
}

// ---------- the projection ----------

/// A parallel projection of space onto the page: math `(x, y, z)` sits at
/// `origin + x·ex + y·ey + z·ez` in the document.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Frame3 {
    /// Where math (0, 0, 0) is, in document coordinates.
    pub origin: Point,
    /// One math unit along x, y and z on the page.
    pub ex: Vec2,
    pub ey: Vec2,
    pub ez: Vec2,
}

impl Frame3 {
    /// The document point of math point `p`.
    pub fn to_doc(self, p: V3) -> Point {
        self.origin + self.vec(p)
    }

    /// The page vector of math vector `v`.
    pub fn vec(self, [x, y, z]: V3) -> Vec2 {
        self.ex * x + self.ey * y + self.ez * z
    }

    /// The page vectors of the three axes.
    fn axes(self) -> [Vec2; 3] {
        [self.ex, self.ey, self.ez]
    }

    /// Why the projection can't be drawn in: an axis with no length (or too long), or all three
    /// along one line on the page.
    fn check(self, cmd: &str) -> Result<()> {
        if !(self.origin.x.abs() <= crate::MAX_COORD && self.origin.y.abs() <= crate::MAX_COORD) {
            return Err(bad(cmd, "the origin must be on the canvas"));
        }
        for (v, name) in self.axes().into_iter().zip(["x", "y", "z"]) {
            let len = v.hypot();
            if !(len.is_finite() && len > 1e-6 && len <= MAX_UNIT) {
                return Err(bad(cmd, format!("one unit of {name} must be longer than 0 and at most {MAX_UNIT} pt on the page")));
            }
        }
        let [a, b, c] = self.axes();
        let flat = |u: Vec2, v: Vec2| u.cross(v).abs() <= 1e-6 * u.hypot() * v.hypot();
        if flat(a, b) && flat(b, c) && flat(a, c) {
            return Err(bad(cmd, "the x, y and z axes lie along one line on the page: there is no projection to draw in"));
        }
        Ok(())
    }
}

/// Where a projection came from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Frame3Source {
    /// The command's `origin`, `ex`, `ey` and `ez`.
    Params,
    /// The reference object in the selection.
    Reference(NodeId),
    /// The artboard's centre, the standard length and the default view.
    Artboard,
}

/// A resolved projection, and what using it does to the document ([`Frame3Plan::apply`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Frame3Plan {
    pub frame: Frame3,
    pub source: Frame3Source,
    /// Draw a reference object for a projection made from the artboard.
    make_reference: bool,
}

impl Frame3Plan {
    /// Inside the command's edit: draw the reference object of a projection made from the
    /// artboard (in `parent`, on top). Returns the reference object, if there is one.
    fn apply(&self, d: &mut Document, parent: Option<NodeId>) -> Result<Option<NodeId>> {
        match self.source {
            Frame3Source::Reference(id) => Ok(Some(id)),
            Frame3Source::Params => Ok(None),
            Frame3Source::Artboard if !self.make_reference => Ok(None),
            Frame3Source::Artboard => {
                let style = d.math_draw.clone();
                let o = self.frame.origin;
                let segments: Vec<std::sync::Arc<Node>> = self
                    .frame
                    .axes()
                    .into_iter()
                    .zip(AXIS_COLORS)
                    .map(|(v, rgb)| std::sync::Arc::new(stroked(d, shapes::line(o, o + v), &style, LineRef::Solid(4), color_in(d, rgb))))
                    .collect();
                let mut g = Node::group(d.alloc_id(), segments);
                g.name = Some(REFERENCE_NAME.into());
                Ok(Some(d.insert(parent, usize::MAX, g)?))
            }
        }
    }
}

/// The colours of the reference object's x, y and z segments: cyan, magenta, yellow (RGB).
const AXIS_COLORS: [[f32; 3]; 3] = [[0.0, 1.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 0.0]];

/// A process colour of [`AXIS_COLORS`] in the document's colour mode (CMYK 100% of one ink).
fn color_in(d: &Document, [r, g, b]: [f32; 3]) -> Color {
    match d.color_mode {
        ColorMode::Cmyk => Color::cmyk(1.0 - r, 1.0 - g, 1.0 - b, 0.0),
        ColorMode::Rgb => Color::rgb(r, g, b),
    }
}

/// Which reference axis (0 x cyan, 1 y magenta, 2 z yellow) `n`'s stroke marks, if any.
fn axis_of(n: &Node) -> Option<usize> {
    let s = n.appearance.stroke().filter(|s| s.visible)?;
    let vectorcraft_color::Paint::Solid { color, .. } = &s.paint else { return None };
    let rgb = color.to_rgb_uncalibrated();
    AXIS_COLORS.iter().position(|c| c.iter().zip(rgb).all(|(a, b)| (a - b).abs() < 0.02))
}

/// The projection a reference object's segments give (`(start, end, axis)` each): the point all
/// three have an end at is the origin, and each segment's other end one unit along its axis.
/// `None` when they aren't one segment of each colour meeting at one point.
fn reference_frame(segments: &[(Point, Point, Option<usize>)]) -> Option<Frame3> {
    if segments.len() != 3 {
        return None;
    }
    let of = |axis: usize| {
        let mut it = segments.iter().filter(|s| s.2 == Some(axis));
        let s = it.next()?;
        it.next().is_none().then_some((s.0, s.1))
    };
    let (x, y, z) = (of(0)?, of(1)?, of(2)?);
    let shortest = [x, y, z].iter().map(|(a, b)| (*b - *a).hypot()).fold(f64::INFINITY, f64::min);
    // Ends this near (1% of the shortest segment) meet.
    let tol = 0.01 * shortest;
    // The end of `s` at `p` (and its other end), if it has one there.
    let meets = |s: (Point, Point), p: Point| {
        let (da, db) = ((s.0 - p).hypot(), (s.1 - p).hypot());
        if da <= tol && da <= db {
            Some((s.0, s.1))
        } else if db <= tol {
            Some((s.1, s.0))
        } else {
            None
        }
    };
    [(x.0, x.1), (x.1, x.0)].into_iter().find_map(|(at, far_x)| {
        let (oy, far_y) = meets(y, at)?;
        let (oz, far_z) = meets(z, at)?;
        let origin = Point::new((at.x + oy.x + oz.x) / 3.0, (at.y + oy.y + oz.y) / 3.0);
        Some(Frame3 { origin, ex: far_x - origin, ey: far_y - origin, ez: far_z - origin })
    })
}

/// An optional page vector parameter `[dx, dy]`.
fn page_vector_param(p: &Value, key: &str, cmd: &str) -> Result<Option<Vec2>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => finite_numbers::<2>(v)
            .filter(|[x, y]| x.abs() <= MAX_UNIT && y.abs() <= MAX_UNIT)
            .map(|[x, y]| Some(Vec2::new(x, y)))
            .ok_or_else(|| bad(cmd, format!("`{key}` must be [dx, dy], one unit on the page, within ±{MAX_UNIT} pt"))),
    }
}

/// The projection a command draws in (see the module docs): `origin`, `ex`, `ey`, `ez` win over
/// the selection's reference object, which wins over the artboard (`artboard`: index, default the
/// first) with `unit` (default the style's standard length) and the default view;
/// `makeReference: false` draws no reference object for the last.
pub(crate) fn resolve_frame3(s: &Session, p: &Value, cmd: &str) -> Result<Frame3Plan> {
    let origin = match p.get("origin").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(
            finite_numbers::<2>(v)
                .filter(|[x, y]| x.abs() <= crate::MAX_COORD && y.abs() <= crate::MAX_COORD)
                .map(|[x, y]| Point::new(x, y))
                .ok_or_else(|| bad(cmd, "`origin` must be [x, y] on the canvas"))?,
        ),
    };
    let given = [page_vector_param(p, "ex", cmd)?, page_vector_param(p, "ey", cmd)?, page_vector_param(p, "ez", cmd)?];
    let unit = match p.get("unit").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(
            v.as_f64()
                .filter(|u| u.is_finite() && *u > 0.0 && *u <= MAX_UNIT)
                .ok_or_else(|| bad(cmd, format!("`unit` must be a length above 0 and at most {MAX_UNIT} pt")))?,
        ),
    };
    let make_reference = flag_param(p, "makeReference", cmd)?.unwrap_or(true);
    let artboard = match p.get("artboard").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(v.as_u64().and_then(|i| usize::try_from(i).ok()).ok_or_else(|| bad(cmd, "`artboard` must be an artboard index"))?),
    };
    let with = |f: Frame3| Frame3 {
        origin: origin.unwrap_or(f.origin),
        ex: given[0].unwrap_or(f.ex),
        ey: given[1].unwrap_or(f.ey),
        ez: given[2].unwrap_or(f.ez),
    };
    let plan = |frame: Frame3, source: Frame3Source, make_reference: bool| -> Result<Frame3Plan> {
        frame.check(cmd)?;
        Ok(Frame3Plan { frame, source, make_reference })
    };
    if let (Some(origin), [Some(ex), Some(ey), Some(ez)]) = (origin, given) {
        return plan(Frame3 { origin, ex, ey, ez }, Frame3Source::Params, false);
    }
    let st = s.doc()?;
    let d = &st.doc;
    // A selected group with cyan, magenta and yellow segments is meant as a reference object.
    for id in &st.selection.objects {
        let Some(n) = d.node(*id).filter(|n| matches!(n.kind, NodeKind::Group { .. })) else { continue };
        let mut segments = vec![];
        n.walk(&mut |c| {
            if let Some((a, b)) = segment_ends(c) {
                segments.push((a, b, axis_of(c)));
            }
        });
        if !(0..3).all(|axis| segments.iter().any(|s| s.2 == Some(axis))) {
            continue;
        }
        let frame = reference_frame(&segments).ok_or_else(|| {
            bad(cmd, "the selected group is not a 3D reference object: it must hold just three segments, cyan (x), magenta (y) and yellow (z), meeting at one point")
        })?;
        return plan(with(frame), Frame3Source::Reference(n.id), false);
    }
    let centre = match artboard {
        Some(i) => d.artboards.get(i).map(|a| a.rect.center()).ok_or_else(|| bad(cmd, format!("there is no artboard {i}")))?,
        None => d.artboards.first().map_or(Point::ZERO, |a| a.rect.center()),
    };
    let k = unit.unwrap_or(d.math_draw.std_length);
    let frame = Frame3 { origin: centre, ex: Vec2::new(DEFAULT_EX.0, DEFAULT_EX.1) * k, ey: Vec2::new(k, 0.0), ez: Vec2::new(0.0, -k) };
    plan(with(frame), Frame3Source::Artboard, make_reference)
}

// ---------- shared ----------

/// What every space command takes besides its own parameters.
struct Common3 {
    plan: Frame3Plan,
    line: Option<LineRef>,
    color: Color,
    style: MathDrawStyle,
    parent: Option<NodeId>,
}

fn common3(s: &Session, p: &Value, cmd: &str) -> Result<Common3> {
    if !p.is_object() && !p.is_null() {
        return Err(bad(cmd, "the parameters must be an object"));
    }
    let line = line_param(p, "line", cmd)?;
    let color = color_param(p, "color", cmd)?.unwrap_or(Color::BLACK);
    let plan = resolve_frame3(s, p, cmd)?;
    let st = s.doc()?;
    Ok(Common3 { plan, line, color, style: st.doc.math_draw.clone(), parent: st.target_parent()? })
}

/// One undo step labelled `label`: the projection's reference object (if it makes one), then what
/// `build` draws, inserted on top as one object (a group named `name` for several). A reference
/// object drawn is added to the selection. Answers what `build` answers with the `id` of what was
/// inserted and the `reference` object's.
fn draw3(
    s: &mut Session,
    c: &Common3,
    label: &str,
    name: &str,
    cmd: &str,
    build: impl FnOnce(&mut Document, &Frame3) -> Result<(Vec<Node>, Map<String, Value>)>,
) -> Result<Value> {
    let (plan, parent) = (c.plan, c.parent);
    s.edit(label, |d, sel| {
        let reference = plan.apply(d, parent)?;
        let (nodes, mut out) = build(d, &plan.frame)?;
        let id = insert_output(d, parent, name, nodes, cmd)?;
        if let (Frame3Source::Artboard, Some(r)) = (plan.source, reference) {
            sel.add(r);
        }
        out.insert("id".into(), json!(id.0));
        if let Some(r) = reference {
            out.insert("reference".into(), json!(r.0));
        }
        Ok(Value::Object(out))
    })
}

/// A math point (or vector) parameter `[x, y, z]`, each within ±[`MAX_MATH`].
fn point3_value(v: &Value) -> Option<V3> {
    finite_numbers::<3>(v).filter(|c| c.iter().all(|x| x.abs() <= MAX_MATH))
}

fn point3_param(p: &Value, key: &str, cmd: &str) -> Result<Option<V3>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => point3_value(v).map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be [x, y, z] within ±{MAX_MATH}"))),
    }
}

/// A direction parameter `[x, y, z]` (not zero), as a unit vector.
fn direction_param(p: &Value, key: &str, cmd: &str) -> Result<Option<V3>> {
    match point3_param(p, key, cmd)? {
        None => Ok(None),
        Some(v) => unit3(v).map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must not be the zero vector"))),
    }
}

/// A curve in space through its projection: the fitted cubics of `doc(t)` (document points) with
/// velocity `vel(t)` over `range` in `n` steps.
fn projected_runs(doc: &dyn Fn(f64) -> Point, vel: &dyn Fn(f64) -> Vec2, range: (f64, f64), n: usize) -> Vec<Vec<CubicBez>> {
    fit_parametric_with(&|t| doc(t).x, &|t| doc(t).y, &|t| vel(t).x, &|t| vel(t).y, range, n)
}

/// Fitted runs (document coordinates) as a path; `closed` closes a single run that ends where it
/// starts (a circle).
fn runs_path(runs: &[Vec<CubicBez>], closed: bool) -> PathData {
    if closed && let [run] = runs {
        let (Some(first), Some((last, init))) = (run.first(), run.split_last()) else { return PathData::default() };
        let mut bp = BezPath::new();
        bp.move_to(first.p0);
        for c in init {
            bp.curve_to(c.p1, c.p2, c.p3);
        }
        bp.curve_to(last.p1, last.p2, first.p0);
        bp.close_path();
        return PathData::from_bezpath(&bp);
    }
    PathData::from_bezpath(&vectorcraft_math::fit::to_bezpath(runs))
}

/// A dashed guide between math points `a` and `b` (none when they meet on the page).
fn guide3(d: &mut Document, frame: &Frame3, a: V3, b: V3, style: &MathDrawStyle, line: LineRef, color: Color) -> Option<Node> {
    let (pa, pb) = (frame.to_doc(a), frame.to_doc(b));
    ((pa - pb).hypot() > 1e-9).then(|| stroked(d, shapes::line(pa, pb), style, line, color))
}

// ---------- axes ----------

/// Three numbers `[a, b, c]` within `0..=MAX_MATH` (above 0 when `positive`).
fn triple_param(p: &Value, key: &str, positive: bool, cmd: &str) -> Result<Option<V3>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => finite_numbers::<3>(v)
            .filter(|c| c.iter().all(|x| *x <= MAX_MATH && if positive { *x > 0.0 } else { *x >= 0.0 }))
            .map(Some)
            .ok_or_else(|| {
                bad(
                    cmd,
                    format!(
                        "`{key}` must be three math lengths [x, y, z], each {} and at most {MAX_MATH}",
                        if positive { "above 0" } else { "0 or more" }
                    ),
                )
            }),
    }
}

/// The direction (an angle on the page) of the widest gap between rays from the origin along
/// `dirs`.
fn widest_gap(dirs: &[Vec2]) -> f64 {
    let mut angles: Vec<f64> = dirs.iter().map(|v| v.y.atan2(v.x)).filter(|a| a.is_finite()).collect();
    angles.sort_by(f64::total_cmp);
    let (Some(&first), Some(&last)) = (angles.first(), angles.last()) else { return std::f64::consts::PI };
    // From the last ray round to the first, then between neighbours.
    let mut best = (first + std::f64::consts::TAU - last, last);
    for w in angles.windows(2) {
        if let [a, b] = w
            && b - a > best.0
        {
            best = (b - a, *a);
        }
    }
    best.1 + best.0 / 2.0
}

fn axes3d(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.axes3d";
    let lengths = triple_param(p, "lengths", true, C)?.unwrap_or([2.0; 3]);
    let negative = triple_param(p, "negative", false, C)?.unwrap_or([0.0; 3]);
    let c = common3(s, p, C)?;
    let line = c.line.unwrap_or(c.style.axis_line);
    draw3(s, &c, "Space Coordinate Axes", "3D Axes", C, |d, frame| {
        let style = &c.style;
        let (mut axes, mut arrows, mut labels) = (vec![], vec![], vec![]);
        let mut tips = vec![];
        // The rays from the origin the axes draw, for placing O between them.
        let mut rays = vec![];
        for k in 0..3 {
            let mut e = [0.0; 3];
            e[k] = 1.0;
            let (len, neg) = (lengths.get(k).copied().unwrap_or(2.0), negative.get(k).copied().unwrap_or(0.0));
            let (from, tip) = (frame.to_doc(scale3(e, -neg)), frame.to_doc(scale3(e, len)));
            let (axis, arrow) = axis_with_arrow(d, style, from, tip, line, c.color, C)?;
            axes.push(axis);
            arrows.push(arrow);
            let v = frame.vec(e);
            tips.push((tip, v));
            rays.push(v);
            if neg > 0.0 {
                rays.push(-v);
            }
        }
        let mut out = Map::new();
        out.insert("axes".into(), ids(&axes));
        out.insert("arrows".into(), ids(&arrows));
        if have_math_font() {
            for ((tip, v), text) in tips.into_iter().zip(["x", "y", "z"]) {
                // Like the plane's x axis (label under the tip) or its y axis (left of the tip).
                let kind = if v.x.abs() >= v.y.abs() { AxisLabelKind::X } else { AxisLabelKind::Y };
                labels.push(axis_label(d, style, text, style.label(kind).italic, label_at(style, kind, tip), c.color, C)?);
            }
            // O: as far from the origin as the plane's O is, in the widest gap between the axes.
            let o = frame.origin;
            let mut l = axis_label(d, style, "O", style.label(AxisLabelKind::Origin).italic, label_at(style, AxisLabelKind::Origin, o), c.color, C)?;
            if let Some(b) = l.geometric_bounds() {
                let r = (b.center() - o).hypot();
                let a = widest_gap(&rays);
                let to = o + Vec2::new(a.cos(), a.sin()) * r;
                l.transform(Affine::translate(to - b.center()), false);
            }
            labels.push(l);
            out.insert("labels".into(), ids(&labels));
        } else {
            out.insert("labels".into(), json!(NO_FONT));
        }
        let nodes = axes.into_iter().zip(arrows).flat_map(|(a, b)| [a, b]).chain(labels).collect();
        Ok((nodes, out))
    })
}

// ---------- points ----------

/// The guides of point `q`: down (along z) to the xy plane and from there to the x and y axes;
/// with `planes`, also to the yz and zx planes and from each of those to its two axes (the edges
/// of the box from the origin to `q` that aren't on an axis). Each once, from → to.
fn point_guides(q: V3, planes: bool) -> Vec<(V3, V3)> {
    let [x, y, z] = q;
    let (xy, yz, zx) = ([x, y, 0.0], [0.0, y, z], [x, 0.0, z]);
    let (ax, ay, az) = ([x, 0.0, 0.0], [0.0, y, 0.0], [0.0, 0.0, z]);
    let mut out = vec![(q, xy), (xy, ax), (xy, ay)];
    if planes {
        out.extend([(q, yz), (q, zx), (yz, ay), (yz, az), (zx, ax), (zx, az)]);
    }
    // The same edge twice where a coordinate is 0.
    let mut unique: Vec<(V3, V3)> = vec![];
    for (a, b) in out {
        if !unique.iter().any(|&(c, d)| (c == a && d == b) || (c == b && d == a)) {
            unique.push((a, b));
        }
    }
    unique
}

fn points3d(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.points3d";
    let list = p.get("coords").filter(|v| !v.is_null()).ok_or_else(|| bad(C, "missing `coords`: [[x, y, z], …]"))?;
    let err = || bad(C, format!("`coords` must be a list of [x, y, z] points within ±{MAX_MATH} (at most {MAX_ITEMS})"));
    let coords = list
        .as_array()
        .filter(|a| a.len() <= MAX_ITEMS)
        .ok_or_else(err)?
        .iter()
        .map(|v| point3_value(v).ok_or_else(err))
        .collect::<Result<Vec<_>>>()?;
    if coords.is_empty() {
        return Err(bad(C, "`coords` has no points"));
    }
    let with_guides = flag_param(p, "guides", C)?.unwrap_or(true);
    let planes = flag_param(p, "planeGuides", C)?.unwrap_or(false);
    let size = number_param(p, "dot", C)?;
    let (lo, hi) = vectorcraft_doc::mathdraw::limits::DOT;
    if size.is_some_and(|d| !(lo..=hi).contains(&d)) {
        return Err(bad(C, format!("`dot` must be a diameter from {lo} to {hi} pt")));
    }
    let mut c = common3(s, p, C)?;
    if let Some(size) = size {
        c.style.dot_size = size;
    }
    let line = c.line.unwrap_or(LineRef::Dashed(3));
    draw3(s, &c, "Points by Space Coordinates", "3D Points", C, |d, frame| {
        let guides: Vec<Node> = if with_guides || planes {
            coords.iter().flat_map(|&q| point_guides(q, planes)).filter_map(|(a, b)| guide3(d, frame, a, b, &c.style, line, c.color)).collect()
        } else {
            vec![]
        };
        let dots: Vec<Node> = coords.iter().map(|&q| dot(d, frame.to_doc(q), &c.style, c.color, Color::WHITE)).collect();
        let mut out = Map::new();
        out.insert("dots".into(), ids(&dots));
        out.insert("guides".into(), ids(&guides));
        Ok((guides.into_iter().chain(dots).collect(), out))
    })
}

// ---------- curves ----------

fn parametric3d(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.parametric3d";
    let exprs: [Expr; 3] = [expr_param(p, "x", &["t"], C)?.1, expr_param(p, "y", &["t"], C)?.1, expr_param(p, "z", &["t"], C)?.1];
    let range = range_param(p, "tRange", C)?.ok_or_else(|| bad(C, "missing `tRange`: [a, b]"))?;
    let n = count_param(p, "precision", 40, 1..=MAX_PRECISION, C)?;
    let c = common3(s, p, C)?;
    let frame = c.plan.frame;
    let at = |e: &[Expr; 3], t: f64| -> V3 {
        let [x, y, z] = e;
        [x.eval(&[("t", t)]), y.eval(&[("t", t)]), z.eval(&[("t", t)])]
    };
    let doc = |t: f64| frame.to_doc(at(&exprs, t));
    let runs = match (exprs[0].diff("t"), exprs[1].diff("t"), exprs[2].diff("t")) {
        (Ok(dx), Ok(dy), Ok(dz)) => {
            let ds = [dx, dy, dz];
            projected_runs(&doc, &|t| frame.vec(at(&ds, t)), range, n)
        }
        // Too large to differentiate: a smooth curve through samples.
        _ => {
            let m = (4 * n).clamp(8, 4 * MAX_PRECISION);
            let pts: Vec<(f64, f64)> = (0..=m)
                .map(|i| {
                    let t = if i == m { range.1 } else { range.0 + (range.1 - range.0) * i as f64 / m as f64 };
                    let q = doc(t);
                    (q.x, q.y)
                })
                .collect();
            fit_samples(&pts)
        }
    };
    if runs.is_empty() {
        return Err(bad(C, "the curve has no points in `tRange`"));
    }
    let line = c.line.unwrap_or(LineRef::Solid(3));
    draw3(s, &c, "Space Parametric Curve", "3D Curve", C, |d, _| {
        let curve = stroked(d, runs_path(&runs, false), &c.style, line, c.color);
        let mut out = Map::new();
        out.insert("curve".into(), json!(curve.id.0));
        Ok((vec![curve], out))
    })
}

/// Where parallel light along `dir` casts the shadow of points on the plane through `point`
/// perpendicular to `normal` (unit vectors).
#[derive(Clone, Copy, Debug)]
struct Shadow {
    normal: V3,
    point: V3,
    dir: V3,
}

impl Shadow {
    fn from_param(v: &Value, cmd: &str) -> Result<Self> {
        let o = v.as_object().ok_or_else(|| bad(cmd, "`shadow` must be {planeNormal, planePoint?, lightDir?}"))?;
        if let Some(k) = o.keys().find(|k| !["planeNormal", "planePoint", "lightDir"].contains(&k.as_str())) {
            return Err(bad(cmd, format!("`shadow` has an unknown key `{k}` (it takes planeNormal, planePoint, lightDir)")));
        }
        let normal = direction_param(v, "planeNormal", cmd)?.ok_or_else(|| bad(cmd, "`shadow` is missing `planeNormal`: [x, y, z]"))?;
        let point = point3_param(v, "planePoint", cmd)?.unwrap_or([0.0; 3]);
        let dir = direction_param(v, "lightDir", cmd)?.unwrap_or(normal);
        if dot3(dir, normal).abs() < 1e-9 {
            return Err(bad(cmd, "the light is parallel to the plane: it casts no shadow on it"));
        }
        Ok(Self { normal, point, dir })
    }

    /// The shadow of point `q`.
    fn point(&self, q: V3) -> V3 {
        let k = dot3(add3(q, scale3(self.point, -1.0)), self.normal) / dot3(self.dir, self.normal);
        add3(q, scale3(self.dir, -k))
    }

    /// The shadow of vector `v` (the map's linear part).
    fn vector(&self, v: V3) -> V3 {
        let k = dot3(v, self.normal) / dot3(self.dir, self.normal);
        add3(v, scale3(self.dir, -k))
    }
}

fn circle3d(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.circle3d";
    let center = point3_param(p, "center", C)?.ok_or_else(|| bad(C, "missing `center`: [x, y, z]"))?;
    let normal = direction_param(p, "normal", C)?.ok_or_else(|| bad(C, "missing `normal`: [x, y, z]"))?;
    let r = number_param(p, "radius", C)?.ok_or_else(|| bad(C, "missing `radius` (math units)"))?;
    if !(r > 0.0 && r <= MAX_MATH) {
        return Err(bad(C, format!("`radius` must be above 0 and at most {MAX_MATH}")));
    }
    let shadow = match p.get("shadow").filter(|v| !v.is_null()) {
        None => None,
        Some(v) => Some(Shadow::from_param(v, C)?),
    };
    let shadow_line = line_param(p, "shadowLine", C)?.unwrap_or(LineRef::Dashed(3));
    let c = common3(s, p, C)?;
    let frame = c.plan.frame;
    let (u, v) = plane_basis(normal);
    let point = |t: f64| add3(center, add3(scale3(u, r * t.cos()), scale3(v, r * t.sin())));
    let velocity = |t: f64| add3(scale3(u, -r * t.sin()), scale3(v, r * t.cos()));
    let full = (0.0, std::f64::consts::TAU);
    let circle = projected_runs(&|t| frame.to_doc(point(t)), &|t| frame.vec(velocity(t)), full, CIRCLE_STEPS);
    let shadow_runs =
        shadow.map(|sh| projected_runs(&|t| frame.to_doc(sh.point(point(t))), &|t| frame.vec(sh.vector(velocity(t))), full, CIRCLE_STEPS));
    if circle.is_empty() || shadow_runs.as_ref().is_some_and(Vec::is_empty) {
        return Err(bad(C, "the circle can't be drawn on the page"));
    }
    let line = c.line.unwrap_or(LineRef::Solid(3));
    draw3(s, &c, "Space Circle", "3D Circle", C, |d, _| {
        let mut out = Map::new();
        let mut nodes = vec![];
        if let Some(runs) = &shadow_runs {
            let n = stroked(d, runs_path(runs, true), &c.style, shadow_line, c.color);
            out.insert("shadow".into(), json!(n.id.0));
            nodes.push(n);
        }
        let n = stroked(d, runs_path(&circle, true), &c.style, line, c.color);
        out.insert("circle".into(), json!(n.id.0));
        nodes.push(n);
        Ok((nodes, out))
    })
}
