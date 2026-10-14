//! Curves through functions: the graph of `y = f(x)`, a parametric curve `(x(t), y(t))`, or a list
//! of samples, as cubic Béziers in math coordinates (y up; the caller maps them to the page).
//!
//! A graph is cut into `n` equal steps (the "precision") and each step is one cubic: the Hermite
//! cubic through the step's end points with the function's slopes there (`f′` from
//! [`Expr::diff`]), whose control points are `p0 + (1, f′(x0))·h/3` and `p3 − (1, f′(x1))·h/3`. It
//! is exact for polynomials of degree 3 or less. Where `f′` is not a number the slope comes from
//! the neighbouring samples; where it is infinite (a vertical tangent, as `\sqrt x` at 0) or wild,
//! the controls come from where the two end tangents meet.
//!
//! The graph is split where the function stops: where it is not a number (outside its domain),
//! leaves the band of values it is drawn in ([`FitOptions::clip`]), or has a pole (a vertical
//! asymptote, found by bisecting a change of sign). Each connected piece is one run of cubics.
//!
//! Inputs are untrusted: step counts are capped ([`MAX_SEGMENTS`]), every search is bounded and
//! nothing panics; a range that isn't finite gives no curve.

use vectorcraft_geom::{BezPath, CubicBez, Point, Vec2};

use crate::eval::Expr;

/// Most steps a curve is cut into.
pub const MAX_SEGMENTS: usize = 10_000;

/// Connected pieces of a curve, each a list of cubics where each starts at the previous one's end.
pub type Runs = Vec<Vec<CubicBez>>;

/// How a graph is fitted.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FitOptions {
    /// The values (`[low, high]`) the graph is drawn between: it is cut where it leaves them. None:
    /// a band a thousand times wider than the graph's typical value, so only the reaches of a pole
    /// are cut.
    pub clip: Option<(f64, f64)>,
}

/// Steps a graph step is split into when it isn't plain (it leaves the band, or has a pole).
const SUBSTEPS: usize = 16;
/// Halvings of a bisection: enough to reach neighbouring floats on any sensible range.
const BISECTIONS: usize = 200;

/// The graph of `f` in the variable `var` over `range`, in `n` steps.
pub fn fit_function(f: &Expr, var: &str, range: (f64, f64), n: usize, opt: FitOptions) -> Runs {
    let value = |x: f64| f.eval(&[(var, x)]);
    let derivative = f.diff(var).ok();
    let h = (range.1 - range.0).abs() / n.clamp(1, MAX_SEGMENTS) as f64;
    let slope = |x: f64| match &derivative {
        Some(d) => d.eval(&[(var, x)]),
        // Too large to differentiate: a numerical slope.
        None => numeric_slope(&value, x, h),
    };
    fit_graph(&value, &slope, range, n, opt)
}

/// The graph of a function given by its values `f` and slopes `df` (see [`fit_function`]).
pub fn fit_graph(f: &dyn Fn(f64) -> f64, df: &dyn Fn(f64) -> f64, range: (f64, f64), n: usize, opt: FitOptions) -> Runs {
    let (a, b) = if range.0 <= range.1 { range } else { (range.1, range.0) };
    if !(a.is_finite() && b.is_finite()) || a == b {
        return vec![];
    }
    let n = n.clamp(1, MAX_SEGMENTS);
    let xs: Vec<f64> = (0..=n).map(|i| if i == n { b } else { a + (b - a) * i as f64 / n as f64 }).collect();
    let ys: Vec<f64> = xs.iter().map(|&x| f(x)).collect();
    let band = band(&ys, opt.clip);
    let g = Graph { f, df, band };
    let slopes = g.sample_slopes(&xs, &ys);
    let mut out = RunBuilder::default();
    for i in 0..n {
        let (Some(&x0), Some(&x1), Some(&y0), Some(&y1), Some(&s0), Some(&s1)) =
            (xs.get(i), xs.get(i + 1), ys.get(i), ys.get(i + 1), slopes.get(i), slopes.get(i + 1))
        else {
            break;
        };
        for piece in g.pieces(Knot { x: x0, y: y0, s: s0 }, Knot { x: x1, y: y1, s: s1 }) {
            out.push(piece.0, piece.1);
        }
    }
    out.finish()
}

