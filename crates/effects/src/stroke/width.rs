//! Variable-width strokes (Width tool / width profiles).
//!
//! A stroke with a [`WidthProfile`] is drawn as a filled outline: each subpath is flattened, and
//! at every sample the left and right offsets `width/2 · factor(t)` (t = fraction of the
//! subpath's length) are placed along the normal. Corners of the path take the stroke's miter,
//! round or bevel join on their outer side (smooth points bend round), open subpaths get butt,
//! round or projecting caps (the round cap blends the two side widths), and closed subpaths become
//! two loops of opposite orientation filled with the non-zero rule. Dashes are pieces of a longer
//! subpath and read the profile where they sit along it ([`outline_spans`]).

use std::f64::consts::PI;

use kurbo::{BezPath, ParamCurve, PathEl, PathSeg, Point, Vec2};
use vectorcraft_doc::{LineCap, LineJoin, StrokeLayer, WidthProfile};

use super::{kink, segments, subpaths, unit};

/// Filled outline of `bp` stroked with `width` scaled by `profile`, with the caps, joins and miter
/// limit of `st`, flattened to `tol`.
pub fn width_outline(bp: &BezPath, width: f64, profile: &WidthProfile, st: &StrokeLayer, tol: f64) -> BezPath {
    outline_spans(bp, &[], width, profile, st, tol)
}

/// [`width_outline`] of pieces of longer subpaths: subpath `i` of `bp` covers the fractions
/// `spans[i]` of the subpath it was cut from (see [`Dashed::spans`](super::Dashed::spans));
/// subpaths without a span are whole.
pub(super) fn outline_spans(bp: &BezPath, spans: &[(f64, f64)], width: f64, profile: &WidthProfile, st: &StrokeLayer, tol: f64) -> BezPath {
    let ctx = Ctx { half: width / 2.0, profile, cap: st.cap, join: st.join, miter_limit: st.miter_limit, tol: tol.max(1e-4) };
    let mut out = BezPath::new();
    for (i, (r, closed)) in subpaths(bp).into_iter().enumerate() {
        let (pts, corner) = flatten(&segments(&bp.elements()[r]), closed, ctx.tol);
        if pts.len() > 1 {
            ctx.subpath(&mut out, &pts, &corner, closed, spans.get(i).copied().unwrap_or((0.0, 1.0)));
        }
    }
    out
}

/// Left normal in y-down space (left of the direction of travel).
pub(super) fn left(t: Vec2) -> Vec2 {
    Vec2::new(t.y, -t.x)
}

/// The flattened points of one subpath (consecutive duplicates removed; a closed one doesn't
/// repeat its start), each marked when it is a corner of the path.
pub(super) fn flatten(segs: &[PathSeg], closed: bool, tol: f64) -> (Vec<Point>, Vec<bool>) {
    let Some(s0) = segs.first() else { return (vec![], vec![]) };
    let (mut pts, mut corner) = (vec![s0.start()], vec![false]);
    // The first and the last segment that drew something.
    let (mut first, mut prev): (Option<&PathSeg>, Option<&PathSeg>) = (None, None);
    for seg in segs {
        let at = pts.len() - 1;
        let mut push = |p: Point| {
            if pts.last().is_none_or(|l| l.distance(p) > 1e-9) {
                pts.push(p);
                corner.push(false);
            }
        };
        match seg {
            PathSeg::Line(l) => push(l.p1),
            _ => kurbo::flatten([PathEl::MoveTo(seg.start()), seg.as_path_el()], tol, |el| {
                if let PathEl::LineTo(p) = el {
                    push(p);
                }
            }),
        }
        if pts.len() > at + 1 {
            match prev {
                Some(p) => corner[at] = kink(p, seg),
                None => first = Some(seg),
            }
            prev = Some(seg);
        }
    }
    if closed {
        if pts.len() > 1 && pts[0].distance(pts[pts.len() - 1]) <= 1e-9 {
            pts.pop();
            corner.pop();
        }
        if let (Some(a), Some(b)) = (prev, first) {
            corner[0] = kink(a, b);
        }
    }
    (pts, corner)
}

