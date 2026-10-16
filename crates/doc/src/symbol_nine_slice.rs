//! Symbol Options › Enable Guides for 9-Slice Scaling
//! ([helpx](https://helpx.adobe.com/illustrator/desktop/manage-objects/traces-mockups-symbols/maintain-proportions-while-scaling-symbols.html)).
//!
//! Two horizontal and two vertical guides divide the symbol into nine regions. When an instance
//! scales, the centre stretches in both axes, the edge strips in one axis, and the corners keep
//! their size. Default guide inset is 25% from each edge. Stored under [`SYMBOL_NINE_SLICE`] keyed
//! by symbol name; disabled (the default) is omitted from the map.

use kurbo::{Affine, Rect};
use serde_json::Value;

use crate::SYMBOL_HALF;

/// Document unknown key: `{ symbolName: { left, right, top, bottom } }` with each inset in (0, 0.5).
/// Presence means guides are enabled; values default to [`SymbolNineSlice::DEFAULT_INSET`].
pub const SYMBOL_NINE_SLICE: &str = "symbolNineSlice";

/// Default inset from each edge (25% of the symbol's width or height).
pub const DEFAULT_NINE_SLICE_INSET: f64 = 0.25;

/// Symbol Options › Enable Guides for 9-Slice Scaling and the four guide insets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SymbolNineSlice {
    pub enabled: bool,
    /// Fraction of width from the left edge, in `(0, 0.5)`.
    pub left: f64,
    /// Fraction of width from the right edge, in `(0, 0.5)`.
    pub right: f64,
    /// Fraction of height from the top edge, in `(0, 0.5)`.
    pub top: f64,
    /// Fraction of height from the bottom edge, in `(0, 0.5)`.
    pub bottom: f64,
}

impl Default for SymbolNineSlice {
    fn default() -> Self {
        Self {
            enabled: false,
            left: DEFAULT_NINE_SLICE_INSET,
            right: DEFAULT_NINE_SLICE_INSET,
            top: DEFAULT_NINE_SLICE_INSET,
            bottom: DEFAULT_NINE_SLICE_INSET,
        }
    }
}

impl SymbolNineSlice {
    /// Enabled with the documented 25% insets.
    pub fn enabled_default() -> Self {
        Self { enabled: true, ..Self::default() }
    }

    /// Clamp each inset into `(0, 0.5)` so the centre strip stays positive.
    pub fn clamped(self) -> Self {
        let clamp = |v: f64| {
            if !v.is_finite() {
                return DEFAULT_NINE_SLICE_INSET;
            }
            v.clamp(0.001, 0.499)
        };
        Self {
            enabled: self.enabled,
            left: clamp(self.left),
            right: clamp(self.right),
            top: clamp(self.top),
            bottom: clamp(self.bottom),
        }
    }

    /// True when this is the document default (nothing stored).
    pub fn is_default(self) -> bool {
        !self.enabled
    }

    pub fn from_value(v: &Value) -> Option<Self> {
        let o = v.as_object()?;
        let inset = |k: &str| o.get(k).and_then(Value::as_f64).unwrap_or(DEFAULT_NINE_SLICE_INSET);
        Some(
            Self {
                enabled: true,
                left: inset("left"),
                right: inset("right"),
                top: inset("top"),
                bottom: inset("bottom"),
            }
            .clamped(),
        )
    }

    pub fn to_value(self) -> Value {
        let s = self.clamped();
        serde_json::json!({
            "left": s.left,
            "right": s.right,
            "top": s.top,
            "bottom": s.bottom,
        })
    }
}

/// One of the nine source → destination rectangles in symbol art space (`±SYMBOL_HALF`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NineSliceCell {
    pub src: Rect,
    pub dst: Rect,
}

/// Guide lines in art space (`±SYMBOL_HALF`): left/right x and top/bottom y.
pub fn guide_lines(ns: SymbolNineSlice) -> (f64, f64, f64, f64) {
    let ns = ns.clamped();
    let w = 2.0 * SYMBOL_HALF;
    let h = 2.0 * SYMBOL_HALF;
    let x0 = -SYMBOL_HALF + ns.left * w;
    let x1 = SYMBOL_HALF - ns.right * w;
    let y0 = -SYMBOL_HALF + ns.top * h;
    let y1 = SYMBOL_HALF - ns.bottom * h;
    (x0, x1, y0, y1)
}