/// A point of a graph with its slope.
#[derive(Clone, Copy, Debug)]
struct Knot {
    x: f64,
    y: f64,
    s: f64,
}

/// The band a graph is drawn in: the clip given (put in order), else ±1000 × (1 + the median |y|).
fn band(ys: &[f64], clip: Option<(f64, f64)>) -> (f64, f64) {
    if let Some((lo, hi)) = clip
        && !lo.is_nan()
        && !hi.is_nan()
        && lo != hi
    {
        return (lo.min(hi), lo.max(hi));
    }
    let mut mags: Vec<f64> = ys.iter().filter(|y| y.is_finite()).map(|y| y.abs()).collect();
    mags.sort_by(f64::total_cmp);
    let median = mags.get(mags.len() / 2).copied().unwrap_or(0.0);
    let b = 1000.0 * (1.0 + median);
    (-b, b)
}

/// A central difference (one-sided where one side isn't a number).
fn numeric_slope(f: &dyn Fn(f64) -> f64, x: f64, h: f64) -> f64 {
    let d = (h * 1e-4).max(1e-9 * x.abs().max(1.0));
    let (l, c, r) = (f(x - d), f(x), f(x + d));
    if l.is_finite() && r.is_finite() {
        (r - l) / (2.0 * d)
    } else if r.is_finite() && c.is_finite() {
        (r - c) / d
    } else if l.is_finite() && c.is_finite() {
        (c - l) / d
    } else {
        f64::NAN
    }
}

struct Graph<'a> {
    f: &'a dyn Fn(f64) -> f64,
    df: &'a dyn Fn(f64) -> f64,
    band: (f64, f64),
}