/// Insert samples where the profile has width points and between them, so its smooth curve is
/// drawn as a smooth outline along straight runs too (the two points of a discontinuous point
/// share one sample). `span` maps the subpath onto the profile (see [`outline_spans`]).
fn densify(pts: &[Point], corner: &[bool], closed: bool, profile: &WidthProfile, span: (f64, f64)) -> (Vec<Point>, Vec<bool>) {
    // Samples per profile segment; fewer when a profile carries many points, so a file can't turn
    // the extra samples into a huge allocation.
    const SAMPLES: usize = 16;
    let total = length(pts, closed);
    let range = span.1 - span.0;
    if total <= 1e-12 || range <= 1e-12 {
        return (pts.to_vec(), corner.to_vec());
    }
    // Where the profile's points fall along this piece (a span past 1 wraps round a closed path).
    let mut ts: Vec<f64> = profile.points.iter().flat_map(|p| [p.0, p.0 + 1.0]).collect();
    ts.sort_by(f64::total_cmp);
    ts.dedup_by(|a, b| (*a - *b).abs() <= 1e-9);
    // A flat profile draws the same straight pieces as before: sample only its points.
    let flat = profile.points.first().is_some_and(|f| profile.points.iter().all(|p| (p.1 - f.1).abs() <= 1e-12 && (p.2 - f.2).abs() <= 1e-12));
    let samples = if flat { 1 } else { SAMPLES.min((4096 / ts.len().max(1)).max(1)) };
    let mut marks = Vec::with_capacity(ts.len() * samples);
    for w in ts.windows(2) {
        for k in 0..samples {
            marks.push(w[0] + (w[1] - w[0]) * k as f64 / samples as f64);
        }
    }
    if let Some(&last) = ts.last() {
        marks.push(last);
    }
    let marks: Vec<f64> = marks.into_iter().map(|t| (t - span.0) / range * total).collect();
    split_at(pts, corner, closed, &marks)
}

/// Length of a flattened subpath (see [`flatten`]).
pub(super) fn length(pts: &[Point], closed: bool) -> f64 {
    let n = pts.len();
    let seg_count = if closed { n } else { n.saturating_sub(1) };
    (0..seg_count).map(|i| pts[i].distance(pts[(i + 1) % n])).sum()
}

/// A flattened subpath (see [`flatten`]) with points added `marks` along it (distances from its
/// start; those on a point or off the subpath are ignored).
pub(super) fn split_at(pts: &[Point], corner: &[bool], closed: bool, marks: &[f64]) -> (Vec<Point>, Vec<bool>) {
    let n = pts.len();
    let seg_count = if closed { n } else { n.saturating_sub(1) };
    let mut out = (Vec::with_capacity(n + marks.len()), Vec::with_capacity(n + marks.len()));
    let mut acc = 0.0;
    for i in 0..seg_count {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let len = a.distance(b);
        out.0.push(a);
        out.1.push(corner[i]);
        let mut inner: Vec<f64> = marks.iter().copied().filter(|d| *d > acc + 1e-9 && *d < acc + len - 1e-9).collect();
        inner.sort_by(f64::total_cmp);
        inner.dedup_by(|a, b| (*a - *b).abs() <= 1e-9);
        for d in inner {
            out.0.push(a.lerp(b, (d - acc) / len));
            out.1.push(false);
        }
        acc += len;
    }
    if !closed && n > 0 {
        out.0.push(pts[n - 1]);
        out.1.push(corner[n - 1]);
    }
    out
}

/// What outlines one variable-width stroke.
struct Ctx<'a> {
    half: f64,
    profile: &'a WidthProfile,
    cap: LineCap,
    join: LineJoin,
    miter_limit: f64,
    tol: f64,
}

/// A vertex of a flattened subpath: where it is, the directions in and out, the lengths of the
/// segments either side and whether it is a corner of the path (else a smooth point).
struct Vertex {
    p: Point,
    a: Vec2,
    b: Vec2,
    la: f64,
    lb: f64,
    corner: bool,
}