/// The nine source/destination cells for scaling the `±SYMBOL_HALF` art box by `(sx, sy)`
/// about the origin (corners keep their size; the centre absorbs the stretch).
///
/// Returns `None` when guides are off, the scale is effectively 1×1, or a factor is not finite.
pub fn scale_cells(ns: SymbolNineSlice, sx: f64, sy: f64) -> Option<[NineSliceCell; 9]> {
    if !ns.enabled || !sx.is_finite() || !sy.is_finite() || sx <= 0.0 || sy <= 0.0 {
        return None;
    }
    if (sx - 1.0).abs() < 1e-9 && (sy - 1.0).abs() < 1e-9 {
        return None;
    }
    let ns = ns.clamped();
    let (gx0, gx1, gy0, gy1) = guide_lines(ns);
    let src_xs = [-SYMBOL_HALF, gx0, gx1, SYMBOL_HALF];
    let src_ys = [-SYMBOL_HALF, gy0, gy1, SYMBOL_HALF];
    // Corner sizes stay; centre gets the rest of the scaled box.
    let left = gx0 - (-SYMBOL_HALF);
    let right = SYMBOL_HALF - gx1;
    let top = gy0 - (-SYMBOL_HALF);
    let bottom = SYMBOL_HALF - gy1;
    let out_w = 2.0 * SYMBOL_HALF * sx;
    let out_h = 2.0 * SYMBOL_HALF * sy;
    let mid_w = (out_w - left - right).max(0.0);
    let mid_h = (out_h - top - bottom).max(0.0);
    let dst_xs = [-out_w * 0.5, -out_w * 0.5 + left, -out_w * 0.5 + left + mid_w, out_w * 0.5];
    let dst_ys = [-out_h * 0.5, -out_h * 0.5 + top, -out_h * 0.5 + top + mid_h, out_h * 0.5];
    let mut cells = [NineSliceCell { src: Rect::ZERO, dst: Rect::ZERO }; 9];
    let mut i = 0;
    for row in 0..3 {
        for col in 0..3 {
            let Some(sx0) = src_xs.get(col).copied() else { return None };
            let Some(sx1) = src_xs.get(col + 1).copied() else { return None };
            let Some(sy0) = src_ys.get(row).copied() else { return None };
            let Some(sy1) = src_ys.get(row + 1).copied() else { return None };
            let Some(dx0) = dst_xs.get(col).copied() else { return None };
            let Some(dx1) = dst_xs.get(col + 1).copied() else { return None };
            let Some(dy0) = dst_ys.get(row).copied() else { return None };
            let Some(dy1) = dst_ys.get(row + 1).copied() else { return None };
            if sx1 - sx0 < 1e-12 || sy1 - sy0 < 1e-12 || dx1 - dx0 < 1e-12 || dy1 - dy0 < 1e-12 {
                // Degenerate strip (instance scaled below corner size): skip by mapping to empty.
                cells[i] = NineSliceCell {
                    src: Rect::new(sx0, sy0, sx0, sy0),
                    dst: Rect::new(dx0, dy0, dx0, dy0),
                };
            } else {
                cells[i] = NineSliceCell {
                    src: Rect::new(sx0, sy0, sx1, sy1),
                    dst: Rect::new(dx0, dy0, dx1, dy1),
                };
            }
            i += 1;
        }
    }
    Some(cells)
}

/// Affine that maps `src` onto `dst` (axis-aligned).
pub fn map_rect(src: Rect, dst: Rect) -> Option<Affine> {
    let sw = src.width();
    let sh = src.height();
    if !(sw.is_finite() && sh.is_finite() && sw > 1e-12 && sh > 1e-12) {
        return None;
    }
    let sx = dst.width() / sw;
    let sy = dst.height() / sh;
    if !(sx.is_finite() && sy.is_finite()) {
        return None;
    }
    Some(Affine::translate(dst.origin().to_vec2()) * Affine::scale_non_uniform(sx, sy) * Affine::translate(-src.origin().to_vec2()))
}

/// Split `xf` into translation, rotation (radians) and axis scales when there is no shear.
/// Returns `None` when the linear part is not a rotation times a positive scale.
pub fn decompose_trs(xf: Affine) -> Option<(f64, f64, f64, f64, f64)> {
    let [a, b, c, d, e, f] = xf.as_coeffs();
    if ![a, b, c, d, e, f].iter().all(|v| v.is_finite()) {
        return None;
    }
    let sx = (a * a + b * b).sqrt();
    let sy = (c * c + d * d).sqrt();
    if sx < 1e-12 || sy < 1e-12 {
        return None;
    }
    // Columns of the linear part should be orthogonal for pure rotation+scale.
    let dot = a * c + b * d;
    if dot.abs() > 1e-6 * sx * sy {
        return None;
    }
    let angle = b.atan2(a);
    Some((e, f, angle, sx, sy))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_disabled() {
        assert!(SymbolNineSlice::default().is_default());
        assert!(!SymbolNineSlice::enabled_default().is_default());
    }

    #[test]
    fn scale_cells_keep_corner_size_when_widened() {
        let ns = SymbolNineSlice::enabled_default();
        let cells = scale_cells(ns, 2.0, 1.0).expect("scaled");
        // Top-left corner: source and dest width equal the left inset of the art box.
        let corner = cells[0];
        let inset = DEFAULT_NINE_SLICE_INSET * 2.0 * SYMBOL_HALF;
        assert!((corner.src.width() - inset).abs() < 1e-9);
        assert!((corner.dst.width() - inset).abs() < 1e-9, "corners must not stretch");
        // Centre cell grows in x.
        let mid = cells[4];
        assert!(mid.dst.width() > mid.src.width());
        assert!((mid.dst.height() - mid.src.height()).abs() < 1e-9);
    }

    #[test]
    fn unit_scale_skips_cells() {
        assert!(scale_cells(SymbolNineSlice::enabled_default(), 1.0, 1.0).is_none());
        assert!(scale_cells(SymbolNineSlice::default(), 2.0, 2.0).is_none());
    }

    #[test]
    fn map_rect_scales_and_moves() {
        let a = map_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Rect::new(5.0, 5.0, 25.0, 15.0)).unwrap();
        let p = a * kurbo::Point::new(0.0, 0.0);
        assert!((p.x - 5.0).abs() < 1e-9 && (p.y - 5.0).abs() < 1e-9);
        let q = a * kurbo::Point::new(10.0, 10.0);
        assert!((q.x - 25.0).abs() < 1e-9 && (q.y - 15.0).abs() < 1e-9);
    }
}
