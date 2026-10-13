//! The canvas a document draws on (#1087): a standard canvas, bounded and laid out on artboards,
//! or an infinite canvas, an endless sheet with no artboards by default that pans without edges
//! and zooms with no fixed stops.
//!
//! Coordinates are absolute `f64` points, so "infinite" has a practical end: art may lie up to
//! [`MAX_COORD_INFINITE`] from the origin (about 350 km), where `f64` still resolves about 1e-7 pt.
//! The text layout and the PDF and EMF writers stop at the same distance. The zoom of an infinite
//! canvas is limited only by that precision ([`CanvasKind::zoom_range`]).

use serde::{Deserialize, Deserializer, Serialize};

/// Coordinates of a standard canvas stay within this distance of the origin (points): about
/// 1,400 m, far past the large canvas of other vector apps.
pub const MAX_COORD_STANDARD: f64 = 4.0e6;

/// Coordinates of an infinite canvas stay within this distance of the origin (points): about
/// 350 km. It is also the limit of text layout and of PDF and EMF export, and the hard limit for
/// code that checks coordinates without a document.
pub const MAX_COORD_INFINITE: f64 = 1.0e9;

/// The zoom range of a standard canvas (a factor: 3.13% to 64,000%).
pub const ZOOM_STANDARD: (f64, f64) = (0.0313, 640.0);

/// The smallest zoom of an infinite canvas: a view about 2,000 px wide then covers the whole
/// canvas.
pub const ZOOM_INFINITE_MIN: f64 = 1.0e-6;

/// The largest zoom of an infinite canvas near the origin (1e11 %). Further from the origin the
/// precision of the coordinates sets a lower limit ([`CanvasKind::zoom_range`]).
pub const ZOOM_INFINITE_MAX: f64 = 1.0e9;

/// The finest step an infinite canvas may show between two neighbouring coordinates (screen px):
/// past it, art would move in visible jumps.
const PRECISION_PX: f64 = 0.05;

/// The kind of canvas a document draws on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CanvasKind {
    /// Bounded ([`MAX_COORD_STANDARD`]) and laid out on artboards; zoom 3.13% to 64,000%.
    #[default]
    Standard,
    /// An endless sheet: no artboards by default, coordinates to [`MAX_COORD_INFINITE`], and a
    /// zoom limited only by precision.
    Infinite,
}

impl<'de> Deserialize<'de> for CanvasKind {
    /// A kind this version doesn't know (from a newer file) opens as a standard canvas, so the
    /// document still loads.
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        Ok(match v.as_str() {
            Some("infinite") => Self::Infinite,
            _ => Self::Standard,
        })
    }
}

