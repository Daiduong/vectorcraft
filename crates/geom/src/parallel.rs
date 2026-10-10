//! Flattened offset curves used by parallel line styles.

use kurbo::{BezPath, PathEl, Point, Vec2};

const MAX_FLAT_ELEMENTS: usize = 1_000_000;

/// Approximate a parallel curve at signed `offset` points from `path`.
/// Positive offsets follow the left-hand normal of each subpath; negative offsets follow the right.
/// Corners use a bounded miter and bevel fallback, preventing untrusted acute corners from
/// producing unbounded coordinates. `None` means invalid input or geometry exceeding the cap.
pub fn parallel_path(path: &BezPath, offset: f64, tol: f64, miter_limit: f64) -> Option<BezPath> {
    if !offset.is_finite()
        || offset.abs() > 1_000_000.0
        || !tol.is_finite()
        || !miter_limit.is_finite()
        || path.elements().len() > MAX_FLAT_ELEMENTS
        || path.elements().iter().any(|el| match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => !finite(*p),
            PathEl::QuadTo(a, b) => !finite(*a) || !finite(*b),
            PathEl::CurveTo(a, b, c) => !finite(*a) || !finite(*b) || !finite(*c),
            PathEl::ClosePath => false,
        })
    {
        return None;
    }
    if offset.abs() <= 1e-12 {
        return Some(path.clone());
    }
    let coordinate_scale = path.elements().iter().fold(1.0_f64, |scale, el| {
        let point_scale = |p: Point| p.x.abs().max(p.y.abs());
        scale.max(match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => point_scale(*p),
            PathEl::QuadTo(a, b) => point_scale(*a).max(point_scale(*b)),
            PathEl::CurveTo(a, b, c) => point_scale(*a).max(point_scale(*b)).max(point_scale(*c)),
            PathEl::ClosePath => 0.0,
        })
    });
    let flat_tol = tol.max(1e-3).max(coordinate_scale / 50_000.0);
    let mut flat = Vec::new();
    kurbo::flatten(path.iter(), flat_tol, |el| {
        if flat.len() < MAX_FLAT_ELEMENTS {
            flat.push(el);
        }
    });
    if flat.len() >= MAX_FLAT_ELEMENTS {
        return None;
    }

    let mut out = BezPath::new();
    let mut points = Vec::new();
    let mut closed = false;
    for el in flat {
        match el {
            PathEl::MoveTo(p) => {
                append_subpath(&mut out, &mut points, closed, offset, miter_limit);
                points.push(p);
                closed = false;
            }
            PathEl::LineTo(p) => points.push(p),
            PathEl::ClosePath => closed = true,
            _ => {}
        }
    }
    append_subpath(&mut out, &mut points, closed, offset, miter_limit);
    Some(out)
}

fn append_subpath(out: &mut BezPath, points: &mut Vec<Point>, closed: bool, offset: f64, miter_limit: f64) {
    points.dedup_by(|b, a| a.distance(*b) <= 1e-10);
    if closed && points.len() > 1 && points.first().zip(points.last()).is_some_and(|(first, last)| first.distance(*last) <= 1e-10) {
        points.pop();
    }
    if points.len() < 2 {
        points.clear();
        return;
    }
    let n = points.len();
    let count = if closed { n } else { n - 1 };
    if count == 0 {
        points.clear();
        return;
    }
    let tangents: Vec<Vec2> = (0..count).filter_map(|i| Some(unit(*points.get((i + 1) % n)? - *points.get(i)?))).collect();
    let normal = |v: Vec2| Vec2::new(v.y, -v.x) * offset;
    let mut shifted = Vec::with_capacity(n + 4);
    for i in 0..n {
        let Some(point) = points.get(i).copied() else { continue };
        if !closed && (i == 0 || i + 1 == n) {
            let t = if i == 0 { tangents.first().copied() } else { tangents.get(count.saturating_sub(1)).copied() };
            if let Some(t) = t {
                shifted.push(point + normal(t));
            }
            continue;
        }
        let Some(prev) = tangents.get((i + count - 1) % count).copied() else { continue };
        let Some(next) = tangents.get(i % count).copied() else { continue };
        let a = normal(prev);
        let b = normal(next);
        let bisector = unit(a + b);
        let next_normal = Vec2::new(next.y, -next.x);
        let denom = bisector.dot(next_normal);
        let length = if denom.abs() > 1e-9 { offset / denom } else { offset };
        let limit = offset.abs() * miter_limit.clamp(1.0, 1000.0);
        if length.is_finite() && length.abs() <= limit {
            shifted.push(point + bisector * length);
        } else {
            shifted.push(point + a);
            shifted.push(point + b);
        }
    }
    if shifted.iter().any(|p| !finite(*p)) {
        points.clear();
        return;
    }
    if let Some(first) = shifted.first().copied() {
        out.move_to(first);
        for p in shifted.into_iter().skip(1) {
            out.line_to(p);
        }
        if closed {
            out.close_path();
        }
    }
    points.clear();
}

fn finite(p: Point) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

fn unit(v: Vec2) -> Vec2 {
    let h = v.hypot();
    if h.is_finite() && h > 1e-12 { v / h } else { Vec2::new(1.0, 0.0) }
}

#[cfg(test)]
mod tests {
    use kurbo::Shape;

    use super::*;

    #[test]
    fn offsets_open_paths_to_the_signed_side() {
        let line = kurbo::Line::new((0.0, 0.0), (10.0, 0.0)).to_path(0.1);
        let left = parallel_path(&line, 2.0, 0.01, 10.0).unwrap();
        let right = parallel_path(&line, -2.0, 0.01, 10.0).unwrap();
        assert!((left.bounding_box().y0 + 2.0).abs() < 1e-9);
        assert!((right.bounding_box().y0 - 2.0).abs() < 1e-9);
    }

    #[test]
    fn preserves_subpaths_and_closedness() {
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((10.0, 0.0));
        p.line_to((10.0, 10.0));
        p.close_path();
        p.move_to((20.0, 0.0));
        p.line_to((30.0, 0.0));
        let shifted = parallel_path(&p, 1.0, 0.01, 4.0).unwrap();
        assert_eq!(shifted.elements().iter().filter(|e| matches!(e, PathEl::MoveTo(_))).count(), 2);
        assert_eq!(shifted.elements().iter().filter(|e| matches!(e, PathEl::ClosePath)).count(), 1);
    }

    #[test]
    fn rejects_non_finite_offsets() {
        assert!(parallel_path(&BezPath::new(), f64::NAN, 0.1, 10.0).is_none());
    }
}
