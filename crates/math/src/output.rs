//! What typesetting produces: glyph placements and rules in layout space, and the helper that
//! turns them into paths.

use std::sync::Arc;

use vectorcraft_geom::{Affine, BezPath, Rect, Shape};
use vectorcraft_text::{FontDb, FontFace};

/// One piece of a typeset formula.
#[derive(Clone, Debug, PartialEq)]
pub enum MathItem {
    /// A glyph of the math font. `xf` takes the glyph's outline in font units, y down (as
    /// [`FontDb::outline`] gives it, the same convention as `vectorcraft_text::PositionedGlyph`)
    /// to layout space.
    Glyph {
        /// [`FontFace::id`] of the face the glyph comes from.
        font_id: u32,
        /// The glyph's id in that face.
        gid: u32,
        xf: Affine,
    },
    /// A filled rectangle: a fraction bar, the bar of a radical, an overline.
    Rule { rect: Rect },
}

/// A typeset formula.
///
/// Layout space has y pointing down (as in the text crate) and the formula's top at `y = 0`: the
/// ink's highest point. `x = 0` is where the formula's advance starts, so the ink can begin a
/// little to the left or right of it. Translate by the insertion point to place it; the line the
/// formula sits on is `y = baseline`.
#[derive(Clone, Debug, PartialEq)]
pub struct MathLayout {
    /// Glyphs and rules, in drawing order.
    pub items: Vec<MathItem>,
    /// The ink's bounding box (its top is 0).
    pub bounds: Rect,
    /// The y of the baseline: the distance from the top of the ink down to it.
    pub baseline: f64,
}

impl MathLayout {
    /// The formula as filled outlines in layout space, one path per glyph or rule, in
    /// [`Self::items`] order. Outlines come from `db`; glyphs of a face `db` doesn't hold and
    /// glyphs without an outline are left out.
    pub fn to_paths(&self, db: &FontDb) -> Vec<BezPath> {
        // A formula uses one face; look each one up once.
        let mut faces: Vec<(u32, Option<Arc<FontFace>>)> = Vec::new();
        let mut paths = Vec::with_capacity(self.items.len());
        for item in &self.items {
            match item {
                MathItem::Glyph { font_id, gid, xf } => {
                    let face = match faces.iter().find(|(id, _)| id == font_id) {
                        Some((_, face)) => face.clone(),
                        None => {
                            let face = db.face_by_id(*font_id);
                            faces.push((*font_id, face.clone()));
                            face
                        }
                    };
                    let Some(face) = face else { continue };
                    let outline = db.outline(&face, *gid);
                    if outline.elements().is_empty() {
                        continue;
                    }
                    let mut path = (*outline).clone();
                    path.apply_affine(*xf);
                    paths.push(path);
                }
                MathItem::Rule { rect } => paths.push(rect.to_path(0.1)),
            }
        }
        paths
    }
}
