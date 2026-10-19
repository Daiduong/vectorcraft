//! Bounding-box handle geometry shared by the Selection tool and the UI.

use vectorcraft_geom::{Affine, Point, Rect, Vec2};

/// The 8 handles, clockwise from top-left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl Handle {
    pub const ALL: [Handle; 8] =
        [Handle::TopLeft, Handle::Top, Handle::TopRight, Handle::Right, Handle::BottomRight, Handle::Bottom, Handle::BottomLeft, Handle::Left];

    pub fn pos(self, r: Rect) -> Point {
        let c = r.center();
        match self {
            Handle::TopLeft => Point::new(r.x0, r.y0),
            Handle::Top => Point::new(c.x, r.y0),
            Handle::TopRight => Point::new(r.x1, r.y0),
            Handle::Right => Point::new(r.x1, c.y),
            Handle::BottomRight => Point::new(r.x1, r.y1),
            Handle::Bottom => Point::new(c.x, r.y1),
            Handle::BottomLeft => Point::new(r.x0, r.y1),
            Handle::Left => Point::new(r.x0, c.y),
        }
    }
    pub fn opposite(self) -> Handle {
        Handle::ALL[(self as usize + 4) % 8]
    }
    pub fn is_corner(self) -> bool {
        (self as usize).is_multiple_of(2)
    }
    /// Which axes this handle scales (x, y).
    pub fn axes(self) -> (bool, bool) {
        match self {
            Handle::Top | Handle::Bottom => (false, true),
            Handle::Left | Handle::Right => (true, false),
            _ => (true, true),
        }
    }
}

/// Handle under `p` (within `tol` document units).
pub fn hit_handle(r: Rect, p: Point, tol: f64) -> Option<Handle> {
    Handle::ALL.into_iter().find(|h| {
        let q = h.pos(r);
        (q.x - p.x).abs() <= tol && (q.y - p.y).abs() <= tol
    })
}

/// Is `p` in the rotate zone just outside a corner (Illustrator shows the curved rotate cursor there)?
pub fn in_rotate_zone(r: Rect, p: Point, inner: f64, outer: f64) -> Option<Handle> {
    [Handle::TopLeft, Handle::TopRight, Handle::BottomRight, Handle::BottomLeft].into_iter().find(|h| {
        let q = h.pos(r);
        let d = p.distance(q);
        d > inner && d <= outer && !r.contains(p)
    })
}

/// Scale transform for dragging `handle` of `r` to `p`.
/// `proportional` (Shift) keeps the aspect ratio; `from_center` (Alt) scales about the centre.
pub fn scale_for_drag(r: Rect, handle: Handle, p: Point, proportional: bool, from_center: bool) -> Affine {
    let origin = if from_center { r.center() } else { handle.opposite().pos(r) };
    let start = handle.pos(r);
    let (ax, ay) = handle.axes();
    let d0 = start - origin;
    let d1 = p - origin;
    let mut sx = if ax && d0.x.abs() > 1e-9 { d1.x / d0.x } else { 1.0 };
    let mut sy = if ay && d0.y.abs() > 1e-9 { d1.y / d0.y } else { 1.0 };
    if proportional {
        if handle.is_corner() {
            let s = if sx.abs() > sy.abs() { sx } else { sy };
            sx = s.abs() * sx.signum();
            sy = s.abs() * sy.signum();
        } else if ax {
            sy = sx.abs();
        } else {
            sx = sy.abs();
        }
    }
    // Avoid degenerate zero scales (Illustrator never collapses an object completely).
    let clamp = |s: f64| if s.abs() < 1e-4 { 1e-4 * if s < 0.0 { -1.0 } else { 1.0 } } else { s };
    Affine::translate(origin.to_vec2()) * Affine::scale_non_uniform(clamp(sx), clamp(sy)) * Affine::translate(-origin.to_vec2())
}

/// Rotation about `center` taking `from` to `to`, and its readout in degrees (counter-clockwise as
/// seen on the y-down page). `snap` (Shift) constrains the angle turned, as Illustrator does, to 45°
/// steps from `base_deg` (Preferences › General › Constrain Angle): the rotation lands on
/// `base_deg + k·45°` wherever the drag started, not on the pointer's direction.
pub fn rotate_for_drag(center: Point, from: Point, to: Point, snap: bool, base_deg: f64) -> (Affine, f64) {
    let a0 = (from - center).atan2();
    let a1 = (to - center).atan2();
    let mut deg = -(a1 - a0).to_degrees();
    if snap {
        deg = base_deg + ((deg - base_deg) / 45.0).round() * 45.0;
    }
    let a = -deg.to_radians();
    (Affine::translate(center.to_vec2()) * Affine::rotate(a) * Affine::translate(-center.to_vec2()), deg)
}

