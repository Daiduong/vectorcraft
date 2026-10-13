//! Deep-zoom guard band (#1087): a shape that reaches far past the view (a huge rectangle seen
//! up close, a line across an infinite canvas) is cut down, in `f64` document space, to the part
//! near the view before it is drawn. The rasterizer works in `f32` device pixels, which can't
//! place points millions of pixels away to a fraction of a pixel; and flattening or stroking the
//! whole shape at screen tolerance would make millions of segments.
//!
//! The cut keeps what shows: curves near the view are flattened finely, the rest become their
//! chords (a Bézier stays inside its control box, so a chord outside the band changes nothing in
//! it), and the result is clipped to the band. A fill is clipped as polygons (Sutherland–Hodgman
//! keeps the winding number of every point inside the band, so both fill rules hold); an open
//! line is split into the pieces that cross the band. The band reaches a viewport past the
//! screen, so the edges the clip adds lie off screen.

use vectorcraft_geom::kurbo;
use vectorcraft_geom::{Affine, BezPath, CubicBez, ParamCurve, PathEl, Point, Rect};

/// A shape whose control points reach this far from the view's origin (device pixels) is cut to
/// the guard band before drawing. Below it `f32` still places points to 1/16 px.
pub const TRIGGER_PX: f64 = 1.0e6;

/// Control points this far from the origin (points) are beyond any canvas (an infinite one reaches
/// 1e9, with room for handles): only a malformed file holds them, and such a shape is left
/// undrawn rather than cut.
pub const BEYOND_ANY_CANVAS: f64 = 1.0e12;

/// Does `bp` reach beyond any canvas ([`BEYOND_ANY_CANVAS`]), or hold a coordinate that isn't a
/// number?
pub fn beyond_any_canvas(bp: &BezPath) -> bool {
    let b = bp.control_box();
    ![b.x0, b.y0, b.x1, b.y1].iter().all(|v| v.is_finite() && v.abs() <= BEYOND_ANY_CANVAS)
}

/// Deepest subdivision of one curve while flattening (2^24 pieces at most, but only near the
/// band, where pieces are a fraction of a pixel long well before that).
const MAX_DEPTH: u32 = 24;

/// Does `bp`, drawn through `view`, reach past [`TRIGGER_PX`]?
pub fn needs_clip(bp: &BezPath, view: Affine) -> bool {
    let b = view.transform_rect_bbox(bp.control_box());
    ![b.x0, b.y0, b.x1, b.y1].iter().all(|v| v.is_finite() && v.abs() <= TRIGGER_PX)
}

/// The guard band in document space: the visible region `visible` grown by its own size and by
/// `reach` (half the widest stroke, in document units) on every side.
pub fn band(visible: Rect, reach: f64) -> Rect {
    let grow = visible.width().max(visible.height()) + reach.max(0.0);
    visible.inflate(grow, grow)
}

/// `bp` flattened to polylines, one per subpath (with whether it is closed), finely (to `tol`,
/// document units) where it comes near `band` and as chords elsewhere.
fn flatten_near(bp: &BezPath, band: Rect, tol: f64) -> Vec<(Vec<Point>, bool)> {
    let tol = if tol.is_finite() && tol > 0.0 { tol } else { 1.0 };
    let mut out: Vec<(Vec<Point>, bool)> = vec![];
    let mut cur: Vec<Point> = vec![];
    let mut last = Point::ZERO;
    let flush = |cur: &mut Vec<Point>, out: &mut Vec<(Vec<Point>, bool)>, closed: bool| {
        if cur.len() > 1 {
            out.push((std::mem::take(cur), closed));
        } else {
            cur.clear();
        }
    };
    for el in bp.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                flush(&mut cur, &mut out, false);
                cur.push(p);
                last = p;
            }
            PathEl::LineTo(p) => {
                cur.push(p);
                last = p;
            }
            PathEl::QuadTo(a, b) => {
                cubic_near(CubicBez::new(last, last + (a - last) * (2.0 / 3.0), b + (a - b) * (2.0 / 3.0), b), band, tol, 0, &mut cur);
                last = b;
            }
            PathEl::CurveTo(a, b, c) => {
                cubic_near(CubicBez::new(last, a, b, c), band, tol, 0, &mut cur);
                last = c;
            }
            PathEl::ClosePath => {
                flush(&mut cur, &mut out, true);
            }
        }
    }
    flush(&mut cur, &mut out, false);
    out
}

/// Append the points after `c.p0` of cubic `c` to `out`: its chord when its control box is
/// outside `band` or it is flat to `tol`, else both halves.
fn cubic_near(c: CubicBez, band: Rect, tol: f64, depth: u32, out: &mut Vec<Point>) {
    let cb = Rect::from_points(c.p0, c.p3).union_pt(c.p1).union_pt(c.p2);
    let outside = cb.x1 < band.x0 || cb.x0 > band.x1 || cb.y1 < band.y0 || cb.y0 > band.y1;
    // The control points' distance from the chord bounds the curve's.
    let chord = kurbo::Line::new(c.p0, c.p3);
    let flat = [c.p1, c.p2].iter().all(|p| distance_to_segment(*p, chord) <= tol);
    if outside || flat || depth >= MAX_DEPTH {
        out.push(c.p3);
        return;
    }
    let (a, b) = c.subdivide();
    cubic_near(a, band, tol, depth + 1, out);
    cubic_near(b, band, tol, depth + 1, out);
}

