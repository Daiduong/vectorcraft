//! VectorCraft math: a pure-Rust LaTeX math typesetter on OpenType `MATH` tables.
//!
//! [`typeset`] reads a subset of LaTeX math ([`parser`]), sets it with the rules of TeX's
//! Appendix G driven by the constants, variants and assemblies of a math font ([`mathfont`]), and
//! returns glyph placements and rules ([`MathLayout`]) that [`MathLayout::to_paths`] turns into
//! outlines. There is no dependency on a TeX installation, and the crate builds for wasm.
//!
//! ```ignore
//! let db = FontDb::global();
//! let face = find_math_face(db).ok_or(MathError::NoMathFont)?;
//! let layout = typeset(db, &face, r"y=\sqrt{x}+\dfrac{1}{\sin x}", MathOptions::default())?;
//! let paths = layout.to_paths(db);
//! ```
//!
//! The source is untrusted: its length ([`parser::MAX_SOURCE_CHARS`]), nesting
//! ([`parser::MAX_DEPTH`]) and glyph count ([`layout::MAX_GLYPHS`]) are bounded, and nothing in
//! the crate panics.
#![forbid(unsafe_code)]

pub mod layout;
pub mod mathfont;
pub mod output;
pub mod parser;
mod symbols;

#[cfg(test)]
mod tests_layout;
#[cfg(test)]
mod tests_parser;

pub use mathfont::find_math_face;
pub use output::{MathItem, MathLayout};
pub use parser::{Node, parse};
use vectorcraft_text::{FontDb, FontFace};

/// How to set a formula.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MathOptions {
    /// Font size of the formula's base style, in points.
    pub size_pt: f64,
    /// Display style (fractions and big operators set large, as on a line of their own) rather
    /// than text style (as inside a sentence).
    pub display: bool,
}

impl Default for MathOptions {
    fn default() -> Self {
        Self { size_pt: 12.0, display: false }
    }
}

/// Why a formula could not be set. The messages are in English.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum MathError {
    /// A command outside the supported LaTeX subset; the name is given without the backslash.
    #[error("unknown command \\{0}")]
    UnknownCommand(String),
    /// Malformed input; `position` counts characters from 0.
    #[error("{message} (at character {})", .position + 1)]
    Syntax { message: String, position: usize },
    #[error("the formula is longer than {max} characters")]
    TooLong { max: usize },
    #[error("the formula is nested deeper than {max} levels")]
    TooDeep { max: usize },
    #[error("the formula needs more than {max} glyphs")]
    TooLarge { max: usize },
    #[error("the formula is empty")]
    Empty,
    #[error("no math font is available (a font with an OpenType MATH table is needed)")]
    NoMathFont,
    #[error("the math font has no glyph for {0:?}")]
    MissingGlyph(char),
    #[error("the font size must be a positive number, not {0}")]
    BadSize(f64),
    #[error("the math font's metrics are unusable")]
    BadMetrics,
}

/// Typesets the LaTeX math `src` in `face`, a font with a `MATH` table (see [`find_math_face`]).
pub fn typeset(db: &FontDb, face: &FontFace, src: &str, opt: MathOptions) -> Result<MathLayout, MathError> {
    let nodes = parse(src)?;
    layout::layout(db, face, &nodes, opt)
}