/// Move constrained to 45° multiples when `shift`.
pub fn move_delta(from: Point, to: Point, shift: bool) -> Vec2 {
    let d = to - from;
    if shift { vectorcraft_geom::constrain_angle(d, 45.0) } else { d }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: Rect = Rect::new(0.0, 0.0, 100.0, 50.0);

    #[test]
    fn handle_positions() {
        assert_eq!(Handle::Right.pos(R), Point::new(100.0, 25.0));
        assert_eq!(Handle::TopLeft.opposite(), Handle::BottomRight);
        assert_eq!(Handle::Top.opposite(), Handle::Bottom);
        assert_eq!(hit_handle(R, Point::new(99.0, 24.0), 3.0), Some(Handle::Right));
        assert_eq!(hit_handle(R, Point::new(50.0, 25.0), 3.0), None);
    }

    #[test]
    fn scale_corner_free_and_proportional() {
        let a = scale_for_drag(R, Handle::BottomRight, Point::new(200.0, 50.0), false, false);
        assert_eq!(a.transform_rect_bbox(R), Rect::new(0.0, 0.0, 200.0, 50.0));
        let a = scale_for_drag(R, Handle::BottomRight, Point::new(200.0, 50.0), true, false);
        assert_eq!(a.transform_rect_bbox(R), Rect::new(0.0, 0.0, 200.0, 100.0));
    }

    #[test]
    fn scale_side_and_center() {
        let a = scale_for_drag(R, Handle::Right, Point::new(150.0, 999.0), false, false);
        assert_eq!(a.transform_rect_bbox(R), Rect::new(0.0, 0.0, 150.0, 50.0));
        let a = scale_for_drag(R, Handle::Right, Point::new(150.0, 25.0), false, true);
        assert_eq!(a.transform_rect_bbox(R), Rect::new(-50.0, 0.0, 150.0, 50.0));
    }

    #[test]
    fn rotate_snaps() {
        let (_, deg) = rotate_for_drag(Point::ZERO, Point::new(10.0, 0.0), Point::new(10.0, 9.0), true, 0.0);
        assert!((deg + 45.0).abs() < 1e-9);
        let (_, deg) = rotate_for_drag(Point::ZERO, Point::new(10.0, 0.0), Point::new(10.0, 9.0), true, 30.0);
        assert!((deg + 60.0).abs() < 1e-9, "the angle turned snaps to -60° (30° - 2·45°) from a 30° Constrain Angle");
        assert!(in_rotate_zone(R, Point::new(-8.0, -8.0), 4.0, 20.0).is_some());
        assert!(in_rotate_zone(R, Point::new(5.0, 5.0), 4.0, 20.0).is_none());
    }

    #[test]
    fn rotate_snap_steps_the_angle_turned_from_a_drag_started_off_a_snap_line() {
        // The drag starts at 20° (not on a 45° line) and turns 42°: Shift lands the rotation on 45°,
        // not on the pointer's 45° direction (which would have been a 25° turn).
        let at = |deg: f64| {
            let r = deg.to_radians();
            Point::new(10.0 * r.cos(), 10.0 * r.sin())
        };
        let (a, deg) = rotate_for_drag(Point::ZERO, at(20.0), at(62.0), true, 0.0);
        assert!((deg + 45.0).abs() < 1e-9, "{deg}");
        assert!((a * at(20.0)).distance(at(65.0)) < 1e-9, "the grabbed point turns by exactly 45°");
        let (_, deg) = rotate_for_drag(Point::ZERO, at(20.0), at(62.0), true, 30.0);
        assert!((deg + 60.0).abs() < 1e-9, "multiples of 45° offset by a 30° Constrain Angle: {deg}");
        let (_, deg) = rotate_for_drag(Point::ZERO, at(20.0), at(62.0), false, 30.0);
        assert!((deg + 42.0).abs() < 1e-9, "without Shift the turn is free: {deg}");
        // A 40° turn across the ±180° seam still lands on a 45° step (-45° ≡ 315°).
        let (_, deg) = rotate_for_drag(Point::ZERO, at(170.0), at(-150.0), true, 0.0);
        assert!((deg.rem_euclid(360.0) - 315.0).abs() < 1e-9, "{deg}");
    }
}