impl Graph<'_> {
    fn inside(&self, y: f64) -> bool {
        y.is_finite() && y >= self.band.0 && y <= self.band.1
    }

    /// The slopes at the samples. One that isn't a number comes from the neighbouring samples
    /// inside the band; an infinite one (a vertical tangent) is kept.
    fn sample_slopes(&self, xs: &[f64], ys: &[f64]) -> Vec<f64> {
        let at = |i: usize| -> Option<(f64, f64)> {
            let (x, y) = (*xs.get(i)?, *ys.get(i)?);
            self.inside(y).then_some((x, y))
        };
        (0..xs.len())
            .map(|i| {
                let Some((x, y)) = at(i) else { return f64::NAN };
                let s = (self.df)(x);
                if !s.is_nan() {
                    return s;
                }
                let before = i.checked_sub(1).and_then(at);
                match (before, at(i + 1)) {
                    (Some(l), Some(r)) => (r.1 - l.1) / (r.0 - l.0),
                    (Some(l), None) => (y - l.1) / (x - l.0),
                    (None, Some(r)) => (r.1 - y) / (r.0 - x),
                    (None, None) => 0.0,
                }
            })
            .collect()
    }

    /// The pieces of one step from `k0` to `k1`: the cubics with where they start.
    fn pieces(&self, k0: Knot, k1: Knot) -> Vec<(Point, CubicBez)> {
        if self.inside(k0.y) && self.inside(k1.y) && self.inside((self.f)((k0.x + k1.x) / 2.0)) {
            let pole = k0.y * k1.y < 0.0 && self.pole_between(k0.x, k0.y, k1.x, k1.y).is_some();
            if !pole {
                return vec![(Point::new(k0.x, k0.y), cubic(k0, k1))];
            }
        }
        self.split_pieces(k0, k1)
    }

    /// A step that leaves the band, has a pole or isn't a number somewhere: sampled finer, the
    /// stretches inside the band, each ended exactly where the graph leaves it.
    fn split_pieces(&self, k0: Knot, k1: Knot) -> Vec<(Point, CubicBez)> {
        let t = |j: usize| match j {
            0 => k0.x,
            j if j >= SUBSTEPS => k1.x,
            j => k0.x + (k1.x - k0.x) * j as f64 / SUBSTEPS as f64,
        };
        let vs: Vec<f64> = (0..=SUBSTEPS)
            .map(|j| match j {
                0 => k0.y,
                j if j == SUBSTEPS => k1.y,
                j => (self.f)(t(j)),
            })
            .collect();
        let mut out = vec![];
        // Where the stretch being followed starts.
        let mut start: Option<Knot> = None;
        for (j, &v) in vs.iter().enumerate() {
            if !self.inside(v) {
                continue;
            }
            let s = match start {
                Some(s) => s,
                None if j == 0 => k0,
                // The sample before was outside.
                None => self.boundary(t(j), t(j - 1)),
            };
            if j == SUBSTEPS {
                push_piece(&mut out, s, k1);
                break;
            }
            let next = vs.get(j + 1).copied().unwrap_or(f64::NAN);
            if !self.inside(next) {
                push_piece(&mut out, s, self.boundary(t(j), t(j + 1)));
                start = None;
            } else if let Some((pa, pb)) = (v * next < 0.0).then(|| self.pole_between(t(j), v, t(j + 1), next)).flatten() {
                // Through a pole: end this stretch before it, start the next one after it.
                let end = if self.inside((self.f)(pa)) { self.knot(pa) } else { self.boundary(t(j), pa) };
                push_piece(&mut out, s, end);
                start = Some(if self.inside((self.f)(pb)) { self.knot(pb) } else { self.boundary(t(j + 1), pb) });
            } else {
                start = Some(s);
            }
        }
        out
    }

    /// The graph's point (and slope) at `x`.
    fn knot(&self, x: f64) -> Knot {
        Knot { x, y: (self.f)(x), s: (self.df)(x) }
    }

    /// Where the graph leaves the band between `inside_x` (inside it) and `outside_x`: the last
    /// point inside, found by bisection.
    fn boundary(&self, inside_x: f64, outside_x: f64) -> Knot {
        let (mut a, mut b) = (inside_x, outside_x);
        for _ in 0..BISECTIONS {
            let m = a + (b - a) / 2.0;
            if m == a || m == b {
                break;
            }
            if self.inside((self.f)(m)) {
                a = m;
            } else {
                b = m;
            }
        }
        self.knot(a)
    }

    /// Whether the change of sign between `(xa, ya)` and `(xb, yb)` is a pole rather than a root:
    /// the bisection's last two points, either side of the pole, when it is.
    fn pole_between(&self, mut xa: f64, mut ya: f64, mut xb: f64, mut yb: f64) -> Option<(f64, f64)> {
        let start = ya.abs().min(yb.abs());
        for _ in 0..BISECTIONS {
            let m = xa + (xb - xa) / 2.0;
            if m == xa || m == xb {
                break;
            }
            let ym = (self.f)(m);
            if !ym.is_finite() {
                // Not a number at the pole itself: step off it to either side.
                let (l, r) = (prev_float(m, xa), next_float(m, xb));
                return Some((l, r));
            }
            if ym == 0.0 {
                return None;
            }
            if (ym < 0.0) == (ya < 0.0) {
                (xa, ya) = (m, ym);
            } else {
                (xb, yb) = (m, ym);
            }
        }
        // A root ends between tiny values, a pole between huge ones.
        (ya.abs().min(yb.abs()) > 1e3 * start).then_some((xa, xb))
    }
}

/// The float just below `x` (not below `floor`).
fn prev_float(x: f64, floor: f64) -> f64 {
    let d = (x.abs() * f64::EPSILON).max(f64::MIN_POSITIVE);
    (x - d).max(floor)
}

/// The float just above `x` (not above `ceil`).
fn next_float(x: f64, ceil: f64) -> f64 {
    let d = (x.abs() * f64::EPSILON).max(f64::MIN_POSITIVE);
    (x + d).min(ceil)
}

fn push_piece(out: &mut Vec<(Point, CubicBez)>, a: Knot, b: Knot) {
    if b.x > a.x && a.y.is_finite() && b.y.is_finite() {
        out.push((Point::new(a.x, a.y), cubic(a, b)));
    }
}