impl Ctx<'_> {
    fn subpath(&self, out: &mut BezPath, pts: &[Point], corner: &[bool], closed: bool, span: (f64, f64)) {
        let (pts, corner) = densify(pts, corner, closed, self.profile, span);
        let n = pts.len();
        let seg_count = if closed { n } else { n - 1 };
        let seg_dir: Vec<Vec2> = (0..seg_count).map(|i| unit(pts[(i + 1) % n] - pts[i])).collect();
        let mut cum = vec![0.0; n + 1];
        for i in 0..seg_count {
            cum[i + 1] = cum[i] + pts[i].distance(pts[(i + 1) % n]);
        }
        let total = cum[seg_count];
        if total <= 1e-12 {
            return;
        }
        // Left and right offsets just before and just after `d` along this piece (they differ
        // at a discontinuous point).
        let width_at = |d: f64| {
            let t = span.0 + (span.1 - span.0) * d / total;
            // A closed path joins its own start and end: a profile with different widths there
            // is a step, not a stretch across the seam.
            let (b, a) = if closed && span == (0.0, 1.0) && d <= 1e-12 {
                (self.profile.around(1.0).0, self.profile.around(0.0).1)
            } else {
                self.profile.around(if t > 1.0 { t - 1.0 } else { t })
            };
            let side = |(l, r): (f64, f64)| (self.half * l.max(0.0), self.half * r.max(0.0));
            (side(b), side(a))
        };
        let mut lpts: Vec<(Point, bool)> = Vec::with_capacity(n);
        let mut rpts: Vec<(Point, bool)> = Vec::with_capacity(n);
        // Only a butt cap makes the side end at a corner; square and round caps stay tangent to it.
        let end_sharp = matches!(self.cap, LineCap::Butt);
        for (i, &p) in pts.iter().enumerate() {
            let ((wl_b, wr_b), (wl_a, wr_a)) = width_at(cum[i]);
            if !closed && (i == 0 || i == n - 1) {
                let nm = left(seg_dir[i.min(seg_count - 1)]);
                let (wl, wr) = if i == 0 { (wl_a, wr_a) } else { (wl_b, wr_b) };
                lpts.push((p + nm * wl, end_sharp));
                rpts.push((p - nm * wr, end_sharp));
                continue;
            }
            let ip = if i == 0 { seg_count - 1 } else { i - 1 };
            let v = Vertex { p, a: seg_dir[ip], b: seg_dir[i], la: cum[ip + 1] - cum[ip], lb: cum[i + 1] - cum[i], corner: corner[i] };
            // A step: the side goes straight out (or in) from the width before to the one after.
            // Both ends of that straight piece are corners: the run before it must not round
            // into the step.
            let (step_l, step_r) = (wl_a != wl_b, wr_a != wr_b);
            self.join(&mut lpts, &v, wl_b, 1.0, step_l);
            self.join(&mut rpts, &v, wr_b, -1.0, step_r);
            if step_l {
                self.join(&mut lpts, &v, wl_a, 1.0, true);
            }
            if step_r {
                self.join(&mut rpts, &v, wr_a, -1.0, true);
            }
        }
        if closed {
            polygon(out, lpts.into_iter(), self.tol);
            polygon(out, rpts.into_iter().rev(), self.tol);
            return;
        }
        let (t0, t1) = (seg_dir[0], seg_dir[seg_count - 1]);
        let ((wl0, wr0), (wl1, wr1)) = (width_at(0.0).1, width_at(total).0);
        let mut ring = lpts;
        // End cap: from the left side around the end to the right side.
        cap_points(&mut ring, pts[n - 1], left(t1), t1, wl1, wr1, self.cap, self.tol);
        ring.extend(rpts.into_iter().rev());
        // Start cap: from the right side around the start back to the left side.
        cap_points(&mut ring, pts[0], -left(t0), -t0, wr0, wl0, self.cap, self.tol);
        polygon(out, ring.into_iter(), self.tol);
    }

    /// The offset points at `v` on one side (`side` 1: left, −1: right), `w` from the path: the
    /// join on the outer side of a turn, the inner miter point (or a detour through the vertex
    /// when that would overshoot a segment) on the inner side.
    ///
    /// Each point says whether the outline is sharp there: a path corner, a miter tip, a width
    /// step or a butt cap end. Only those break the smooth fitting; a rapid width change makes
    /// the offset turn sharply, but it is not a corner and must not be fitted as one.
    fn join(&self, pts: &mut Vec<(Point, bool)>, v: &Vertex, w: f64, side: f64, step: bool) {
        let (na, nb) = (left(v.a) * side, left(v.b) * side);
        let mid = na + nb;
        // Cosine of half the turn; the miter point is `w / c` out along `mid`.
        let c = mid.hypot() / 2.0;
        if w <= 0.0 {
            pts.push((v.p, true));
            return;
        }
        let miter = if c > 1e-9 { Some(v.p + mid * (w / (2.0 * c * c))) } else { None };
        if v.b.dot(na) <= 0.0 {
            // Outer side: the path turns away from it.
            if !v.corner {
                // A smooth point has no join: mitering its flattened segments is the offset
                // itself. A round join here would dent the edge by the turn's excess at every
                // sample, which shows as scallops along a wide curve.
                match miter {
                    Some(m) => pts.push((m, step)),
                    None => self.arc(pts, v, na, nb, w, step),
                }
            } else {
                let excess = if c > 1e-9 { w * (1.0 / c - 1.0) } else { f64::INFINITY };
                match self.join {
                    _ if excess <= self.tol => pts.push((miter.unwrap_or(v.p), true)),
                    LineJoin::Miter if c > 1e-9 && 1.0 / c <= self.miter_limit => pts.push((miter.unwrap_or(v.p), true)),
                    LineJoin::Miter | LineJoin::Bevel => pts.extend([(v.p + na * w, true), (v.p + nb * w, true)]),
                    _ => self.arc(pts, v, na, nb, w, step),
                }
            }
        } else if c <= 1e-9 {
            // A reversal has no inner miter; go through the vertex. That is a notch in the
            // outline, so it stays sharp: fitting a curve through it would loop.
            pts.extend([(v.p + na * w, true), (v.p, true), (v.p + nb * w, true)]);
        } else if !v.corner || w * (1.0 - c * c).sqrt() / c <= v.la.min(v.lb) {
            // A smooth point's inner offset is a miter too, however short its neighbours are: a
            // sample added right beside a vertex must not turn it into a detour to the centre.
            pts.push((miter.unwrap_or(v.p), step || v.corner));
        } else {
            // The inner offset of a corner cuts through the vertex; keep that notch sharp.
            pts.extend([(v.p + na * w, true), (v.p, true), (v.p + nb * w, true)]);
        }
    }

    /// A round join from `na` to `nb` (unit normals) at radius `w` around `v`; a full reversal
    /// rounds through the direction of travel. `sharp` marks a step at the join's ends.
    fn arc(&self, pts: &mut Vec<(Point, bool)>, v: &Vertex, na: Vec2, nb: Vec2, w: f64, sharp: bool) {
        let phi =
            if (na + nb).hypot() < 1e-9 { if Vec2::new(-na.y, na.x).dot(v.a) > 0.0 { PI } else { -PI } } else { na.cross(nb).atan2(na.dot(nb)) };
        let step = 2.0 * (1.0 - (self.tol / w).min(1.0)).acos();
        let steps = ((phi.abs() / step.max(1e-3)).ceil() as usize).clamp(1, 256);
        for k in 0..=steps {
            let (s, c) = (phi * k as f64 / steps as f64).sin_cos();
            pts.push((v.p + Vec2::new(na.x * c - na.y * s, na.x * s + na.y * c) * w, sharp));
        }
    }
}