fn distance_to_segment(p: Point, l: kurbo::Line) -> f64 {
    let d = l.p1 - l.p0;
    let len2 = d.hypot2();
    if len2 <= 0.0 {
        return p.distance(l.p0);
    }
    let t = ((p - l.p0).dot(d) / len2).clamp(0.0, 1.0);
    p.distance(l.p0 + d * t)
}

/// Clip polygon `poly` (closed) to `r` (Sutherland–Hodgman).
fn clip_polygon(poly: &[Point], r: Rect) -> Vec<Point> {
    // One pass per edge of `r`: (inside test, intersection with the edge line).
    type Inside = fn(Point, Rect) -> bool;
    type Cross = fn(Point, Point, Rect) -> Point;
    let passes: [(Inside, Cross); 4] = [
        (|p, r| p.x >= r.x0, |a, b, r| cross_x(a, b, r.x0)),
        (|p, r| p.x <= r.x1, |a, b, r| cross_x(a, b, r.x1)),
        (|p, r| p.y >= r.y0, |a, b, r| cross_y(a, b, r.y0)),
        (|p, r| p.y <= r.y1, |a, b, r| cross_y(a, b, r.y1)),
    ];
    let mut pts = poly.to_vec();
    for (inside, cross) in passes {
        if pts.is_empty() {
            break;
        }
        let mut next = Vec::with_capacity(pts.len() + 4);
        let Some(&prev0) = pts.last() else { break };
        let mut prev = prev0;
        for &p in &pts {
            match (inside(prev, r), inside(p, r)) {
                (true, true) => next.push(p),
                (true, false) => next.push(cross(prev, p, r)),
                (false, true) => {
                    next.push(cross(prev, p, r));
                    next.push(p);
                }
                (false, false) => {}
            }
            prev = p;
        }
        pts = next;
    }
    pts
}

fn cross_x(a: Point, b: Point, x: f64) -> Point {
    let t = if b.x != a.x { (x - a.x) / (b.x - a.x) } else { 0.0 };
    Point::new(x, a.y + (b.y - a.y) * t)
}

fn cross_y(a: Point, b: Point, y: f64) -> Point {
    let t = if b.y != a.y { (y - a.y) / (b.y - a.y) } else { 0.0 };
    Point::new(a.x + (b.x - a.x) * t, y)
}

/// The part of segment `a`–`b` inside `r` (Liang–Barsky), if any.
fn clip_segment(a: Point, b: Point, r: Rect) -> Option<(Point, Point)> {
    let d = b - a;
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (p, q) in [(-d.x, a.x - r.x0), (d.x, r.x1 - a.x), (-d.y, a.y - r.y0), (d.y, r.y1 - a.y)] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
    }
    (t0 <= t1).then(|| (a + d * t0, a + d * t1))
}

/// `bp` cut to `band` for a fill: each subpath closed and clipped as a polygon. `tol` is the
/// flattening tolerance in document units (a fraction of a pixel).
pub fn clip_fill(bp: &BezPath, band: Rect, tol: f64) -> BezPath {
    let mut out = BezPath::new();
    for (poly, _) in flatten_near(bp, band, tol) {
        let clipped = clip_polygon(&poly, band);
        let mut it = clipped.into_iter();
        let Some(first) = it.next() else { continue };
        out.move_to(first);
        for p in it {
            out.line_to(p);
        }
        out.close_path();
    }
    out
}