/// The cubic from `a` to `b` with their slopes: Hermite when the slopes are tame, else through
/// where the end tangents meet (a vertical tangent, a slope far steeper than the step), else with
/// the step's secant slope in place of a wild one.
fn cubic(a: Knot, b: Knot) -> CubicBez {
    let (p0, p3) = (Point::new(a.x, a.y), Point::new(b.x, b.y));
    let h = b.x - a.x;
    let dy = b.y - a.y;
    let tame = |s: f64| s.is_finite() && (s * h / 3.0).abs() <= 10.0 * (dy.abs() + h.abs());
    let hermite =
        |s0: f64, s1: f64| CubicBez::new(p0, Point::new(a.x + h / 3.0, a.y + s0 * h / 3.0), Point::new(b.x - h / 3.0, b.y - s1 * h / 3.0), p3);
    if tame(a.s) && tame(b.s) {
        return hermite(a.s, b.s);
    }
    // The tangent directions, in the direction of travel.
    let up = if dy >= 0.0 { 1.0 } else { -1.0 };
    let dir = |s: f64| -> Option<Vec2> {
        if s.is_finite() {
            Some(Vec2::new(1.0, s))
        } else if s.is_infinite() {
            Some(Vec2::new(0.0, up))
        } else {
            None
        }
    };
    if let (Some(d0), Some(d1)) = (dir(a.s), dir(b.s)) {
        let chord = p3 - p0;
        let det = d0.cross(d1);
        if det.abs() > 1e-12 * d0.hypot() * d1.hypot() {
            let t0 = chord.cross(d1) / det;
            let t1 = d0.cross(chord) / det;
            let meet = p0 + d0 * t0;
            if t0 > 0.0 && t1 > 0.0 && meet.x.is_finite() && meet.y.is_finite() {
                return CubicBez::new(p0, p0.lerp(meet, 2.0 / 3.0), p3.lerp(meet, 2.0 / 3.0), p3);
            }
        }
    }
    let secant = dy / h;
    let pick = |s: f64| if tame(s) { s } else { secant };
    hermite(pick(a.s), pick(b.s))
}

/// Collects cubics into runs: a cubic that doesn't start where the last one ended starts a new run.
#[derive(Default)]
struct RunBuilder {
    runs: Runs,
    current: Vec<CubicBez>,
}

impl RunBuilder {
    fn push(&mut self, start: Point, c: CubicBez) {
        if self.current.last().is_some_and(|last| last.p3 != start) {
            self.runs.push(std::mem::take(&mut self.current));
        }
        self.current.push(c);
    }

    fn finish(mut self) -> Runs {
        if !self.current.is_empty() {
            self.runs.push(self.current);
        }
        self.runs
    }
}

/// The curve `(x(t), y(t))` in the variable `var` over `range`, in `n` steps: the cubic of each
/// step has the controls `p0 + v0·h/3` and `p3 − v1·h/3`, `v` the velocity `(x′, y′)`.
pub fn fit_parametric(x: &Expr, y: &Expr, var: &str, range: (f64, f64), n: usize) -> Runs {
    let fx = |t: f64| x.eval(&[(var, t)]);
    let fy = |t: f64| y.eval(&[(var, t)]);
    let (dx, dy) = (x.diff(var).ok(), y.diff(var).ok());
    let h = (range.1 - range.0).abs() / n.clamp(1, MAX_SEGMENTS) as f64;
    let dfx = |t: f64| dx.as_ref().map_or_else(|| numeric_slope(&fx, t, h), |d| d.eval(&[(var, t)]));
    let dfy = |t: f64| dy.as_ref().map_or_else(|| numeric_slope(&fy, t, h), |d| d.eval(&[(var, t)]));
    fit_parametric_with(&fx, &fy, &dfx, &dfy, range, n)
}