/// Cap points between `c + u·r0` and `c − u·r1`, bulging towards `v`.
#[allow(clippy::too_many_arguments)]
fn cap_points(ring: &mut Vec<(Point, bool)>, c: Point, u: Vec2, v: Vec2, r0: f64, r1: f64, cap: LineCap, tol: f64) {
    match cap {
        LineCap::Butt => {}
        LineCap::Square => {
            let h = (r0 + r1) / 2.0;
            ring.push((c + u * r0 + v * h, true));
            ring.push((c - u * r1 + v * h, true));
        }
        LineCap::Round => {
            // Enough steps that the cap's facets stay within `tol`, as a round join does.
            let r = r0.max(r1).max(1e-9);
            let step = 2.0 * (1.0 - (tol / r).min(1.0)).acos();
            let steps = ((PI / step.max(1e-6)).ceil() as usize).clamp(8, 256);
            for k in 1..steps {
                let a = PI * k as f64 / steps as f64;
                let r = r0 + (r1 - r0) * (k as f64 / steps as f64);
                ring.push((c + u * (a.cos() * r) + v * (a.sin() * r), false));
            }
        }
    }
}

/// Append `pts` as a closed contour (a width outline's ring or loop), through a smooth run's
/// points as cubic curves. Sampled as line segments, a curved, wide stroke shows the facets of
/// its offset whenever it is magnified; the cubics keep it smooth. Every point is interpolated
/// (so the sampled width is kept), a run whose points are collinear stays a line, and the points
/// marked sharp — caps, miter joins, width steps and path corners — stay sharp. A rapid width
/// change turns the offset sharply without being a corner: marking corners here rather than
/// measuring turns keeps that turn smooth.
fn polygon(out: &mut BezPath, pts: impl Iterator<Item = (Point, bool)>, tol: f64) {
    let mut ring: Vec<(Point, bool)> = Vec::new();
    for (p, sharp) in pts {
        if !p.x.is_finite() || !p.y.is_finite() {
            return;
        }
        if let Some(last) = ring.last_mut()
            && last.0.distance(p) <= 1e-9
        {
            last.1 |= sharp;
            continue;
        }
        ring.push((p, sharp));
    }
    if ring.len() > 1
        && ring[0].0.distance(ring[ring.len() - 1].0) <= 1e-9
        && let Some(last) = ring.pop()
    {
        ring[0].1 |= last.1;
    }
    let n = ring.len();
    if n == 0 {
        return;
    }
    if n < 3 {
        out.move_to(ring[0].0);
        if n == 2 {
            out.line_to(ring[1].0);
        }
        out.close_path();
        return;
    }
    // Start at a sharp point when there is one, so no run has to wrap: `smooth_run` starts from
    // whichever point is current, and starting anywhere else would bridge over the skipped arc.
    match ring.iter().position(|&(_, sharp)| sharp) {
        None => {
            // A smooth closed loop: fit across the seam, so it doesn't start with a kink.
            let pts: Vec<Point> = ring.iter().map(|&(p, _)| p).collect();
            out.move_to(pts[0]);
            smooth_run(out, &pts, true, tol);
        }
        Some(first) => {
            out.move_to(ring[first].0);
            // From each sharp point to the next; the sharp points themselves stay sharp.
            let mut i = first;
            loop {
                let mut j = (i + 1) % n;
                while !ring[j].1 {
                    j = (j + 1) % n;
                }
                // A contour with a single sharp point comes back to it: the run is the whole
                // loop, fitted non-cyclically so that point stays sharp.
                let steps = if j == i { n } else { (j + n - i) % n };
                if steps == 1 {
                    out.line_to(ring[j].0);
                } else {
                    let run: Vec<Point> = (0..=steps).map(|k| ring[(i + k) % n].0).collect();
                    smooth_run(out, &run, false, tol);
                }
                if j == first {
                    break;
                }
                i = j;
            }
        }
    }
    out.close_path();
}