/// `bp` cut to `band` for a stroke: closed subpaths clipped as polygons (their added edges run
/// along the band, off screen), open ones split into the pieces inside the band.
pub fn clip_stroke(bp: &BezPath, band: Rect, tol: f64) -> BezPath {
    let mut out = BezPath::new();
    for (line, closed) in flatten_near(bp, band, tol) {
        if closed {
            let clipped = clip_polygon(&line, band);
            let mut it = clipped.into_iter();
            let Some(first) = it.next() else { continue };
            out.move_to(first);
            for p in it {
                out.line_to(p);
            }
            out.close_path();
            continue;
        }
        // Open: the runs of segments inside the band.
        let mut pen: Option<Point> = None;
        for w in line.windows(2) {
            let (Some(&a), Some(&b)) = (w.first(), w.get(1)) else { continue };
            match clip_segment(a, b, band) {
                Some((s, e)) => {
                    if pen != Some(s) {
                        out.move_to(s);
                    }
                    out.line_to(e);
                    pen = (e == b).then_some(e);
                }
                None => pen = None,
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_geom::Shape;

    fn area(bp: &BezPath) -> f64 {
        bp.area().abs()
    }

    /// A huge square seen through a small view: the cut keeps exactly the square's part inside the
    /// band, and nothing is left near f32's limits.
    #[test]
    fn a_huge_fill_is_cut_to_the_band() {
        let huge = Rect::new(-1.0e9, -1.0e9, 1.0e9, 1.0e9).to_path(0.1);
        let view = Rect::new(10.0, 10.0, 20.0, 20.0);
        let b = band(view, 0.0);
        assert_eq!(b, Rect::new(0.0, 0.0, 30.0, 30.0));
        let cut = clip_fill(&huge, b, 1e-3);
        assert!((area(&cut) - 900.0).abs() < 1e-6, "{}", area(&cut));
        assert!(cut.control_box().union(b) == b, "inside the band");
        // A drawing transform that zooms 1e7 × onto the view would have put the corners 1e16 px
        // away; the cut keeps them within a few view sizes.
        let zoom = Affine::scale(1.0e7) * Affine::translate((-10.0, -10.0));
        assert!(needs_clip(&huge, zoom) && !needs_clip(&cut, Affine::scale(1.0e4)));
    }

    /// A circle much larger than the view keeps its curve where it crosses the band (within the
    /// tolerance), and both fill rules keep a hole that lies in the band.
    #[test]
    fn curves_near_the_band_stay_curves_and_holes_stay_holes() {
        let r = 1.0e8;
        let circle = kurbo::Circle::new((0.0, 0.0), r).to_path(1e-3);
        // A view on the circle's rim at the top.
        let view = Rect::new(-5.0, -r - 5.0, 5.0, -r + 5.0);
        let b = band(view, 0.0);
        let cut = clip_fill(&circle, b, 1e-4);
        for el in cut.elements() {
            if let PathEl::LineTo(p) | PathEl::MoveTo(p) = el {
                let on_rim = (p.to_vec2().hypot() - r).abs() < 1e-2;
                let on_band = (p.x - b.x0).abs() < 1e-6 || (p.x - b.x1).abs() < 1e-6 || (p.y - b.y0).abs() < 1e-6 || (p.y - b.y1).abs() < 1e-6;
                assert!(on_rim || on_band, "{p:?} is on the rim or the band");
            }
        }
        // A ring: a big square with a square hole in the band.
        let mut ring = Rect::new(-1.0e9, -1.0e9, 1.0e9, 1.0e9).to_path(0.1);
        ring.extend(Rect::new(12.0, 12.0, 18.0, 18.0).to_path(0.1).iter());
        let cut = clip_fill(&ring, band(Rect::new(10.0, 10.0, 20.0, 20.0), 0.0), 1e-3);
        let hole = Point::new(15.0, 15.0);
        assert_eq!(cut.winding(hole), ring.winding(hole), "the hole stays");
        assert_eq!(cut.winding(Point::new(5.0, 5.0)), ring.winding(Point::new(5.0, 5.0)));
    }

    /// An open line across the canvas becomes the piece crossing the band.
    #[test]
    fn an_open_line_is_split_to_the_pieces_in_the_band() {
        let mut line = BezPath::new();
        line.move_to((-1.0e9, 15.0));
        line.line_to((1.0e9, 15.0));
        let cut = clip_stroke(&line, Rect::new(0.0, 0.0, 30.0, 30.0), 1e-3);
        let els: Vec<_> = cut.elements().to_vec();
        assert_eq!(els, [PathEl::MoveTo(Point::new(0.0, 15.0)), PathEl::LineTo(Point::new(30.0, 15.0))]);
        // Wholly outside: nothing.
        let mut far = BezPath::new();
        far.move_to((100.0, 100.0));
        far.line_to((200.0, 100.0));
        assert!(clip_stroke(&far, Rect::new(0.0, 0.0, 30.0, 30.0), 1e-3).elements().is_empty());
    }

    /// Junk never panics: NaN tolerance, empty paths, a degenerate band.
    #[test]
    fn odd_input_is_safe() {
        let sq = Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1);
        assert!(clip_fill(&BezPath::new(), Rect::new(0.0, 0.0, 1.0, 1.0), 0.1).elements().is_empty());
        let _ = clip_fill(&sq, Rect::new(0.0, 0.0, 1.0, 1.0), f64::NAN);
        let _ = clip_stroke(&sq, Rect::new(5.0, 5.0, 5.0, 5.0), 0.0);
        assert!(!needs_clip(&sq, Affine::IDENTITY));
        assert!(needs_clip(&sq, Affine::scale(f64::NAN)));
        // Within an infinite canvas a shape is real art; past any canvas, a malformed file's.
        assert!(!beyond_any_canvas(&Rect::new(-1.0e9, -1.0e9, 1.0e9, 1.0e9).to_path(0.1)));
        assert!(beyond_any_canvas(&Rect::new(0.0, 0.0, 3.0e20, 1.0).to_path(0.1)));
    }
}