/// [`fit_parametric`] for coordinates given by their values and derivatives.
pub fn fit_parametric_with(
    fx: &dyn Fn(f64) -> f64,
    fy: &dyn Fn(f64) -> f64,
    dfx: &dyn Fn(f64) -> f64,
    dfy: &dyn Fn(f64) -> f64,
    range: (f64, f64),
    n: usize,
) -> Runs {
    let (a, b) = range;
    if !(a.is_finite() && b.is_finite()) || a == b {
        return vec![];
    }
    let n = n.clamp(1, MAX_SEGMENTS);
    let ts: Vec<f64> = (0..=n).map(|i| if i == n { b } else { a + (b - a) * i as f64 / n as f64 }).collect();
    let ps: Vec<Point> = ts.iter().map(|&t| Point::new(fx(t), fy(t))).collect();
    let finite = |p: &Point| p.x.is_finite() && p.y.is_finite();
    let point = |i: usize| ps.get(i).copied().filter(finite);
    let vs: Vec<Vec2> = ts
        .iter()
        .enumerate()
        .map(|(i, &t)| {
            let v = Vec2::new(dfx(t), dfy(t));
            if v.x.is_finite() && v.y.is_finite() {
                return v;
            }
            // From the neighbouring points.
            let (Some(p), before, after) = (point(i), i.checked_sub(1).and_then(point), point(i + 1)) else { return Vec2::ZERO };
            let step = (b - a) / n as f64;
            match (before, after) {
                (Some(l), Some(r)) => (r - l) / (2.0 * step),
                (Some(l), None) => (p - l) / step,
                (None, Some(r)) => (r - p) / step,
                (None, None) => Vec2::ZERO,
            }
        })
        .collect();
    // A coordinate that changes sign through a pole between two samples breaks the curve.
    let pole = |f: &dyn Fn(f64) -> f64, t0: f64, v0: f64, t1: f64, v1: f64| {
        let g = Graph { f, df: f, band: (f64::NEG_INFINITY, f64::INFINITY) };
        v0 * v1 < 0.0 && g.pole_between(t0, v0, t1, v1).is_some()
    };
    let mut out = RunBuilder::default();
    for i in 0..n {
        let (Some(p0), Some(p3), Some(&v0), Some(&v1), Some(&t0), Some(&t1)) =
            (point(i), point(i + 1), vs.get(i), vs.get(i + 1), ts.get(i), ts.get(i + 1))
        else {
            continue;
        };
        if pole(fx, t0, p0.x, t1, p3.x) || pole(fy, t0, p0.y, t1, p3.y) {
            continue;
        }
        let h = t1 - t0;
        out.push(p0, CubicBez::new(p0, p0 + v0 * (h / 3.0), p3 - v1 * (h / 3.0), p3));
    }
    out.finish()
}

/// A smooth curve through samples (data from paths: derivative, integral and combined graphs),
/// Catmull-Rom style: each point's tangent is the chord between its neighbours (second-order
/// one-sided at the ends). A sample that isn't a number breaks the curve.
pub fn fit_samples(pts: &[(f64, f64)]) -> Runs {
    let mut runs = vec![];
    let mut stretch: Vec<Point> = vec![];
    let flush = |stretch: &mut Vec<Point>, runs: &mut Runs| {
        if stretch.len() >= 2 {
            runs.push(catmull_rom(stretch));
        }
        stretch.clear();
    };
    for &(x, y) in pts {
        if x.is_finite() && y.is_finite() {
            stretch.push(Point::new(x, y));
        } else {
            flush(&mut stretch, &mut runs);
        }
    }
    flush(&mut stretch, &mut runs);
    runs
}

fn catmull_rom(p: &[Point]) -> Vec<CubicBez> {
    let n = p.len();
    let at = |i: usize| p.get(i).copied().unwrap_or_default();
    // Tangents per index step.
    let tangent = |i: usize| -> Vec2 {
        if n == 2 {
            at(1) - at(0)
        } else if i == 0 {
            (at(1) - at(0)) * 2.0 - (at(2) - at(0)) * 0.5
        } else if i + 1 == n {
            (at(n - 1) - at(n - 2)) * 2.0 - (at(n - 1) - at(n - 3)) * 0.5
        } else {
            (at(i + 1) - at(i - 1)) * 0.5
        }
    };
    (0..n.saturating_sub(1)).map(|i| CubicBez::new(at(i), at(i) + tangent(i) / 3.0, at(i + 1) - tangent(i + 1) / 3.0, at(i + 1))).collect()
}

/// Runs as one path, a subpath per run.
pub fn to_bezpath(runs: &[Vec<CubicBez>]) -> BezPath {
    let mut path = BezPath::new();
    for run in runs {
        let Some(first) = run.first() else { continue };
        path.move_to(first.p0);
        for c in run {
            path.curve_to(c.p1, c.p2, c.p3);
        }
    }
    path
}