/// The tangent at `run[i]`: the angle bisector of its two segments, as long as their harmonic
/// mean (equal to a centered difference for evenly spaced points, and bounded by the shorter
/// segment so uneven samples can't make the curve overshoot). At an end, the one segment it has.
fn tangent(run: &[Point], i: usize, cyclic: bool) -> Vec2 {
    let n = run.len();
    let (prev, next) = if cyclic { (run[(i + n - 1) % n], run[(i + 1) % n]) } else { (run[i.saturating_sub(1)], run[(i + 1).min(n - 1)]) };
    let (a, b) = (run[i] - prev, next - run[i]);
    let (la, lb) = (a.hypot(), b.hypot());
    if la <= 1e-12 {
        return b;
    }
    if lb <= 1e-12 {
        return a;
    }
    let d = a / la + b / lb;
    let dl = d.hypot();
    if dl <= 1e-12 {
        return a;
    }
    d * (2.0 * la * lb / ((la + lb) * dl))
}

/// Append a run of points as cubics through them (or one line when they are collinear).
fn smooth_run(out: &mut BezPath, run: &[Point], cyclic: bool, tol: f64) {
    let n = run.len();
    if n < 2 {
        return;
    }
    if !cyclic && is_straight(run, tol) {
        out.line_to(run[n - 1]);
        return;
    }
    let segs = if cyclic { n } else { n - 1 };
    for i in 0..segs {
        let j = (i + 1) % n;
        let (m0, m1) = (tangent(run, i, cyclic), tangent(run, j, cyclic));
        if m0.hypot() <= 1e-12 || m1.hypot() <= 1e-12 {
            out.line_to(run[j]);
        } else {
            out.curve_to(run[i] + m0 / 3.0, run[j] - m1 / 3.0, run[j]);
        }
    }
}

