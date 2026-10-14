//! Equations (Object → Insert Equation…): a group whose art is the outlines of a typeset LaTeX
//! formula, kept with its source so the formula can be edited again.

use serde::{Deserialize, Serialize};
use vectorcraft_geom::Rect;

/// The font size of a new equation's base style, in points.
pub const DEFAULT_MATH_SIZE: f64 = 24.0;

/// What an equation group was made from. Its children (one filled path per glyph or rule) are
/// generated from it, so they carry their own paints and the spec keeps none.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MathSpec {
    /// The LaTeX math source.
    pub source: String,
    /// Font size of the formula's base style, in points.
    pub size: f64,
    /// Display style (fractions and big operators set large) rather than text style.
    pub display: bool,
    /// The formula's ink box at its unscaled size, as last generated and centred where the group
    /// stood then. The group's bounds against it give the scale the user has applied since (a move
    /// or a resize on the canvas), so an edit keeps it; its position is not kept up to date.
    pub bounds: Rect,
}

impl Default for MathSpec {
    fn default() -> Self {
        Self { source: String::new(), size: DEFAULT_MATH_SIZE, display: true, bounds: Rect::ZERO }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spec_reads_with_defaults_for_what_a_file_lacks_and_writes_camel_case() {
        let s: MathSpec = serde_json::from_str(r#"{"source":"x^2"}"#).unwrap();
        assert_eq!((s.source.as_str(), s.size, s.display, s.bounds), ("x^2", DEFAULT_MATH_SIZE, true, Rect::ZERO));
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["source"], "x^2");
        assert!(v.get("size").is_some() && v.get("display").is_some() && v.get("bounds").is_some());
        assert_eq!(serde_json::from_value::<MathSpec>(v).unwrap(), s);
        assert_eq!(serde_json::from_str::<MathSpec>("{}").unwrap(), MathSpec::default());
    }
}