impl CanvasKind {
    /// The name used in commands and files: "standard" or "infinite".
    pub fn key(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Infinite => "infinite",
        }
    }

    /// The kind a command names ("standard", "infinite"); `None` for any other value.
    pub fn from_key(s: &str) -> Option<Self> {
        match s {
            "standard" => Some(Self::Standard),
            "infinite" => Some(Self::Infinite),
            _ => None,
        }
    }

    pub fn is_infinite(self) -> bool {
        self == Self::Infinite
    }

    /// How far from the origin coordinates may lie (points).
    pub fn coord_limit(self) -> f64 {
        match self {
            Self::Standard => MAX_COORD_STANDARD,
            Self::Infinite => MAX_COORD_INFINITE,
        }
    }

    /// The zoom range (a factor) for a view centred at `center` that spans `extent` points from it
    /// (half the view's diagonal in document units). A standard canvas has fixed limits. An
    /// infinite canvas zooms out to [`ZOOM_INFINITE_MIN`] and in until neighbouring coordinates at
    /// the far edge of the view would lie [`PRECISION_PX`] apart on screen, at most
    /// [`ZOOM_INFINITE_MAX`].
    pub fn zoom_range(self, center: (f64, f64), extent: f64) -> (f64, f64) {
        match self {
            Self::Standard => ZOOM_STANDARD,
            Self::Infinite => {
                // `f64::max` skips a NaN, so check every input for a number first.
                let finite = center.0.is_finite() && center.1.is_finite() && extent.is_finite();
                let far = center.0.abs().max(center.1.abs()) + extent.abs();
                // The spacing of f64 values near `far` (at least that near 1).
                let ulp = if finite { far.max(1.0) * f64::EPSILON } else { f64::MAX };
                let max = (PRECISION_PX / ulp).clamp(ZOOM_STANDARD.1, ZOOM_INFINITE_MAX);
                (ZOOM_INFINITE_MIN, max)
            }
        }
    }

    /// `zoom` within [`Self::zoom_range`]; a zoom that isn't a number becomes 1.
    pub fn clamp_zoom(self, zoom: f64, center: (f64, f64), extent: f64) -> f64 {
        let (lo, hi) = self.zoom_range(center, extent);
        if zoom.is_finite() { zoom.clamp(lo, hi) } else { 1.0_f64.clamp(lo, hi) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip_and_unknown_kinds_open_as_standard() {
        for k in [CanvasKind::Standard, CanvasKind::Infinite] {
            assert_eq!(CanvasKind::from_key(k.key()), Some(k));
            let json = serde_json::to_string(&k).unwrap();
            assert_eq!(serde_json::from_str::<CanvasKind>(&json).unwrap(), k);
        }
        assert_eq!(CanvasKind::from_key("bounded"), None);
        assert_eq!(serde_json::from_str::<CanvasKind>("\"spiral\"").unwrap(), CanvasKind::Standard);
        assert_eq!(serde_json::from_str::<CanvasKind>("7").unwrap(), CanvasKind::Standard);
    }

    #[test]
    fn limits_and_zoom_ranges() {
        assert_eq!(CanvasKind::Standard.coord_limit(), 4.0e6);
        assert_eq!(CanvasKind::Infinite.coord_limit(), 1.0e9);
        assert_eq!(CanvasKind::Standard.zoom_range((1e9, 0.0), 10.0), ZOOM_STANDARD);
        // Near the origin an infinite canvas zooms in to its cap; far away, precision limits it.
        let (lo, near) = CanvasKind::Infinite.zoom_range((0.0, 0.0), 100.0);
        assert_eq!((lo, near), (ZOOM_INFINITE_MIN, ZOOM_INFINITE_MAX));
        let (_, far) = CanvasKind::Infinite.zoom_range((1e9, -5.0), 100.0);
        assert!(far < 1e6 && far > 1e5, "{far}");
        // At that zoom, neighbouring coordinates at 1e9 are about 0.05 px apart.
        assert!((1e9 * f64::EPSILON * far - PRECISION_PX).abs() < 1e-3);
        // Never below the standard maximum, even at the edge of the canvas or for junk input.
        assert_eq!(CanvasKind::Infinite.zoom_range((f64::NAN, 0.0), 1.0).1, ZOOM_STANDARD.1);
        assert_eq!(CanvasKind::Infinite.clamp_zoom(f64::INFINITY, (0.0, 0.0), 1.0), 1.0);
        assert_eq!(CanvasKind::Infinite.clamp_zoom(1e-12, (0.0, 0.0), 1.0), ZOOM_INFINITE_MIN);
        assert_eq!(CanvasKind::Standard.clamp_zoom(1e4, (0.0, 0.0), 1.0), 640.0);
    }

    /// Rulers keep a step of at least 50 screen points at any zoom an infinite canvas allows: the
    /// standard table inside its range, then 1, 2, 5 in each power of ten.
    #[test]
    fn ruler_steps_reach_every_zoom() {
        use crate::Unit;
        let pt = Unit::Points;
        assert_eq!(pt.ruler_step(1.0), 50.0);
        assert_eq!(pt.ruler_step(2.0), 25.0, "the table's own steps");
        assert_eq!(pt.ruler_step(640.0), 0.1);
        for zoom in [ZOOM_INFINITE_MIN, 1e-4, 0.0313, 1.0, 640.0, 1e5, ZOOM_INFINITE_MAX] {
            let step = pt.ruler_step(zoom);
            let px = step * zoom;
            assert!((50.0..=250.0).contains(&px), "zoom {zoom}: step {step} is {px} px");
            let lead = step / 10f64.powf(step.log10().floor());
            assert!([1.0, 2.0, 2.5, 5.0].iter().any(|m| (lead - m).abs() < 1e-6), "zoom {zoom}: step {step}");
        }
        assert_eq!(pt.ruler_step(f64::NAN), 10000.0);
        assert_eq!(pt.ruler_step(0.0), 10000.0);
    }
}