/// Are the run's points within the flattening tolerance of its chord?
fn is_straight(run: &[Point], tol: f64) -> bool {
    let (a, b) = (run[0], run[run.len() - 1]);
    let d = b - a;
    let l = d.hypot();
    if l <= 1e-12 {
        return false;
    }
    run.iter().all(|p| (d.cross(*p - a) / l).abs() <= tol.max(1e-6) * 0.25)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Shape;

    fn line() -> BezPath {
        let mut b = BezPath::new();
        b.move_to((0.0, 0.0));
        b.line_to((100.0, 0.0));
        b
    }

    fn area(b: &BezPath) -> f64 {
        b.area().abs()
    }

    fn capped(cap: LineCap) -> StrokeLayer {
        let mut st = StrokeLayer::new(vectorcraft_color::Paint::None, 10.0);
        st.cap = cap;
        st
    }

    #[test]
    fn uniform_profile_matches_a_plain_stroke() {
        let p = WidthProfile { points: vec![(0.0, 1.0, 1.0), (1.0, 1.0, 1.0)] };
        assert!((area(&width_outline(&line(), 10.0, &p, &capped(LineCap::Butt), 0.01)) - 1000.0).abs() < 1e-6);
        let round = area(&width_outline(&line(), 10.0, &p, &capped(LineCap::Round), 0.01));
        let want = 1000.0 + std::f64::consts::PI * 25.0;
        assert!((round - want).abs() / want < 0.01, "{round} vs {want}");
        assert!((area(&width_outline(&line(), 10.0, &p, &capped(LineCap::Square), 0.01)) - 1100.0).abs() < 1e-6);
    }

    /// The lens eases up to full width and back down (a smooth bump averaging half the width); a
    /// one-sided profile keeps its side at zero and stays on the other.
    #[test]
    fn lens_profile_is_a_smooth_bump_and_one_sided_profiles_stay_on_their_side() {
        let a = area(&width_outline(&line(), 10.0, &WidthProfile::lens(), &capped(LineCap::Butt), 0.01));
        assert!((a - 500.0).abs() < 0.1, "{a}");
        let one = WidthProfile { points: vec![(0.0, 1.0, 0.0), (1.0, 1.0, 0.0)] };
        let o = width_outline(&line(), 10.0, &one, &capped(LineCap::Butt), 0.01);
        assert!((area(&o) - 500.0).abs() < 1e-6);
        let bb = o.bounding_box();
        // Travelling +x, the left side is up (negative y).
        assert!(bb.y0 < -4.99 && bb.y1 <= 1e-9, "{bb:?}");
    }

    #[test]
    fn a_discontinuous_point_steps_the_width() {
        // 5 pt either side for the first half, 10 pt for the second: a step at the middle.
        let p = WidthProfile { points: vec![(0.0, 1.0, 1.0), (0.5, 1.0, 1.0), (0.5, 2.0, 2.0), (1.0, 2.0, 2.0)] };
        let o = width_outline(&line(), 10.0, &p, &capped(LineCap::Butt), 0.01);
        assert!((area(&o) - 1500.0).abs() < 1e-6, "{}", area(&o));
        // No ramp: just before the middle the outline is still 5 pt from the line.
        assert!(!o.contains(kurbo::Point::new(49.0, -6.0)) && o.contains(kurbo::Point::new(51.0, -6.0)));
        // A step at a corner of a polyline renders too.
        let mut l = line();
        l.line_to((100.0, 100.0));
        let a = area(&width_outline(&l, 10.0, &p, &capped(LineCap::Butt), 0.01));
        assert!(a.is_finite() && (a - 3000.0).abs() < 150.0, "{a}");
    }

    #[test]
    fn a_closed_path_steps_where_its_profile_ends_differ() {
        let mut sq = BezPath::new();
        sq.move_to((0.0, 0.0));
        sq.line_to((100.0, 0.0));
        sq.line_to((100.0, 100.0));
        sq.line_to((0.0, 100.0));
        sq.close_path();
        // 5 pt at the seam's start, 10 pt at its end: the ring must step between them, not
        // stretch the last width point round the corner.
        let p = WidthProfile { points: vec![(0.0, 1.0, 1.0), (1.0, 2.0, 2.0)] };
        let o = width_outline(&sq, 10.0, &p, &capped(LineCap::Butt), 0.01);
        let at = |p: Point, x: f64, y: f64| (p.x - x).abs() < 1e-9 && (p.y - y).abs() < 1e-9;
        let step = o.elements().windows(2).any(|w| match (w[0], w[1]) {
            (PathEl::MoveTo(a), PathEl::LineTo(b)) | (PathEl::LineTo(a), PathEl::LineTo(b)) => {
                at(a, -10.0, -10.0) && at(b, -5.0, -5.0) || at(a, -5.0, -5.0) && at(b, -10.0, -10.0)
            }
            _ => false,
        });
        assert!(step, "{o:?}");
    }

    #[test]
    fn a_wiggly_profile_on_a_curve_fits_without_kinks() {
        // A rapid width change turns the offset sharply; that is not a path corner, so the
        // outline must stay tangent-continuous through it (the old turn-angle test did not).
        let c = kurbo::Circle::new((0.0, 0.0), 50.0).to_path(1e-3);
        let p = WidthProfile {
            points: vec![(0.0, 1.0, 1.0), (0.15, 3.0, 0.4), (0.3, 0.2, 2.8), (0.5, 2.5, 0.3), (0.7, 0.4, 2.2), (0.85, 2.0, 0.5), (1.0, 1.0, 1.0)],
        };
        let o = width_outline(&c, 10.0, &p, &capped(LineCap::Butt), 0.01);
        let mut cursor = None;
        let mut prev: Option<Vec2> = None;
        let mut kinks = 0;
        let mut check = |a: Vec2, b: Vec2| {
            let (la, lb) = (a.hypot(), b.hypot());
            if la > 1e-9 && lb > 1e-9 && (a.dot(b) <= 0.0 || a.cross(b).abs() / (la * lb) > 1e-6) {
                kinks += 1;
            }
        };
        for el in o.elements() {
            match el {
                PathEl::MoveTo(p) => {
                    cursor = Some(*p);
                    prev = None;
                }
                PathEl::LineTo(p) => {
                    if let Some(q) = cursor {
                        let d = *p - q;
                        if let Some(a) = prev {
                            check(a, d);
                        }
                        prev = Some(d);
                    }
                    cursor = Some(*p);
                }
                PathEl::CurveTo(a, b, p) => {
                    if let Some(q) = cursor {
                        let d = if a.distance(q) > 1e-9 { *a - q } else { *b - q };
                        if let Some(prev) = prev {
                            check(prev, d);
                        }
                        prev = Some(if p.distance(*b) > 1e-9 { *p - *b } else { *p - *a });
                    }
                    cursor = Some(*p);
                }
                PathEl::QuadTo(_, _) | PathEl::ClosePath => {}
            }
        }
        assert_eq!(kinks, 0, "{o:?}");
    }

    #[test]
    fn closed_paths_make_a_ring() {
        let mut sq = BezPath::new();
        sq.move_to((0.0, 0.0));
        sq.line_to((100.0, 0.0));
        sq.line_to((100.0, 100.0));
        sq.line_to((0.0, 100.0));
        sq.close_path();
        let p = WidthProfile { points: vec![(0.0, 1.0, 1.0), (1.0, 1.0, 1.0)] };
        let a = area(&width_outline(&sq, 10.0, &p, &capped(LineCap::Butt), 0.01));
        assert!((a - (110.0 * 110.0 - 90.0 * 90.0)).abs() < 1e-6, "{a}");
    }

    #[test]
    fn curves_are_offset_by_the_profile() {
        // A circle of radius 50 with a uniform 10 pt stroke: ring area 2π·50·10.
        let c = kurbo::Circle::new((0.0, 0.0), 50.0).to_path(1e-4);
        let p = WidthProfile { points: vec![(0.0, 1.0, 1.0), (1.0, 1.0, 1.0)] };
        let a = area(&width_outline(&c, 10.0, &p, &capped(LineCap::Butt), 0.001));
        let want = 2.0 * std::f64::consts::PI * 50.0 * 10.0;
        assert!((a - want).abs() / want < 0.01, "{a} vs {want}");
    }
}
