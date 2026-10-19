//! Cursor glyphs generated from original SVG drawings by VectorCraft contributors (each glyph names
//! its author; ASSETS.md). [`GENERATED`] says which cursors use them; the rest keep their
//! hand-drawn glyph in the parent module, or borrow another cursor's ([`Cursor::stand_in`]).
//! The `GENERATED` table and the glyph `const`s are regenerated from the drawings by cursor id
//! (the `Cursor` value in `crates/tools/src/cursors.toml`): change the drawing, not the numbers.

use super::{HALO, INK, Ink};
use egui::{Pos2, Shape, Stroke, vec2};
use vectorcraft_tools::Cursor;

/// The cursors drawn with a generated glyph.
pub(super) const GENERATED: &[(Cursor, &Glyph)] = &[
    (Cursor::Arrow, &ARROW),
    (Cursor::SelectionArrow, &SELECTION_ARROW),
    (Cursor::SelectionArrowHollow, &SELECTION_ARROW_HOLLOW),
    (Cursor::DirectAnchor, &DIRECT_ANCHOR),
    (Cursor::DirectAnchorSelected, &DIRECT_ANCHOR_SELECTED),
    (Cursor::ReshapeSegment, &RESHAPE_SEGMENT),
    (Cursor::GroupSelection, &GROUP_SELECTION),
    (Cursor::SelectionCopy, &SELECTION_COPY),
    (Cursor::Move, &MOVE),
    (Cursor::Crosshair, &CROSSHAIR),
    (Cursor::ResizeH, &RESIZE_H),
    (Cursor::ResizeV, &RESIZE_V),
    (Cursor::ResizeNwSe, &RESIZE_NW_SE),
    (Cursor::ResizeNeSw, &RESIZE_NE_SW),
    (Cursor::ShearH, &SHEAR_H),
    (Cursor::ShearV, &SHEAR_V),
    (Cursor::Rotate, &ROTATE),
    (Cursor::RotateToward(super::ROTATE_TOWARD_DRAWN), &ROTATE_TOWARD),
    (Cursor::CornerRadius, &CORNER_RADIUS),
    (Cursor::Pen, &PEN),
    (Cursor::PenDrawing, &PEN_DRAWING),
    (Cursor::PenAdd, &PEN_ADD),
    (Cursor::PenDelete, &PEN_DELETE),
    (Cursor::PenClose, &PEN_CLOSE),
    (Cursor::PenContinue, &PEN_CONTINUE),
    (Cursor::AnchorPoint, &ANCHOR_POINT),
    (Cursor::Curvature, &CURVATURE),
    (Cursor::PenJoin, &PEN_JOIN),
    (Cursor::PenConvert, &PEN_CONVERT),
    (Cursor::Text, &TEXT),
    (Cursor::Hand, &HAND),
    (Cursor::HandGrab, &HAND_GRAB),
    (Cursor::ZoomIn, &ZOOM_IN),
    (Cursor::ZoomOut, &ZOOM_OUT),
    (Cursor::Eyedropper, &EYEDROPPER),
    (Cursor::AddStop, &ADD_STOP),
    (Cursor::RemoveStop, &REMOVE_STOP),
    (Cursor::Slice, &SLICE),
    (Cursor::SliceSelect, &SLICE_SELECT),
    (Cursor::Width, &WIDTH),
    (Cursor::WidthAdd, &WIDTH_ADD),
    (Cursor::WidthPoint, &WIDTH_POINT),
    (Cursor::Blend, &BLEND),
    (Cursor::BlendObject, &BLEND_OBJECT),
    (Cursor::BlendAnchor, &BLEND_ANCHOR),
    (Cursor::PathBracket, &PATH_BRACKET),
    (Cursor::TypeWidget, &TYPE_WIDGET),
    (Cursor::ShapeBuilder, &SHAPE_BUILDER),
    (Cursor::ShapeBuilderErase, &SHAPE_BUILDER_ERASE),
    (Cursor::Lasso, &LASSO),
    (Cursor::MagicWand, &MAGIC_WAND),
    (Cursor::Pencil, &PENCIL),
    (Cursor::Paintbrush, &PAINTBRUSH),
    (Cursor::Smooth, &SMOOTH),
    (Cursor::Knife, &KNIFE),
    (Cursor::LivePaintBucket, &LIVE_PAINT_BUCKET),
    (Cursor::Measure, &MEASURE),
    (Cursor::SymbolSprayer, &SYMBOL_SPRAYER),
    (Cursor::SymbolShifter, &SYMBOL_SHIFTER),
    (Cursor::SymbolScruncher, &SYMBOL_SCRUNCHER),
    (Cursor::SymbolSizer, &SYMBOL_SIZER),
    (Cursor::SymbolSpinner, &SYMBOL_SPINNER),
    (Cursor::SymbolStainer, &SYMBOL_STAINER),
    (Cursor::SymbolScreener, &SYMBOL_SCREENER),
    (Cursor::SymbolStyler, &SYMBOL_STYLER),
    (Cursor::TypeVertical, &TYPE_VERTICAL),
    (Cursor::Scissors, &SCISSORS),
    (Cursor::ScissorsNotAllowed, &SCISSORS_NOT_ALLOWED),
    (Cursor::MeshAdd, &MESH_ADD),
];

/// The generated glyph for cursor `c`, if it has one.
pub(super) fn generated(c: Cursor) -> Option<&'static Glyph> {
    GENERATED.iter().find(|(k, _)| *k == c).map(|&(_, g)| g)
}

/// A cursor glyph generated from a drawing: its parts in drawing order, in points relative to the
/// hotspot. Drawn by [`draw_glyph`]. `id` names the cursor (its `Cursor` value).
pub(crate) struct Glyph {
    #[allow(dead_code, reason = "read by the tests")]
    pub id: &'static str,
    pub parts: &'static [GlyphPart],
}

pub(crate) enum GlyphPart {
    /// A filled shape: `tris` triangulates a concave shape, empty for a convex one.
    Fill { points: &'static [(f32, f32)], tris: &'static [[u16; 3]], white: bool },
    /// A stroked line, open or closed.
    Stroke { points: &'static [(f32, f32)], closed: bool, width: f32, white: bool },
}

/// The white halo under the ink, as the hand-drawn glyphs have it: a 3-point outline under a
/// fill (inked with a 1-point outline), and a stroke 1.8 points wider under a stroke.
const GLYPH_FILL_HALO: f32 = 3.0;
const GLYPH_STROKE_HALO: f32 = 1.8;

/// Draw a generated glyph with its hotspot at `o`, turned `turn` radians clockwise about it (0 as
/// drawn): every halo first, then every part in order, so no part's halo cuts through another's ink.
pub(super) fn draw_glyph(p: &mut Ink, o: Pos2, g: &Glyph, turn: f32) {
    let (s, c) = turn.sin_cos();
    let at = |pts: &[(f32, f32)]| pts.iter().map(|&(x, y)| o + vec2(x * c - y * s, x * s + y * c)).collect::<Vec<_>>();
    for part in g.parts {
        match part {
            GlyphPart::Fill { points, white: false, .. } => p.add(Shape::closed_line(at(points), Stroke::new(GLYPH_FILL_HALO, HALO))),
            GlyphPart::Stroke { points, closed, width, white: false } => {
                let stroke = Stroke::new(width + GLYPH_STROKE_HALO, HALO);
                p.add(if *closed { Shape::closed_line(at(points), stroke) } else { Shape::line(at(points), stroke) });
            }
            _ => {}
        }
    }
    for part in g.parts {
        match part {
            GlyphPart::Fill { points, tris, white } => {
                let colour = if *white { HALO } else { INK };
                let pts = at(points);
                if tris.is_empty() {
                    p.add(Shape::convex_polygon(pts.clone(), colour, Stroke::NONE));
                } else {
                    let mut mesh = egui::Mesh::default();
                    for &pos in &pts {
                        mesh.colored_vertex(pos, colour);
                    }
                    for t in *tris {
                        // Generated data: every index is below the point count (a test checks).
                        if t.iter().all(|&i| usize::from(i) < pts.len()) {
                            mesh.add_triangle(u32::from(t[0]), u32::from(t[1]), u32::from(t[2]));
                        }
                    }
                    p.add(Shape::mesh(mesh));
                }
                if !white {
                    p.add(Shape::closed_line(pts, Stroke::new(1.0, INK)));
                }
            }
            GlyphPart::Stroke { points, closed, width, white } => {
                let stroke = Stroke::new(*width, if *white { HALO } else { INK });
                p.add(if *closed { Shape::closed_line(at(points), stroke) } else { Shape::line(at(points), stroke) });
            }
        }
    }
}

// Generated glyphs, one per drawn cursor.

/// The `Arrow` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const ARROW: Glyph = Glyph {
    id: "Arrow",
    parts: &[
        GlyphPart::Fill { points: &[(0.34, 14.92), (0.35, 0.8), (10.65, 10.45), (6.01, 10.96), (8.73, 17.23), (6.61, 18.15), (3.89, 11.88)], tris: &[[6, 0, 1], [1, 2, 3], [6, 1, 3], [6, 3, 4], [4, 5, 6]], white: false },
        GlyphPart::Stroke { points: &[(0.34, 14.92), (0.35, 0.8), (10.65, 10.45), (6.01, 10.96), (8.73, 17.23), (6.61, 18.15), (3.89, 11.88)], closed: true, width: 1.0, white: false },
    ],
};

/// The `SelectionArrow` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SELECTION_ARROW: Glyph = Glyph {
    id: "SelectionArrow",
    parts: &[
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
    ],
};

/// The `SelectionArrowHollow` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SELECTION_ARROW_HOLLOW: Glyph = Glyph {
    id: "SelectionArrowHollow",
    parts: &[
        GlyphPart::Fill { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], tris: &[[0, 3, 2], [2, 1, 0]], white: true },
        GlyphPart::Stroke { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], closed: true, width: 1.0, white: false },
    ],
};

/// The `DirectAnchor` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const DIRECT_ANCHOR: Glyph = Glyph {
    id: "DirectAnchor",
    parts: &[
        GlyphPart::Stroke { points: &[(15.44, 16.49), (19.59, 16.49), (19.59, 20.63), (15.44, 20.63)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], tris: &[[0, 3, 2], [2, 1, 0]], white: true },
        GlyphPart::Stroke { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], closed: true, width: 1.0, white: false },
    ],
};

/// The `DirectAnchorSelected` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const DIRECT_ANCHOR_SELECTED: Glyph = Glyph {
    id: "DirectAnchorSelected",
    parts: &[
        GlyphPart::Fill { points: &[(15.61, 16.77), (19.42, 16.77), (19.42, 20.59), (15.61, 20.59)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(15.61, 16.77), (19.42, 16.77), (19.42, 20.59), (15.61, 20.59)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], tris: &[[0, 3, 2], [2, 1, 0]], white: true },
        GlyphPart::Stroke { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], closed: true, width: 1.0, white: false },
    ],
};

/// The `ReshapeSegment` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const RESHAPE_SEGMENT: Glyph = Glyph {
    id: "ReshapeSegment",
    parts: &[
        GlyphPart::Fill { points: &[(15.38, 21.31), (15.56, 19.92), (16.09, 18.67), (16.91, 17.61), (17.97, 16.8), (19.22, 16.27), (20.61, 16.08), (20.61, 16.25), (19.01, 16.51), (17.62, 17.23), (16.52, 18.32), (15.81, 19.71), (15.55, 21.31)], tris: &[[12, 0, 1], [1, 2, 3], [3, 4, 5], [5, 6, 7], [5, 7, 8], [5, 8, 9], [3, 5, 9], [3, 9, 10], [1, 3, 10], [1, 10, 11], [1, 11, 12]], white: false },
        GlyphPart::Stroke { points: &[(15.38, 21.31), (15.56, 19.92), (16.09, 18.67), (16.91, 17.61), (17.97, 16.8), (19.22, 16.27), (20.61, 16.08), (20.61, 16.25), (19.01, 16.51), (17.62, 17.23), (16.52, 18.32), (15.81, 19.71), (15.55, 21.31)], closed: true, width: 1.2, white: false },
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(23.0, 15.33), (24.51, 15.33), (24.51, 16.84), (23.0, 16.84)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(23.0, 15.33), (24.51, 15.33), (24.51, 16.84), (23.0, 16.84)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(14.62, 23.37), (16.13, 23.37), (16.13, 24.88), (14.62, 24.88)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(14.62, 23.37), (16.13, 23.37), (16.13, 24.88), (14.62, 24.88)], closed: true, width: 1.0, white: false },
    ],
};

/// The `GroupSelection` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const GROUP_SELECTION: Glyph = Glyph {
    id: "GroupSelection",
    parts: &[
        GlyphPart::Fill { points: &[(12.71, 7.65), (12.71, 0.5), (12.95, 0.5), (12.95, 7.65)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(12.71, 7.65), (12.71, 0.5), (12.95, 0.5), (12.95, 7.65)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(9.26, 4.19), (9.26, 3.95), (16.4, 3.95), (16.4, 4.19)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(9.26, 4.19), (9.26, 3.95), (16.4, 3.95), (16.4, 4.19)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], tris: &[[0, 3, 2], [2, 1, 0]], white: true },
        GlyphPart::Stroke { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], closed: true, width: 1.0, white: false },
    ],
};

/// The `SelectionCopy` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SELECTION_COPY: Glyph = Glyph {
    id: "SelectionCopy",
    parts: &[
        GlyphPart::Fill { points: &[(11.61, 19.74), (18.13, 19.74), (6.8, 7.81), (6.93, 24.26)], tris: &[[0, 3, 2], [2, 1, 0]], white: true },
        GlyphPart::Stroke { points: &[(11.61, 19.74), (18.13, 19.74), (6.8, 7.81), (6.93, 24.26)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
    ],
};

/// The `Move` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const MOVE: Glyph = Glyph {
    id: "Move",
    parts: &[
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(17.73, 22.11), (17.73, 15.62)], closed: false, width: 0.7, white: false },
        GlyphPart::Fill { points: &[(17.39, 15.54), (18.07, 15.54), (17.73, 15.12)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(17.39, 15.54), (18.07, 15.54), (17.73, 15.12)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(18.07, 22.29), (17.39, 22.29), (17.73, 22.72)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(18.07, 22.29), (17.39, 22.29), (17.73, 22.72)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(14.55, 18.92), (21.03, 18.92)], closed: false, width: 0.7, white: false },
        GlyphPart::Fill { points: &[(21.11, 18.58), (21.11, 19.26), (21.53, 18.92)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(21.11, 18.58), (21.11, 19.26), (21.53, 18.92)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(14.36, 19.26), (14.36, 18.58), (13.93, 18.92)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(14.36, 19.26), (14.36, 18.58), (13.93, 18.92)], closed: true, width: 1.0, white: false },
    ],
};

/// The `Crosshair` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (12.0, 12.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const CROSSHAIR: Glyph = Glyph {
    id: "Crosshair",
    parts: &[
        GlyphPart::Stroke { points: &[(-9.0, 0.0), (-2.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(2.0, 0.0), (9.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, -9.0), (0.0, -2.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, 2.0), (0.0, 9.0)], closed: false, width: 1.2, white: false },
    ],
};

/// The `ResizeH` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const RESIZE_H: Glyph = Glyph {
    id: "ResizeH",
    parts: &[
        GlyphPart::Stroke { points: &[(-5.76, 0.0), (5.76, 0.0)], closed: false, width: 1.3, white: false },
        GlyphPart::Fill { points: &[(-5.83, 1.18), (-5.83, -1.19), (-7.31, 0.0)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-5.83, 1.18), (-5.83, -1.19), (-7.31, 0.0)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(5.83, 1.18), (5.83, -1.19), (7.31, 0.0)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(5.83, 1.18), (5.83, -1.19), (7.31, 0.0)], closed: true, width: 1.0, white: false },
    ],
};

/// The `ResizeV` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const RESIZE_V: Glyph = Glyph {
    id: "ResizeV",
    parts: &[
        GlyphPart::Stroke { points: &[(0.0, -5.76), (0.0, 5.76)], closed: false, width: 1.3, white: false },
        GlyphPart::Fill { points: &[(-1.19, -5.83), (1.18, -5.83), (0.0, -7.31)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-1.19, -5.83), (1.18, -5.83), (0.0, -7.31)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-1.19, 5.83), (1.18, 5.83), (0.0, 7.31)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-1.19, 5.83), (1.18, 5.83), (0.0, 7.31)], closed: true, width: 1.0, white: false },
    ],
};

/// The `ResizeNwSe` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const RESIZE_NW_SE: Glyph = Glyph {
    id: "ResizeNwSe",
    parts: &[
        GlyphPart::Stroke { points: &[(-4.07, -4.07), (4.07, 4.07)], closed: false, width: 1.3, white: false },
        GlyphPart::Fill { points: &[(-4.96, -3.29), (-3.29, -4.96), (-5.17, -5.17)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-4.96, -3.29), (-3.29, -4.96), (-5.17, -5.17)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(3.29, 4.96), (4.96, 3.29), (5.17, 5.17)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(3.29, 4.96), (4.96, 3.29), (5.17, 5.17)], closed: true, width: 1.0, white: false },
    ],
};

/// The `ResizeNeSw` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const RESIZE_NE_SW: Glyph = Glyph {
    id: "ResizeNeSw",
    parts: &[
        GlyphPart::Stroke { points: &[(-4.07, 4.07), (4.07, -4.07)], closed: false, width: 1.3, white: false },
        GlyphPart::Fill { points: &[(-3.29, 4.96), (-4.96, 3.29), (-5.17, 5.17)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-3.29, 4.96), (-4.96, 3.29), (-5.17, 5.17)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(4.96, -3.29), (3.29, -4.96), (5.17, -5.17)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(4.96, -3.29), (3.29, -4.96), (5.17, -5.17)], closed: true, width: 1.0, white: false },
    ],
};

/// The `ShearH` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SHEAR_H: Glyph = Glyph {
    id: "ShearH",
    parts: &[
        GlyphPart::Fill { points: &[(7.0, -3.5), (2.8, -0.9), (2.8, -6.1)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(7.0, -3.5), (2.8, -0.9), (2.8, -6.1)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-7.0, 3.5), (-2.8, 0.9), (-2.8, 6.1)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-7.0, 3.5), (-2.8, 0.9), (-2.8, 6.1)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-7.4, -3.85), (2.8, -3.85), (2.8, -3.15), (-7.4, -3.15)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-7.4, -3.85), (2.8, -3.85), (2.8, -3.15), (-7.4, -3.15)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(7.4, 3.85), (-2.8, 3.85), (-2.8, 3.15), (7.4, 3.15)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(7.4, 3.85), (-2.8, 3.85), (-2.8, 3.15), (7.4, 3.15)], closed: true, width: 1.0, white: false },
    ],
};

/// The `ShearV` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SHEAR_V: Glyph = Glyph {
    id: "ShearV",
    parts: &[
        GlyphPart::Fill { points: &[(3.5, 7.0), (0.9, 2.8), (6.1, 2.8)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(3.5, 7.0), (0.9, 2.8), (6.1, 2.8)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-3.5, -7.0), (-0.9, -2.8), (-6.1, -2.8)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-3.5, -7.0), (-0.9, -2.8), (-6.1, -2.8)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(3.85, -7.4), (3.85, 2.8), (3.15, 2.8), (3.15, -7.4)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(3.85, -7.4), (3.85, 2.8), (3.15, 2.8), (3.15, -7.4)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-3.85, 7.4), (-3.85, -2.8), (-3.15, -2.8), (-3.15, 7.4)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-3.85, 7.4), (-3.85, -2.8), (-3.15, -2.8), (-3.15, 7.4)], closed: true, width: 1.0, white: false },
    ],
};

/// The `Rotate` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const ROTATE: Glyph = Glyph {
    id: "Rotate",
    parts: &[
        GlyphPart::Stroke { points: &[(-7.34, -3.19), (-6.59, -4.53), (-5.62, -5.7), (-4.44, -6.66), (-3.09, -7.38), (-1.6, -7.84), (0.0, -8.0), (1.6, -7.84), (3.1, -7.38), (4.45, -6.65), (5.63, -5.68), (6.61, -4.51), (7.35, -3.17)], closed: false, width: 1.3, white: false },
        GlyphPart::Fill { points: &[(8.05, -3.59), (6.36, -2.88), (7.65, -2.18)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(8.05, -3.59), (6.36, -2.88), (7.65, -2.18)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-8.05, -3.59), (-6.36, -2.88), (-7.65, -2.18)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-8.05, -3.59), (-6.36, -2.88), (-7.65, -2.18)], closed: true, width: 1.0, white: false },
    ],
};

/// The `RotateToward` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const ROTATE_TOWARD: Glyph = Glyph {
    id: "RotateToward",
    parts: &[
        GlyphPart::Stroke { points: &[(-2.93, -7.45), (-1.46, -7.87), (0.06, -8.0), (1.57, -7.85), (3.04, -7.4), (4.41, -6.67), (5.66, -5.66), (6.68, -4.41), (7.41, -3.03), (7.85, -1.56), (8.0, -0.04), (7.86, 1.48), (7.44, 2.95)], closed: false, width: 1.3, white: false },
        GlyphPart::Fill { points: &[(8.23, 3.16), (6.53, 2.46), (6.95, 3.86)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(8.23, 3.16), (6.53, 2.46), (6.95, 3.86)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-3.15, -8.23), (-2.46, -6.53), (-3.86, -6.95)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-3.15, -8.23), (-2.46, -6.53), (-3.86, -6.95)], closed: true, width: 1.0, white: false },
    ],
};

/// The `CornerRadius` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const CORNER_RADIUS: Glyph = Glyph {
    id: "CornerRadius",
    parts: &[
        GlyphPart::Fill { points: &[(14.22, 19.79), (14.45, 18.38), (15.08, 17.15), (16.04, 16.19), (17.27, 15.55), (18.68, 15.33), (21.98, 15.33), (21.98, 15.56), (18.68, 15.56), (17.34, 15.77), (16.18, 16.37), (15.26, 17.29), (14.66, 18.45), (14.45, 19.79), (14.45, 23.07), (14.22, 23.07)], tris: &[[15, 0, 1], [1, 2, 3], [3, 4, 5], [5, 6, 7], [5, 7, 8], [5, 8, 9], [3, 5, 9], [3, 9, 10], [3, 10, 11], [1, 3, 11], [1, 11, 12], [1, 12, 13], [15, 1, 13], [13, 14, 15]], white: false },
        GlyphPart::Stroke { points: &[(14.22, 19.79), (14.45, 18.38), (15.08, 17.15), (16.04, 16.19), (17.27, 15.55), (18.68, 15.33), (21.98, 15.33), (21.98, 15.56), (18.68, 15.56), (17.34, 15.77), (16.18, 16.37), (15.26, 17.29), (14.66, 18.45), (14.45, 19.79), (14.45, 23.07), (14.22, 23.07)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], tris: &[[0, 3, 2], [2, 1, 0]], white: true },
        GlyphPart::Stroke { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], closed: true, width: 1.0, white: false },
    ],
};

/// The `Pen` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PEN: Glyph = Glyph {
    id: "Pen",
    parts: &[
        GlyphPart::Fill { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(0.0, 0.0), (5.06, 12.03)], closed: false, width: 1.3, white: true },
        GlyphPart::Fill { points: &[(2.93, 12.13), (2.93, 10.61), (4.0, 9.52), (5.53, 9.51), (6.62, 10.59), (6.62, 12.11), (5.55, 13.2), (4.02, 13.21)], tris: &[], white: true },
        GlyphPart::Fill { points: &[(21.85, 18.33), (21.65, 18.45), (20.67, 16.75), (20.87, 16.63)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(21.85, 18.33), (21.65, 18.45), (20.67, 16.75), (20.87, 16.63)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(23.66, 14.87), (23.66, 15.1), (21.7, 15.1), (21.7, 14.87)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(23.66, 14.87), (23.66, 15.1), (21.7, 15.1), (21.7, 14.87)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(21.57, 11.57), (21.77, 11.68), (20.79, 13.38), (20.59, 13.27)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(21.57, 11.57), (21.77, 11.68), (20.79, 13.38), (20.59, 13.27)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(17.66, 11.73), (17.87, 11.61), (18.85, 13.31), (18.64, 13.43)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(17.66, 11.73), (17.87, 11.61), (18.85, 13.31), (18.64, 13.43)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(15.85, 15.2), (15.85, 14.96), (17.81, 14.96), (17.81, 15.2)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(15.85, 15.2), (15.85, 14.96), (17.81, 14.96), (17.81, 15.2)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(17.95, 18.5), (17.74, 18.38), (18.72, 16.68), (18.93, 16.8)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(17.95, 18.5), (17.74, 18.38), (18.72, 16.68), (18.93, 16.8)], closed: true, width: 1.0, white: false },
    ],
};

/// The `PenDrawing` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PEN_DRAWING: Glyph = Glyph {
    id: "PenDrawing",
    parts: &[
        GlyphPart::Fill { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(0.0, 0.0), (5.06, 12.03)], closed: false, width: 1.3, white: true },
        GlyphPart::Fill { points: &[(2.93, 12.13), (2.93, 10.61), (4.0, 9.52), (5.53, 9.51), (6.62, 10.59), (6.62, 12.11), (5.55, 13.2), (4.02, 13.21)], tris: &[], white: true },
    ],
};

/// The `PenAdd` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PEN_ADD: Glyph = Glyph {
    id: "PenAdd",
    parts: &[
        GlyphPart::Fill { points: &[(19.62, 18.67), (19.62, 11.53), (19.86, 11.53), (19.86, 18.67)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(19.62, 18.67), (19.62, 11.53), (19.86, 11.53), (19.86, 18.67)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(16.16, 15.22), (16.16, 14.98), (23.31, 14.98), (23.31, 15.22)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(16.16, 15.22), (16.16, 14.98), (23.31, 14.98), (23.31, 15.22)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(0.0, 0.0), (5.06, 12.03)], closed: false, width: 1.3, white: true },
        GlyphPart::Fill { points: &[(2.93, 12.13), (2.93, 10.61), (4.0, 9.52), (5.53, 9.51), (6.62, 10.59), (6.62, 12.11), (5.55, 13.2), (4.02, 13.21)], tris: &[], white: true },
    ],
};

/// The `PenDelete` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PEN_DELETE: Glyph = Glyph {
    id: "PenDelete",
    parts: &[
        GlyphPart::Fill { points: &[(16.16, 15.22), (16.16, 14.98), (23.31, 14.98), (23.31, 15.22)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(16.16, 15.22), (16.16, 14.98), (23.31, 14.98), (23.31, 15.22)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(0.0, 0.0), (5.06, 12.03)], closed: false, width: 1.3, white: true },
        GlyphPart::Fill { points: &[(2.93, 12.13), (2.93, 10.61), (4.0, 9.52), (5.53, 9.51), (6.62, 10.59), (6.62, 12.11), (5.55, 13.2), (4.02, 13.21)], tris: &[], white: true },
    ],
};

/// The `PenClose` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PEN_CLOSE: Glyph = Glyph {
    id: "PenClose",
    parts: &[
        GlyphPart::Stroke { points: &[(22.86, 15.1), (22.61, 16.31), (21.94, 17.3), (20.95, 17.97), (19.74, 18.22), (18.52, 17.97), (17.53, 17.3), (16.86, 16.31), (16.62, 15.1), (16.86, 13.88), (17.53, 12.89), (18.52, 12.23), (19.74, 11.98), (20.95, 12.23), (21.94, 12.89), (22.61, 13.88)], closed: true, width: 1.2, white: false },
        GlyphPart::Fill { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(0.0, 0.0), (5.06, 12.03)], closed: false, width: 1.3, white: true },
        GlyphPart::Fill { points: &[(2.93, 12.13), (2.93, 10.61), (4.0, 9.52), (5.53, 9.51), (6.62, 10.59), (6.62, 12.11), (5.55, 13.2), (4.02, 13.21)], tris: &[], white: true },
    ],
};

/// The `PenContinue` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PEN_CONTINUE: Glyph = Glyph {
    id: "PenContinue",
    parts: &[
        GlyphPart::Fill { points: &[(16.79, 18.59), (22.74, 11.45), (22.93, 11.6), (16.97, 18.75)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(16.79, 18.59), (22.74, 11.45), (22.93, 11.6), (16.97, 18.75)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(0.0, 0.0), (5.06, 12.03)], closed: false, width: 1.3, white: true },
        GlyphPart::Fill { points: &[(2.93, 12.13), (2.93, 10.61), (4.0, 9.52), (5.53, 9.51), (6.62, 10.59), (6.62, 12.11), (5.55, 13.2), (4.02, 13.21)], tris: &[], white: true },
    ],
};

/// The `AnchorPoint` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 11.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const ANCHOR_POINT: Glyph = Glyph {
    id: "AnchorPoint",
    parts: &[
        GlyphPart::Fill { points: &[(-2.03, 1.43), (-2.03, -2.03), (2.03, -2.03), (2.03, 1.43), (7.11, 12.75), (6.01, 13.25), (0.98, 2.03), (-0.98, 2.03), (-6.01, 13.25), (-7.11, 12.75)], tris: &[[0, 1, 2], [0, 2, 3], [3, 4, 5], [3, 5, 6], [0, 3, 6], [0, 6, 7], [9, 0, 7], [7, 8, 9]], white: false },
        GlyphPart::Fill { points: &[(-1.03, -1.03), (1.03, -1.03), (1.03, 1.03), (-1.03, 1.03)], tris: &[], white: true },
    ],
};

/// The `Curvature` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const CURVATURE: Glyph = Glyph {
    id: "Curvature",
    parts: &[
        GlyphPart::Fill { points: &[(9.64, 16.63), (9.51, 10.47), (0.83, 0.97), (0.11, 1.27), (0.83, 14.12), (5.14, 18.53)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(9.64, 16.63), (9.51, 10.47), (0.83, 0.97), (0.11, 1.27), (0.83, 14.12), (5.14, 18.53)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(0.0, 0.0), (4.66, 11.1)], closed: false, width: 1.3, white: true },
        GlyphPart::Fill { points: &[(2.71, 11.19), (2.7, 9.78), (3.69, 8.78), (5.1, 8.77), (6.1, 9.77), (6.11, 11.18), (5.12, 12.18), (3.71, 12.19)], tris: &[], white: true },
        GlyphPart::Fill { points: &[(17.04, 19.29), (17.22, 17.69), (17.75, 16.23), (18.57, 14.93), (19.65, 13.85), (20.94, 13.03), (22.41, 12.51), (24.0, 12.32), (24.0, 12.55), (22.46, 12.73), (21.04, 13.24), (19.79, 14.03), (18.75, 15.08), (17.95, 16.33), (17.44, 17.75), (17.27, 19.29)], tris: &[[15, 0, 1], [1, 2, 3], [3, 4, 5], [5, 6, 7], [7, 8, 9], [5, 7, 9], [5, 9, 10], [5, 10, 11], [3, 5, 11], [3, 11, 12], [3, 12, 13], [1, 3, 13], [1, 13, 14], [1, 14, 15]], white: false },
        GlyphPart::Stroke { points: &[(17.04, 19.29), (17.22, 17.69), (17.75, 16.23), (18.57, 14.93), (19.65, 13.85), (20.94, 13.03), (22.41, 12.51), (24.0, 12.32), (24.0, 12.55), (22.46, 12.73), (21.04, 13.24), (19.79, 14.03), (18.75, 15.08), (17.95, 16.33), (17.44, 17.75), (17.27, 19.29)], closed: true, width: 1.2, white: false },
    ],
};

/// The `PenJoin` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PEN_JOIN: Glyph = Glyph {
    id: "PenJoin",
    parts: &[
        GlyphPart::Fill { points: &[(17.83, 13.19), (21.65, 13.19), (21.65, 17.01), (17.83, 17.01)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(17.83, 13.19), (21.65, 13.19), (21.65, 17.01), (17.83, 17.01)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(0.0, 0.0), (5.06, 12.03)], closed: false, width: 1.3, white: true },
        GlyphPart::Fill { points: &[(2.93, 12.13), (2.93, 10.61), (4.0, 9.52), (5.53, 9.51), (6.62, 10.59), (6.62, 12.11), (5.55, 13.2), (4.02, 13.21)], tris: &[], white: true },
    ],
};

/// The `PenConvert` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PEN_CONVERT: Glyph = Glyph {
    id: "PenConvert",
    parts: &[
        GlyphPart::Fill { points: &[(19.74, 12.35), (22.78, 18.43), (22.58, 18.53), (19.74, 12.85), (16.9, 18.53), (16.7, 18.43)], tris: &[[0, 1, 2], [0, 2, 3], [5, 0, 3], [3, 4, 5]], white: false },
        GlyphPart::Stroke { points: &[(19.74, 12.35), (22.78, 18.43), (22.58, 18.53), (19.74, 12.85), (16.9, 18.53), (16.7, 18.43)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(10.45, 18.03), (10.31, 11.35), (0.9, 1.05), (0.12, 1.38), (0.9, 15.31), (5.57, 20.08)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(0.0, 0.0), (5.06, 12.03)], closed: false, width: 1.3, white: true },
        GlyphPart::Fill { points: &[(2.93, 12.13), (2.93, 10.61), (4.0, 9.52), (5.53, 9.51), (6.62, 10.59), (6.62, 12.11), (5.55, 13.2), (4.02, 13.21)], tris: &[], white: true },
    ],
};

/// The `Text` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 19.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const TEXT: Glyph = Glyph {
    id: "Text",
    parts: &[
        GlyphPart::Fill { points: &[(-1.44, -0.05), (1.44, -0.05), (1.44, 0.05), (-1.44, 0.05)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-1.44, -0.05), (1.44, -0.05), (1.44, 0.05), (-1.44, 0.05)], closed: true, width: 1.1, white: false },
        GlyphPart::Fill { points: &[(0.05, -10.42), (0.05, 4.92), (-0.05, 4.92), (-0.05, -10.42)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(0.05, -10.42), (0.05, 4.92), (-0.05, 4.92), (-0.05, -10.42)], closed: true, width: 1.1, white: false },
        GlyphPart::Fill { points: &[(-2.46, -11.05), (2.46, -11.05), (2.46, -10.95), (-2.46, -10.95)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-2.46, -11.05), (2.46, -11.05), (2.46, -10.95), (-2.46, -10.95)], closed: true, width: 1.1, white: false },
        GlyphPart::Fill { points: &[(-2.46, 5.0), (2.46, 5.0), (2.46, 5.1), (-2.46, 5.1)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-2.46, 5.0), (2.46, 5.0), (2.46, 5.1), (-2.46, 5.1)], closed: true, width: 1.1, white: false },
    ],
};

/// The `Hand` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.5, 18.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const HAND: Glyph = Glyph {
    id: "Hand",
    parts: &[
        GlyphPart::Fill { points: &[(-6.71, -2.28), (-8.03, -7.71), (-7.82, -8.91), (-6.82, -9.62), (-6.77, -9.63), (-5.56, -9.42), (-4.86, -8.42), (-3.25, -1.38)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-6.71, -2.28), (-8.03, -7.71), (-7.82, -8.91), (-6.82, -9.62), (-6.77, -9.63), (-5.56, -9.42), (-4.86, -8.42), (-3.25, -1.38)], closed: false, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-2.73, -1.26), (-3.66, -10.48), (-3.31, -11.65), (-2.23, -12.24), (-2.18, -12.24), (-1.01, -11.89), (-0.42, -10.82), (0.5, -1.6)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-2.73, -1.26), (-3.66, -10.48), (-3.31, -11.65), (-2.23, -12.24), (-2.18, -12.24), (-1.01, -11.89), (-0.42, -10.82), (0.5, -1.6)], closed: false, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(0.5, -1.6), (1.05, -9.88), (1.59, -10.98), (2.75, -11.37), (2.8, -11.37), (3.9, -10.83), (4.29, -9.67), (3.75, -1.38)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(0.5, -1.6), (1.05, -9.88), (1.59, -10.98), (2.75, -11.37), (2.8, -11.37), (3.9, -10.83), (4.29, -9.67), (3.75, -1.38)], closed: false, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(4.0, -2.1), (5.14, -6.23), (5.82, -7.16), (6.95, -7.33), (7.88, -6.65), (8.05, -5.51), (6.91, -1.38)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(4.0, -2.1), (5.14, -6.23), (5.82, -7.16), (6.95, -7.33), (6.95, -7.33), (7.88, -6.65), (8.05, -5.51), (6.91, -1.38)], closed: false, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-10.3, -0.57), (-10.27, -0.6), (-9.1, -0.97), (-8.01, -0.41), (-3.0, 5.57), (-2.63, 6.74), (-3.2, 7.82), (-3.23, 7.85), (-4.4, 8.22), (-5.49, 7.66), (-10.5, 1.68), (-10.87, 0.51)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-10.3, -0.57), (-10.27, -0.6), (-9.1, -0.97), (-8.01, -0.41), (-3.0, 5.57), (-2.63, 6.74), (-3.2, 7.82), (-3.23, 7.85), (-4.4, 8.22), (-5.49, 7.66), (-10.5, 1.68), (-10.87, 0.51)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-6.21, -2.29), (7.42, -2.29), (6.88, 4.38), (6.71, 5.9), (6.2, 7.18), (5.4, 8.23), (4.35, 9.05), (3.09, 9.64), (1.65, 9.99), (0.07, 10.1), (-1.76, 9.95), (-3.4, 9.51), (-4.77, 8.81), (-5.83, 7.88), (-6.51, 6.76), (-6.75, 5.47), (-7.09, 2.13), (-5.8, 1.15)], tris: &[[17, 0, 1], [17, 1, 2], [17, 2, 3], [17, 3, 4], [17, 4, 5], [17, 5, 6], [17, 6, 7], [17, 7, 8], [17, 8, 9], [17, 9, 10], [17, 10, 11], [17, 11, 12], [17, 12, 13], [17, 13, 14], [17, 14, 15], [15, 16, 17]], white: true },
        GlyphPart::Stroke { points: &[(7.48, -3.45), (7.06, -1.35), (7.0, -0.33), (7.0, 4.25), (6.82, 5.75), (6.32, 7.03), (5.52, 8.09), (4.48, 8.92), (3.23, 9.52), (1.8, 9.88), (0.25, 10.0), (-1.61, 9.88), (-3.04, 9.54), (-4.15, 8.97), (-5.05, 8.2), (-5.84, 7.24), (-6.56, 6.35), (-7.17, 5.7), (-8.11, 4.53)], closed: false, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-7.09, -3.98), (-6.92, -3.21), (-6.55, -1.42), (-6.25, 0.58), (-6.25, 2.0)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-7.09, -3.98), (-6.92, -3.21), (-6.55, -1.42), (-6.25, 0.58), (-6.25, 2.0)], closed: false, width: 1.0, white: false },
    ],
};

/// The `HandGrab` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.5, 18.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const HAND_GRAB: Glyph = Glyph {
    id: "HandGrab",
    parts: &[
        GlyphPart::Fill { points: &[(-6.52, -0.2), (-6.71, -6.23), (-6.27, -7.38), (-5.16, -7.88), (-5.11, -7.88), (-3.96, -7.45), (-3.46, -6.33), (-3.3, 1.33)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-6.52, -0.2), (-6.71, -6.23), (-6.27, -7.38), (-5.16, -7.88), (-5.11, -7.88), (-3.96, -7.45), (-3.46, -6.33), (-3.3, 1.33)], closed: false, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-2.96, 2.43), (-3.49, -7.27), (-3.09, -8.43), (-1.99, -8.97), (-1.94, -8.98), (-0.78, -8.58), (-0.24, -7.48), (0.29, 2.23)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-2.96, 2.43), (-3.49, -7.27), (-3.09, -8.43), (-1.99, -8.97), (-1.94, -8.98), (-0.78, -8.58), (-0.24, -7.48), (0.29, 2.23)], closed: false, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-0.22, 1.56), (-0.11, -7.2), (0.38, -8.33), (1.52, -8.78), (1.57, -8.78), (2.69, -8.29), (3.14, -7.15), (3.03, 1.61)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-0.22, 1.56), (-0.11, -7.2), (0.38, -8.33), (1.52, -8.78), (1.57, -8.78), (2.69, -8.29), (3.14, -7.15), (3.03, 1.61)], closed: false, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(3.14, -0.79), (3.22, -5.52), (3.68, -6.57), (4.75, -6.99), (5.8, -6.54), (6.22, -5.47), (6.61, -1.86)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(3.14, -0.79), (3.22, -5.52), (3.68, -6.57), (4.75, -6.99), (4.75, -6.99), (5.8, -6.54), (6.22, -5.47), (6.61, -1.86)], closed: false, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-7.43, 5.22), (-8.04, 4.44), (-9.08, 2.9), (-9.32, 1.28), (-8.64, 0.0), (-7.33, -0.39), (-5.87, 0.0), (-4.77, 1.03)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-7.43, 5.22), (-8.04, 4.44), (-9.08, 2.9), (-9.32, 1.28), (-8.64, 0.0), (-7.33, -0.39), (-5.87, 0.0), (-4.77, 1.03)], closed: false, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-6.42, -3.29), (-4.88, -3.29), (-4.3, -4.02), (4.08, -4.02), (5.02, -3.29), (6.67, -3.29), (6.67, 3.25), (6.49, 4.77), (5.98, 6.06), (5.18, 7.11), (4.13, 7.93), (2.87, 8.51), (1.43, 8.86), (-0.15, 8.98), (-1.98, 8.82), (-3.62, 8.38), (-4.99, 7.68), (-6.05, 6.76), (-6.73, 5.63), (-6.97, 4.34), (-7.31, 1.01), (-5.87, 0.0)], tris: &[[21, 0, 1], [21, 1, 2], [21, 2, 3], [21, 3, 4], [21, 4, 5], [21, 5, 6], [21, 6, 7], [21, 7, 8], [21, 8, 9], [21, 9, 10], [21, 10, 11], [21, 11, 12], [21, 12, 13], [21, 13, 14], [21, 14, 15], [21, 15, 16], [21, 16, 17], [21, 17, 18], [21, 18, 19], [19, 20, 21]], white: true },
        GlyphPart::Stroke { points: &[(6.43, -3.53), (6.64, -1.9), (6.81, -0.44), (6.88, 1.14), (6.78, 3.13), (6.52, 4.7), (5.98, 6.01), (5.19, 7.06), (4.18, 7.87), (2.96, 8.43), (1.57, 8.77), (0.03, 8.88), (-1.79, 8.8), (-3.16, 8.55), (-4.26, 8.09), (-5.26, 7.39), (-6.32, 6.4), (-6.99, 5.74), (-7.43, 5.21), (-8.28, 4.07)], closed: false, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(-6.64, -3.89), (-6.6, -3.3), (-6.51, -1.84), (-6.36, 0.0)], closed: false, width: 1.0, white: false },
    ],
};

/// The `ZoomIn` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (13.0, 13.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const ZOOM_IN: Glyph = Glyph {
    id: "ZoomIn",
    parts: &[
        GlyphPart::Fill { points: &[(5.17, 0.04), (4.99, 1.41), (4.47, 2.64), (3.66, 3.68), (2.62, 4.49), (1.39, 5.01), (0.02, 5.19), (-1.35, 5.01), (-2.59, 4.49), (-3.63, 3.68), (-4.44, 2.64), (-4.95, 1.41), (-5.14, 0.04), (-4.95, -1.33), (-4.44, -2.57), (-3.63, -3.61), (-2.59, -4.42), (-1.35, -4.94), (0.02, -5.12), (1.39, -4.94), (2.62, -4.42), (3.66, -3.61), (4.47, -2.57), (4.99, -1.33)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(5.17, 0.04), (4.99, 1.41), (4.47, 2.64), (3.66, 3.68), (2.62, 4.49), (1.39, 5.01), (0.02, 5.19), (-1.35, 5.01), (-2.59, 4.49), (-3.63, 3.68), (-4.44, 2.64), (-4.95, 1.41), (-5.14, 0.04), (-4.95, -1.33), (-4.44, -2.57), (-3.63, -3.61), (-2.59, -4.42), (-1.35, -4.94), (0.02, -5.12), (1.39, -4.94), (2.62, -4.42), (3.66, -3.61), (4.47, -2.57), (4.99, -1.33)], closed: true, width: 1.3, white: false },
        GlyphPart::Fill { points: &[(-0.08, 2.33), (-0.08, -2.26), (0.08, -2.26), (0.08, 2.33)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-0.08, 2.33), (-0.08, -2.26), (0.08, -2.26), (0.08, 2.33)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-2.3, 0.11), (-2.3, -0.04), (2.3, -0.04), (2.3, 0.11)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-2.3, 0.11), (-2.3, -0.04), (2.3, -0.04), (2.3, 0.11)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(3.0, 4.92), (4.92, 3.0), (10.21, 8.28), (8.28, 10.21)], tris: &[], white: false },
    ],
};

/// The `ZoomOut` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (13.0, 13.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const ZOOM_OUT: Glyph = Glyph {
    id: "ZoomOut",
    parts: &[
        GlyphPart::Fill { points: &[(5.17, 0.04), (4.99, 1.41), (4.47, 2.64), (3.66, 3.68), (2.62, 4.49), (1.39, 5.01), (0.02, 5.19), (-1.35, 5.01), (-2.59, 4.49), (-3.63, 3.68), (-4.44, 2.64), (-4.95, 1.41), (-5.14, 0.04), (-4.95, -1.33), (-4.44, -2.57), (-3.63, -3.61), (-2.59, -4.42), (-1.35, -4.94), (0.02, -5.12), (1.39, -4.94), (2.62, -4.42), (3.66, -3.61), (4.47, -2.57), (4.99, -1.33)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(5.17, 0.04), (4.99, 1.41), (4.47, 2.64), (3.66, 3.68), (2.62, 4.49), (1.39, 5.01), (0.02, 5.19), (-1.35, 5.01), (-2.59, 4.49), (-3.63, 3.68), (-4.44, 2.64), (-4.95, 1.41), (-5.14, 0.04), (-4.95, -1.33), (-4.44, -2.57), (-3.63, -3.61), (-2.59, -4.42), (-1.35, -4.94), (0.02, -5.12), (1.39, -4.94), (2.62, -4.42), (3.66, -3.61), (4.47, -2.57), (4.99, -1.33)], closed: true, width: 1.3, white: false },
        GlyphPart::Fill { points: &[(3.0, 0.04), (2.59, 1.54), (1.52, 2.61), (0.02, 3.02), (-1.49, 2.61), (-2.56, 1.54), (-2.97, 0.04), (-2.56, -1.47), (-1.49, -2.54), (0.02, -2.95), (1.52, -2.54), (2.59, -1.47)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(3.0, 0.04), (2.59, 1.54), (1.52, 2.61), (0.02, 3.02), (-1.49, 2.61), (-2.56, 1.54), (-2.97, 0.04), (-2.56, -1.47), (-1.49, -2.54), (0.02, -2.95), (1.52, -2.54), (2.59, -1.47)], closed: true, width: 1.3, white: false },
        GlyphPart::Fill { points: &[(-2.4, 0.61), (-2.4, -0.54), (2.4, -0.54), (2.4, 0.61)], tris: &[], white: true },
        GlyphPart::Fill { points: &[(3.0, 4.92), (4.92, 3.0), (10.21, 8.28), (8.28, 10.21)], tris: &[], white: false },
    ],
};

/// The `Eyedropper` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (5.0, 27.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const EYEDROPPER: Glyph = Glyph {
    id: "Eyedropper",
    parts: &[
        GlyphPart::Fill { points: &[(-0.42, -0.42), (0.42, 0.42), (12.72, -10.88), (10.88, -12.72)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-0.42, -0.42), (0.42, 0.42), (12.72, -10.88), (10.88, -12.72)], closed: true, width: 0.9, white: false },
        GlyphPart::Fill { points: &[(9.51, -13.9), (13.9, -9.51), (12.49, -8.1), (8.1, -12.49)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(9.51, -13.9), (13.9, -9.51), (12.49, -8.1), (8.1, -12.49)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(10.62, -12.76), (14.47, -16.61), (15.54, -17.05), (16.61, -16.61), (17.06, -15.54), (16.61, -14.47), (12.76, -10.62)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(10.62, -12.76), (14.47, -16.61), (15.54, -17.05), (16.61, -16.61), (17.06, -15.54), (16.61, -14.47), (12.76, -10.62)], closed: true, width: 1.0, white: false },
    ],
};

/// The `AddStop` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const ADD_STOP: Glyph = Glyph {
    id: "AddStop",
    parts: &[
        GlyphPart::Fill { points: &[(17.39, 22.15), (17.39, 15.0), (17.62, 15.0), (17.62, 22.15)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(17.39, 22.15), (17.39, 15.0), (17.62, 15.0), (17.62, 22.15)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(13.93, 18.7), (13.93, 18.46), (21.08, 18.46), (21.08, 18.7)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(13.93, 18.7), (13.93, 18.46), (21.08, 18.46), (21.08, 18.7)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
    ],
};

/// The `RemoveStop` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const REMOVE_STOP: Glyph = Glyph {
    id: "RemoveStop",
    parts: &[
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(13.93, 18.7), (13.93, 18.46), (21.08, 18.46), (21.08, 18.7)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(13.93, 18.7), (13.93, 18.46), (21.08, 18.46), (21.08, 18.7)], closed: true, width: 1.0, white: false },
    ],
};

/// The `Slice` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (7.43, 23.32) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SLICE: Glyph = Glyph {
    id: "Slice",
    parts: &[
        GlyphPart::Fill { points: &[(12.93, -6.93), (7.5, -1.76), (0.67, -0.37), (9.91, -10.08)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(12.93, -6.93), (7.5, -1.76), (0.67, -0.37), (9.91, -10.08)], closed: true, width: 0.7, white: false },
        GlyphPart::Fill { points: &[(15.35, -16.43), (9.42, -10.57), (13.44, -6.38), (19.17, -12.4)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(15.35, -16.43), (9.42, -10.57), (13.44, -6.38), (19.17, -12.4)], closed: true, width: 0.7, white: false },
    ],
};

/// The `SliceSelect` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SLICE_SELECT: Glyph = Glyph {
    id: "SliceSelect",
    parts: &[
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(20.62, 20.8), (14.62, 20.8), (14.62, 20.6), (20.62, 20.6)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(20.62, 20.8), (14.62, 20.8), (14.62, 20.6), (20.62, 20.6)], closed: true, width: 2.0, white: false },
        GlyphPart::Fill { points: &[(20.62, 17.48), (14.62, 17.48), (14.62, 17.28), (20.62, 17.28)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(20.62, 17.48), (14.62, 17.48), (14.62, 17.28), (20.62, 17.28)], closed: true, width: 2.0, white: false },
    ],
};

/// The `Width` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const WIDTH: Glyph = Glyph {
    id: "Width",
    parts: &[
        GlyphPart::Fill { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], tris: &[[0, 3, 2], [2, 1, 0]], white: true },
        GlyphPart::Stroke { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(11.77, 18.5), (13.59, 18.27), (14.79, 17.72), (15.64, 17.07), (16.46, 16.52), (17.52, 16.29), (18.57, 16.52), (19.39, 17.07), (20.24, 17.72), (21.44, 18.27), (23.26, 18.5), (23.26, 18.86), (21.44, 19.09), (20.24, 19.64), (19.39, 20.29), (18.57, 20.84), (17.52, 21.07), (16.46, 20.84), (15.64, 20.29), (14.79, 19.64), (13.59, 19.09), (11.77, 18.86)], tris: &[[21, 0, 1], [2, 3, 4], [2, 4, 5], [2, 5, 6], [1, 2, 6], [1, 6, 7], [21, 1, 7], [21, 7, 8], [21, 8, 9], [21, 9, 10], [21, 10, 11], [21, 11, 12], [21, 12, 13], [13, 14, 15], [13, 15, 16], [13, 16, 17], [13, 17, 18], [13, 18, 19], [13, 19, 20], [13, 20, 21]], white: false },
    ],
};

/// The `WidthAdd` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const WIDTH_ADD: Glyph = Glyph {
    id: "WidthAdd",
    parts: &[
        GlyphPart::Fill { points: &[(17.42, 10.96), (17.42, 5.14), (17.61, 5.14), (17.61, 10.96)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(17.42, 10.96), (17.42, 5.14), (17.61, 5.14), (17.61, 10.96)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(14.6, 8.15), (14.6, 7.95), (20.43, 7.95), (20.43, 8.15)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(14.6, 8.15), (14.6, 7.95), (20.43, 7.95), (20.43, 8.15)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(11.77, 18.39), (13.59, 18.16), (14.79, 17.61), (15.64, 16.96), (16.46, 16.41), (17.52, 16.18), (18.57, 16.41), (19.39, 16.96), (20.24, 17.61), (21.44, 18.16), (23.26, 18.39), (23.26, 18.75), (21.44, 18.98), (20.24, 19.53), (19.39, 20.18), (18.57, 20.73), (17.52, 20.96), (16.46, 20.73), (15.64, 20.18), (14.79, 19.53), (13.59, 18.98), (11.77, 18.75)], tris: &[[21, 0, 1], [2, 3, 4], [2, 4, 5], [2, 5, 6], [1, 2, 6], [1, 6, 7], [21, 1, 7], [21, 7, 8], [21, 8, 9], [21, 9, 10], [21, 10, 11], [21, 11, 12], [21, 12, 13], [13, 14, 15], [13, 15, 16], [13, 16, 17], [13, 17, 18], [13, 18, 19], [13, 19, 20], [13, 20, 21]], white: false },
        GlyphPart::Fill { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], tris: &[[0, 3, 2], [2, 1, 0]], white: true },
        GlyphPart::Stroke { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], closed: true, width: 1.0, white: false },
    ],
};

/// The `WidthPoint` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const WIDTH_POINT: Glyph = Glyph {
    id: "WidthPoint",
    parts: &[
        GlyphPart::Fill { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], tris: &[[0, 3, 2], [2, 1, 0]], white: true },
        GlyphPart::Stroke { points: &[(5.14, 12.74), (11.66, 12.74), (0.33, 0.81), (0.46, 17.26)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(13.63, 22.35), (13.47, 22.2), (13.32, 22.35), (13.47, 22.51)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(13.63, 22.35), (13.47, 22.2), (13.32, 22.35), (13.47, 22.51)], closed: true, width: 2.0, white: false },
        GlyphPart::Fill { points: &[(21.44, 14.54), (21.29, 14.38), (21.13, 14.54), (21.29, 14.69)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(21.44, 14.54), (21.29, 14.38), (21.13, 14.54), (21.29, 14.69)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(13.47, 22.35), (21.1, 14.73)], closed: false, width: 0.5, white: false },
        GlyphPart::Fill { points: &[(11.89, 18.36), (13.71, 18.13), (14.9, 17.58), (15.76, 16.93), (16.57, 16.38), (17.63, 16.15), (18.69, 16.38), (19.51, 16.93), (20.36, 17.58), (21.56, 18.13), (23.38, 18.36), (23.38, 18.72), (21.56, 18.95), (20.36, 19.5), (19.51, 20.15), (18.69, 20.7), (17.63, 20.93), (16.57, 20.7), (15.76, 20.15), (14.9, 19.5), (13.71, 18.95), (11.89, 18.72)], tris: &[[21, 0, 1], [2, 3, 4], [2, 4, 5], [2, 5, 6], [1, 2, 6], [1, 6, 7], [21, 1, 7], [21, 7, 8], [21, 8, 9], [21, 9, 10], [21, 10, 11], [21, 11, 12], [21, 12, 13], [13, 14, 15], [13, 15, 16], [13, 16, 17], [13, 17, 18], [13, 18, 19], [13, 19, 20], [13, 20, 21]], white: false },
    ],
};

/// The `Blend` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (12.0, 12.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const BLEND: Glyph = Glyph {
    id: "Blend",
    parts: &[
        GlyphPart::Stroke { points: &[(-9.0, 0.0), (-2.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(2.0, 0.0), (9.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, -9.0), (0.0, -2.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, 2.0), (0.0, 9.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(8.68, 8.83), (12.83, 8.83), (12.83, 12.97), (8.68, 12.97)], closed: true, width: 1.0, white: false },
    ],
};

/// The `BlendObject` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (12.0, 12.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const BLEND_OBJECT: Glyph = Glyph {
    id: "BlendObject",
    parts: &[
        GlyphPart::Stroke { points: &[(-9.0, 0.0), (-2.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(2.0, 0.0), (9.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, -9.0), (0.0, -2.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, 2.0), (0.0, 9.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Fill { points: &[(8.84, 8.99), (12.66, 8.99), (12.66, 12.81), (8.84, 12.81)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(8.84, 8.99), (12.66, 8.99), (12.66, 12.81), (8.84, 12.81)], closed: true, width: 1.0, white: false },
    ],
};

/// The `BlendAnchor` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (12.0, 12.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const BLEND_ANCHOR: Glyph = Glyph {
    id: "BlendAnchor",
    parts: &[
        GlyphPart::Stroke { points: &[(-9.0, 0.0), (-2.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(2.0, 0.0), (9.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, -9.0), (0.0, -2.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, 2.0), (0.0, 9.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(13.87, 10.9), (13.63, 12.12), (12.96, 13.11), (11.97, 13.77), (10.75, 14.02), (9.54, 13.77), (8.55, 13.11), (7.88, 12.12), (7.64, 10.9), (7.88, 9.69), (8.55, 8.7), (9.54, 8.03), (10.75, 7.78), (11.97, 8.03), (12.96, 8.7), (13.63, 9.69)], closed: true, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(11.45, 10.9), (11.25, 11.4), (10.75, 11.6), (10.26, 11.4), (10.05, 10.9), (10.26, 10.41), (10.75, 10.2), (11.25, 10.41)], closed: true, width: 1.2, white: false },
    ],
};

/// The `PathBracket` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PATH_BRACKET: Glyph = Glyph {
    id: "PathBracket",
    parts: &[
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(17.39, 22.15), (17.39, 15.0), (17.62, 15.0), (17.62, 22.15)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(17.39, 22.15), (17.39, 15.0), (17.62, 15.0), (17.62, 22.15)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(20.26, 17.67), (20.26, 19.49), (21.39, 18.58)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(20.26, 17.67), (20.26, 19.49), (21.39, 18.58)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(14.75, 17.67), (14.75, 19.49), (13.62, 18.58)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(14.75, 17.67), (14.75, 19.49), (13.62, 18.58)], closed: true, width: 1.0, white: false },
    ],
};

/// The `TypeWidget` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const TYPE_WIDGET: Glyph = Glyph {
    id: "TypeWidget",
    parts: &[
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(14.73, 15.45), (14.26, 16.7), (14.31, 16.72), (15.31, 15.7), (16.55, 15.51), (17.0, 15.66), (17.14, 16.13), (17.14, 21.19), (16.97, 22.24), (16.35, 22.58), (16.13, 22.58), (16.13, 22.6), (18.64, 22.6), (18.64, 22.58), (18.42, 22.58), (17.81, 22.24), (17.63, 21.19), (17.63, 16.13), (17.77, 15.66), (18.22, 15.51), (19.47, 15.7), (20.46, 16.72), (20.51, 16.7), (20.04, 15.45)], tris: &[[23, 22, 21], [23, 21, 20], [23, 20, 19], [0, 23, 19], [14, 13, 12], [14, 12, 11], [14, 11, 10], [14, 10, 9], [15, 14, 9], [15, 9, 8], [16, 15, 8], [16, 8, 7], [17, 16, 7], [17, 7, 6], [18, 17, 6], [18, 6, 5], [19, 18, 5], [19, 5, 4], [0, 19, 4], [0, 4, 3], [0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(14.73, 15.45), (14.26, 16.7), (14.31, 16.72), (15.31, 15.7), (16.55, 15.51), (17.0, 15.66), (17.14, 16.13), (17.14, 21.19), (16.97, 22.24), (16.35, 22.58), (16.13, 22.58), (16.13, 22.6), (18.64, 22.6), (18.64, 22.58), (18.42, 22.58), (17.81, 22.24), (17.63, 21.19), (17.63, 16.13), (17.77, 15.66), (18.22, 15.51), (19.47, 15.7), (20.46, 16.72), (20.51, 16.7), (20.04, 15.45)], closed: true, width: 1.0, white: false },
    ],
};

/// The `ShapeBuilder` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (12.0, 12.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SHAPE_BUILDER: Glyph = Glyph {
    id: "ShapeBuilder",
    parts: &[
        GlyphPart::Stroke { points: &[(-9.0, 0.0), (-2.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(2.0, 0.0), (9.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, -9.0), (0.0, -2.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, 2.0), (0.0, 9.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Fill { points: &[(10.63, 14.47), (10.63, 7.33), (10.87, 7.33), (10.87, 14.47)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(10.63, 14.47), (10.63, 7.33), (10.87, 7.33), (10.87, 14.47)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(7.18, 11.02), (7.18, 10.78), (14.33, 10.78), (14.33, 11.02)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(7.18, 11.02), (7.18, 10.78), (14.33, 10.78), (14.33, 11.02)], closed: true, width: 1.0, white: false },
    ],
};

/// The `ShapeBuilderErase` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (12.0, 12.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SHAPE_BUILDER_ERASE: Glyph = Glyph {
    id: "ShapeBuilderErase",
    parts: &[
        GlyphPart::Stroke { points: &[(-9.0, 0.0), (-2.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(2.0, 0.0), (9.0, 0.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, -9.0), (0.0, -2.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(0.0, 2.0), (0.0, 9.0)], closed: false, width: 1.2, white: false },
        GlyphPart::Fill { points: &[(7.18, 11.02), (7.18, 10.78), (14.33, 10.78), (14.33, 11.02)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(7.18, 11.02), (7.18, 10.78), (14.33, 10.78), (14.33, 11.02)], closed: true, width: 1.0, white: false },
    ],
};

/// The `Lasso` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const LASSO: Glyph = Glyph {
    id: "Lasso",
    parts: &[
        GlyphPart::Stroke { points: &[(26.55, 17.97), (26.25, 19.15), (25.42, 20.18), (24.16, 20.99), (22.56, 21.52), (20.72, 21.72), (18.88, 21.52), (17.28, 20.99), (16.01, 20.18), (15.19, 19.15), (14.89, 17.97), (15.19, 16.78), (16.01, 15.75), (17.28, 14.94), (18.88, 14.41), (20.72, 14.22), (22.56, 14.41), (24.16, 14.94), (25.42, 15.75), (26.25, 16.78)], closed: true, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(15.72, 20.05), (15.35, 21.21), (15.46, 22.25), (15.63, 23.26), (15.44, 24.3), (14.47, 25.46)], closed: false, width: 1.2, white: false },
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
    ],
};

/// The `MagicWand` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (6.0, 6.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const MAGIC_WAND: Glyph = Glyph {
    id: "MagicWand",
    parts: &[
        GlyphPart::Stroke { points: &[(3.5, 3.5), (19.0, 19.0)], closed: false, width: 2.75, white: false },
        GlyphPart::Stroke { points: &[(4.25, 4.25), (6.75, 6.75)], closed: false, width: 1.25, white: true },
        GlyphPart::Fill { points: &[(0.0, -0.63), (0.13, -0.13), (0.63, 0.0), (0.13, 0.13), (0.0, 0.63), (-0.13, 0.13), (-0.63, 0.0), (-0.13, -0.13)], tris: &[[7, 0, 1], [7, 1, 2], [7, 2, 3], [7, 3, 4], [7, 4, 5], [5, 6, 7]], white: false },
        GlyphPart::Stroke { points: &[(0.0, -0.63), (0.13, -0.13), (0.63, 0.0), (0.13, 0.13), (0.0, 0.63), (-0.13, 0.13), (-0.63, 0.0), (-0.13, -0.13)], closed: true, width: 2.0, white: false },
        GlyphPart::Stroke { points: &[(1.07, 8.61), (-1.57, 11.25)], closed: false, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(8.76, -3.72), (6.12, -1.09)], closed: false, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(-1.57, 8.61), (1.07, 11.25)], closed: false, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(6.12, -3.72), (8.76, -1.09)], closed: false, width: 1.0, white: false },
    ],
};

/// The `Pencil` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (5.0, 27.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PENCIL: Glyph = Glyph {
    id: "Pencil",
    parts: &[
        GlyphPart::Fill { points: &[(0.14, -0.07), (0.07, -0.15), (2.5, -5.33), (3.04, -5.5), (5.5, -2.99), (5.33, -2.5)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(0.14, -0.07), (0.07, -0.15), (2.5, -5.33), (3.04, -5.5), (5.5, -2.99), (5.33, -2.5)], closed: true, width: 0.9, white: false },
        GlyphPart::Fill { points: &[(0.0, 0.0), (0.91, -2.01), (2.01, -0.91)], tris: &[], white: false },
        GlyphPart::Fill { points: &[(11.1, -13.96), (13.96, -11.1), (5.72, -2.86), (2.86, -5.72)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(11.1, -13.96), (13.96, -11.1), (5.72, -2.86), (2.86, -5.72)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(11.7, -14.56), (13.29, -16.14), (14.08, -16.46), (14.87, -16.14), (16.14, -14.87), (16.47, -14.08), (16.14, -13.28), (14.56, -11.7)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(11.7, -14.56), (13.29, -16.14), (14.08, -16.46), (14.87, -16.14), (16.14, -14.87), (16.47, -14.08), (16.14, -13.28), (14.56, -11.7)], closed: true, width: 1.0, white: false },
    ],
};

/// The `Paintbrush` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (5.0, 27.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const PAINTBRUSH: Glyph = Glyph {
    id: "Paintbrush",
    parts: &[
        GlyphPart::Fill { points: &[(7.81, -11.21), (10.34, -9.32), (7.22, -5.11), (4.77, -7.28)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(7.81, -11.21), (10.34, -9.32), (7.22, -5.11), (4.77, -7.28)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(7.81, -11.21), (10.34, -9.32), (16.36, -18.44), (16.56, -19.11), (16.23, -19.67), (15.56, -19.92), (14.87, -19.57)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(7.81, -11.21), (10.34, -9.32), (16.36, -18.44), (16.56, -19.11), (16.23, -19.67), (15.56, -19.92), (14.87, -19.57)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(0.0, 0.0), (0.78, -0.8), (1.17, -1.69), (1.33, -2.65), (1.42, -3.63), (1.59, -4.61), (2.02, -5.57), (2.86, -6.47), (4.28, -7.28), (7.39, -4.98), (7.07, -3.33), (6.44, -2.11), (5.55, -1.25), (4.43, -0.68), (3.11, -0.33), (1.62, -0.12)], tris: &[[15, 0, 1], [15, 1, 2], [15, 2, 3], [15, 3, 4], [15, 4, 5], [15, 5, 6], [15, 6, 7], [15, 7, 8], [15, 8, 9], [15, 9, 10], [15, 10, 11], [15, 11, 12], [15, 12, 13], [13, 14, 15]], white: false },
    ],
};

/// The `Smooth` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (5.0, 27.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SMOOTH: Glyph = Glyph {
    id: "Smooth",
    parts: &[
        GlyphPart::Stroke { points: &[(5.56, 0.58), (10.07, -1.54)], closed: false, width: 0.9, white: false },
        GlyphPart::Stroke { points: &[(9.66, 1.12), (15.29, -1.54)], closed: false, width: 0.9, white: false },
        GlyphPart::Stroke { points: &[(13.49, 2.23), (20.25, -0.96)], closed: false, width: 0.9, white: false },
        GlyphPart::Fill { points: &[(0.74, -0.85), (5.56, -8.62), (17.34, -22.55), (21.19, -19.06), (8.71, -5.53), (2.82, -0.96)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(0.74, -0.85), (5.56, -8.62), (17.34, -22.55), (21.19, -19.06), (8.71, -5.53), (2.82, -0.96)], closed: true, width: 1.3, white: false },
        GlyphPart::Stroke { points: &[(3.1, -4.64), (7.82, -4.64)], closed: false, width: 1.3, white: false },
        GlyphPart::Stroke { points: &[(7.95, -11.18), (13.57, -11.18)], closed: false, width: 1.3, white: false },
        GlyphPart::Stroke { points: &[(10.68, -14.49), (16.87, -14.49)], closed: false, width: 1.3, white: false },
        GlyphPart::Stroke { points: &[(13.49, -17.8), (20.25, -17.8)], closed: false, width: 1.3, white: false },
        GlyphPart::Stroke { points: &[(5.14, -7.87), (10.76, -7.87)], closed: false, width: 1.3, white: false },
    ],
};

/// The `Knife` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (5.0, 27.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const KNIFE: Glyph = Glyph {
    id: "Knife",
    parts: &[
        GlyphPart::Fill { points: &[(0.6, -0.66), (2.55, -0.66), (2.55, -2.61), (4.51, -2.61), (4.51, -4.57), (6.46, -4.57), (6.46, -6.52), (8.41, -6.52), (8.41, -8.47), (10.37, -8.47), (10.37, -10.42), (12.32, -10.42), (12.32, -12.38), (10.65, -14.05), (0.6, -4.0)], tris: &[[0, 14, 13], [13, 12, 11], [13, 11, 10], [10, 9, 8], [13, 10, 8], [0, 13, 8], [8, 7, 6], [6, 5, 4], [8, 6, 4], [0, 8, 4], [4, 3, 2], [0, 4, 2], [2, 1, 0]], white: true },
        GlyphPart::Stroke { points: &[(0.6, -0.66), (2.55, -0.66), (2.55, -2.61), (4.51, -2.61), (4.51, -4.57), (6.46, -4.57), (6.46, -6.52), (8.41, -6.52), (8.41, -8.47), (10.37, -8.47), (10.37, -10.42), (12.32, -10.42), (12.32, -12.38), (10.65, -14.05), (0.6, -4.0)], closed: true, width: 0.7, white: false },
        GlyphPart::Fill { points: &[(14.25, -18.71), (18.19, -18.71), (18.19, -15.15), (15.56, -13.04), (15.48, -11.81), (14.27, -10.47), (9.91, -14.83), (11.23, -16.15), (12.34, -16.37)], tris: &[[8, 0, 1], [8, 1, 2], [8, 2, 3], [8, 3, 4], [8, 4, 5], [8, 5, 6], [6, 7, 8]], white: false },
        GlyphPart::Stroke { points: &[(14.25, -18.71), (18.19, -18.71), (18.19, -15.15), (15.56, -13.04), (15.48, -11.81), (14.27, -10.47), (9.91, -14.83), (11.23, -16.15), (12.34, -16.37)], closed: true, width: 1.0, white: false },
    ],
};

/// The `LivePaintBucket` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (9.0, 26.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const LIVE_PAINT_BUCKET: Glyph = Glyph {
    id: "LivePaintBucket",
    parts: &[
        GlyphPart::Fill { points: &[(1.81, -10.35), (9.22, -19.28), (15.79, -12.19), (15.82, -11.63), (15.49, -10.1), (14.19, -7.84), (12.44, -6.04), (11.11, -5.27), (10.26, -5.13), (9.97, -5.17)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(1.81, -10.35), (9.22, -19.28), (15.79, -12.19), (15.82, -11.63), (15.49, -10.1), (14.19, -7.84), (12.44, -6.04), (11.11, -5.27), (10.26, -5.13), (9.97, -5.17)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(8.82, -20.0), (9.07, -19.38), (8.9, -18.37), (8.34, -17.09), (7.45, -15.61), (6.27, -14.05), (4.94, -12.61), (3.64, -11.47), (2.47, -10.71), (1.52, -10.36), (0.87, -10.5), (0.62, -11.12), (0.79, -12.12), (1.35, -13.41), (2.24, -14.88), (3.42, -16.44), (4.75, -17.88), (6.05, -19.02), (7.22, -19.79), (8.17, -20.13)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(8.82, -20.0), (9.07, -19.38), (8.9, -18.37), (8.34, -17.09), (7.45, -15.61), (6.27, -14.05), (4.94, -12.61), (3.64, -11.47), (2.47, -10.71), (1.52, -10.36), (0.87, -10.5), (0.62, -11.12), (0.79, -12.12), (1.35, -13.41), (2.24, -14.88), (3.42, -16.44), (4.75, -17.88), (6.05, -19.02), (7.22, -19.79), (8.17, -20.13)], closed: true, width: 1.3, white: false },
        GlyphPart::Fill { points: &[(1.07, -10.8), (1.23, -12.2), (3.68, -12.2), (1.23, -10.75), (1.23, -3.42), (0.96, -1.42), (0.22, -0.55), (-0.42, -1.15), (-0.5, -2.91), (-0.27, -4.5), (0.12, -6.55), (0.56, -8.59), (0.92, -10.17)], tris: &[[0, 1, 2], [0, 2, 3], [12, 0, 3], [12, 3, 4], [12, 4, 5], [12, 5, 6], [12, 6, 7], [12, 7, 8], [12, 8, 9], [12, 9, 10], [10, 11, 12]], white: false },
        GlyphPart::Stroke { points: &[(1.07, -10.8), (1.23, -12.2), (3.68, -12.2), (1.23, -10.75), (1.23, -3.42), (0.96, -1.42), (0.22, -0.55), (-0.42, -1.15), (-0.5, -2.91), (-0.27, -4.5), (0.12, -6.55), (0.56, -8.59), (0.92, -10.17)], closed: true, width: 1.0, white: false },
    ],
};

/// The `Measure` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (8.0, 8.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const MEASURE: Glyph = Glyph {
    id: "Measure",
    parts: &[
        GlyphPart::Fill { points: &[(-0.12, 3.57), (-0.12, -3.57), (0.12, -3.57), (0.12, 3.57)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-0.12, 3.57), (-0.12, -3.57), (0.12, -3.57), (0.12, 3.57)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-3.57, 0.12), (-3.57, -0.12), (3.57, -0.12), (3.57, 0.12)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-3.57, 0.12), (-3.57, -0.12), (3.57, -0.12), (3.57, 0.12)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(13.4, 3.16), (15.88, 6.02), (4.33, 16.02), (1.86, 13.16)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(13.4, 3.16), (15.88, 6.02), (4.33, 16.02), (1.86, 13.16)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(11.98, 7.01), (10.63, 5.45)], closed: false, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(9.15, 9.46), (7.8, 7.9)], closed: false, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(6.31, 11.92), (4.96, 10.36)], closed: false, width: 1.0, white: false },
    ],
};

/// The `SymbolSprayer` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (8.0, 8.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SYMBOL_SPRAYER: Glyph = Glyph {
    id: "SymbolSprayer",
    parts: &[
        GlyphPart::Fill { points: &[(8.0, 5.0), (13.5, 5.0), (14.21, 5.29), (14.5, 6.0), (14.5, 18.0), (14.21, 18.71), (13.5, 19.0), (8.0, 19.0), (7.29, 18.71), (7.0, 18.0), (7.0, 6.0), (7.29, 5.29)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(8.0, 5.0), (13.5, 5.0), (14.21, 5.29), (14.5, 6.0), (14.5, 18.0), (14.21, 18.71), (13.5, 19.0), (8.0, 19.0), (7.29, 18.71), (7.0, 18.0), (7.0, 6.0), (7.29, 5.29)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(8.74, 1.15), (12.76, 1.15), (12.76, 5.0), (8.74, 5.0)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(8.74, 1.15), (12.76, 1.15), (12.76, 5.0), (8.74, 5.0)], closed: true, width: 0.9, white: false },
        GlyphPart::Fill { points: &[(5.75, 2.33), (8.25, 2.33), (8.25, 3.33), (5.75, 3.33)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(5.75, 2.33), (8.25, 2.33), (8.25, 3.33), (5.75, 3.33)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(0.94, 0.0), (0.67, 0.67), (0.0, 0.94), (-0.67, 0.67), (-0.94, 0.0), (-0.67, -0.67), (0.0, -0.94), (0.67, -0.67)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(0.94, 0.0), (0.67, 0.67), (0.0, 0.94), (-0.67, 0.67), (-0.94, 0.0), (-0.67, -0.67), (0.0, -0.94), (0.67, -0.67)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-2.92, -2.0), (-3.09, -1.59), (-3.5, -1.42), (-3.91, -1.59), (-4.08, -2.0), (-3.91, -2.41), (-3.5, -2.58), (-3.09, -2.41)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-2.92, -2.0), (-3.09, -1.59), (-3.5, -1.42), (-3.91, -1.59), (-4.08, -2.0), (-3.91, -2.41), (-3.5, -2.58), (-3.09, -2.41)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-1.0, 4.5), (-1.15, 4.85), (-1.5, 5.0), (-1.85, 4.85), (-2.0, 4.5), (-1.85, 4.15), (-1.5, 4.0), (-1.15, 4.15)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-1.0, 4.5), (-1.15, 4.85), (-1.5, 5.0), (-1.85, 4.85), (-2.0, 4.5), (-1.85, 4.15), (-1.5, 4.0), (-1.15, 4.15)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(3.65, -3.5), (3.46, -3.04), (3.0, -2.85), (2.54, -3.04), (2.35, -3.5), (2.54, -3.96), (3.0, -4.15), (3.46, -3.96)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(3.65, -3.5), (3.46, -3.04), (3.0, -2.85), (2.54, -3.04), (2.35, -3.5), (2.54, -3.96), (3.0, -4.15), (3.46, -3.96)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-3.92, 2.5), (-3.97, 2.72), (-4.09, 2.91), (-4.28, 3.03), (-4.5, 3.08), (-4.91, 2.91), (-5.08, 2.5), (-4.91, 2.09), (-4.5, 1.92), (-4.28, 1.97), (-4.09, 2.09), (-3.97, 2.28)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-3.92, 2.5), (-3.97, 2.72), (-4.09, 2.91), (-4.28, 3.03), (-4.5, 3.08), (-4.91, 2.91), (-5.08, 2.5), (-4.91, 2.09), (-4.5, 1.92), (-4.28, 1.97), (-4.09, 2.09), (-3.97, 2.28)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(3.0, 2.5), (2.96, 2.69), (2.85, 2.85), (2.69, 2.96), (2.5, 3.0), (2.31, 2.96), (2.15, 2.85), (2.04, 2.69), (2.0, 2.5), (2.04, 2.31), (2.15, 2.15), (2.31, 2.04), (2.5, 2.0), (2.69, 2.04), (2.85, 2.15), (2.96, 2.31)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(3.0, 2.5), (2.96, 2.69), (2.85, 2.85), (2.69, 2.96), (2.5, 3.0), (2.31, 2.96), (2.15, 2.85), (2.04, 2.69), (2.0, 2.5), (2.04, 2.31), (2.15, 2.15), (2.31, 2.04), (2.5, 2.0), (2.69, 2.04), (2.85, 2.15), (2.96, 2.31)], closed: true, width: 1.0, white: false },
    ],
};

/// The `SymbolShifter` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SYMBOL_SHIFTER: Glyph = Glyph {
    id: "SymbolShifter",
    parts: &[
        GlyphPart::Fill { points: &[(-7.37, 0.0), (-7.85, 1.15), (-9.0, 1.63), (-10.15, 1.15), (-10.63, 0.0), (-10.15, -1.15), (-9.0, -1.63), (-7.85, -1.15)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-7.37, 0.0), (-7.85, 1.15), (-9.0, 1.63), (-10.15, 1.15), (-10.63, 0.0), (-10.15, -1.15), (-9.0, -1.63), (-7.85, -1.15)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-1.61, 0.0), (-2.08, 1.15), (-3.23, 1.63), (-4.39, 1.15), (-4.86, 0.0), (-4.39, -1.15), (-3.23, -1.63), (-2.08, -1.15)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-1.61, 0.0), (-2.08, 1.15), (-3.23, 1.63), (-4.39, 1.15), (-4.86, 0.0), (-4.39, -1.15), (-3.23, -1.63), (-2.08, -1.15)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(4.16, 0.0), (3.68, 1.15), (2.53, 1.63), (1.38, 1.15), (0.9, 0.0), (1.38, -1.15), (2.53, -1.63), (3.68, -1.15)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(4.16, 0.0), (3.68, 1.15), (2.53, 1.63), (1.38, 1.15), (0.9, 0.0), (1.38, -1.15), (2.53, -1.63), (3.68, -1.15)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(-7.0, 5.86), (7.8, 5.86), (7.8, 6.14), (-7.0, 6.14)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(7.72, 7.18), (7.72, 4.82), (9.72, 6.0)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(7.72, 7.18), (7.72, 4.82), (9.72, 6.0)], closed: true, width: 1.0, white: false },
    ],
};

/// The `SymbolScruncher` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SYMBOL_SCRUNCHER: Glyph = Glyph {
    id: "SymbolScruncher",
    parts: &[
        GlyphPart::Stroke { points: &[(-7.62, 7.42), (-5.2, 5.0), (-5.0, 5.2), (-7.42, 7.62)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(-7.42, -7.62), (-5.0, -5.2), (-5.2, -5.0), (-7.62, -7.42)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(7.62, -7.42), (5.2, -5.0), (5.0, -5.2), (7.42, -7.62)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(7.42, 7.62), (5.0, 5.2), (5.2, 5.0), (7.62, 7.42)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(1.79, 0.0), (1.27, 1.27), (0.0, 1.79), (-1.27, 1.27), (-1.79, 0.0), (-1.27, -1.27), (0.0, -1.79), (1.27, -1.27)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(1.79, 0.0), (1.27, 1.27), (0.0, 1.79), (-1.27, 1.27), (-1.79, 0.0), (-1.27, -1.27), (0.0, -1.79), (1.27, -1.27)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-4.32, 6.0), (-6.0, 4.32), (-4.11, 4.11)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-4.32, 6.0), (-6.0, 4.32), (-4.11, 4.11)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-6.0, -4.32), (-4.32, -6.0), (-4.11, -4.11)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-6.0, -4.32), (-4.32, -6.0), (-4.11, -4.11)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(4.32, -6.0), (6.0, -4.32), (4.11, -4.11)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(4.32, -6.0), (6.0, -4.32), (4.11, -4.11)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(6.0, 4.32), (4.32, 6.0), (4.11, 4.11)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(6.0, 4.32), (4.32, 6.0), (4.11, 4.11)], closed: true, width: 1.0, white: false },
    ],
};

/// The `SymbolSizer` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SYMBOL_SIZER: Glyph = Glyph {
    id: "SymbolSizer",
    parts: &[
        GlyphPart::Stroke { points: &[(-9.0, -9.0), (9.0, -9.0), (9.0, 9.0), (-9.0, 9.0)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-6.15, 1.85), (-1.85, 1.85), (-1.85, 6.15), (-6.15, 6.15)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-6.15, 1.85), (-1.85, 1.85), (-1.85, 6.15), (-6.15, 6.15)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(0.4, -0.6), (5.25, -5.45), (5.45, -5.25), (0.6, -0.4)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(6.13, -4.45), (4.45, -6.13), (6.34, -6.34)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(6.13, -4.45), (4.45, -6.13), (6.34, -6.34)], closed: true, width: 1.0, white: false },
    ],
};

/// The `SymbolSpinner` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SYMBOL_SPINNER: Glyph = Glyph {
    id: "SymbolSpinner",
    parts: &[
        GlyphPart::Fill { points: &[(2.25, 0.0), (1.94, 1.14), (1.14, 1.94), (0.0, 2.25), (-1.14, 1.94), (-1.94, 1.14), (-2.25, 0.0), (-1.94, -1.14), (-1.14, -1.94), (0.0, -2.25), (1.14, -1.94), (1.94, -1.14)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(2.25, 0.0), (1.94, 1.14), (1.14, 1.94), (0.0, 2.25), (-1.14, 1.94), (-1.94, 1.14), (-2.25, 0.0), (-1.94, -1.14), (-1.14, -1.94), (0.0, -2.25), (1.14, -1.94), (1.94, -1.14)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(0.0, -7.0), (-1.61, -6.82), (-3.08, -6.29), (-4.38, -5.46), (-5.46, -4.38), (-6.29, -3.08), (-6.82, -1.61), (-7.0, 0.0), (-6.82, 1.61), (-6.29, 3.08), (-5.46, 4.38), (-4.38, 5.46), (-3.08, 6.29), (-1.61, 6.82), (0.0, 7.0), (1.61, 6.82), (3.08, 6.29), (4.38, 5.46), (5.46, 4.38), (6.29, 3.08), (6.82, 1.61), (7.0, 0.0), (6.62, 0.0), (6.44, 1.52), (5.94, 2.91), (5.16, 4.14), (4.14, 5.16), (2.91, 5.94), (1.52, 6.44), (0.0, 6.62), (-1.52, 6.44), (-2.91, 5.94), (-4.14, 5.16), (-5.16, 4.14), (-5.94, 2.91), (-6.44, 1.52), (-6.62, 0.0), (-6.44, -1.52), (-5.94, -2.91), (-5.16, -4.14), (-4.14, -5.16), (-2.91, -5.94), (-1.52, -6.44), (0.0, -6.62)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(2.77, -7.0), (-0.21, -5.14), (-0.21, -8.86)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(2.77, -7.0), (-0.21, -5.14), (-0.21, -8.86)], closed: true, width: 1.0, white: false },
    ],
};

/// The `SymbolStainer` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 23.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SYMBOL_STAINER: Glyph = Glyph {
    id: "SymbolStainer",
    parts: &[
        GlyphPart::Fill { points: &[(0.0, -18.0), (0.88, -16.34), (1.77, -14.79), (2.61, -13.33), (3.36, -11.94), (3.96, -10.59), (4.36, -9.29), (4.5, -8.0), (4.27, -6.57), (3.64, -5.34), (2.66, -4.36), (1.43, -3.73), (0.0, -3.5), (-1.43, -3.73), (-2.66, -4.36), (-3.64, -5.34), (-4.27, -6.57), (-4.5, -8.0), (-4.36, -9.29), (-3.96, -10.59), (-3.36, -11.94), (-2.61, -13.33), (-1.77, -14.79), (-0.88, -16.34)], tris: &[[23, 0, 1], [23, 1, 2], [23, 2, 3], [23, 3, 4], [23, 4, 5], [23, 5, 6], [23, 6, 7], [23, 7, 8], [23, 8, 9], [23, 9, 10], [23, 10, 11], [23, 11, 12], [23, 12, 13], [23, 13, 14], [23, 14, 15], [23, 15, 16], [23, 16, 17], [23, 17, 18], [23, 18, 19], [23, 19, 20], [20, 21, 22], [20, 22, 23]], white: false },
        GlyphPart::Stroke { points: &[(0.0, -18.0), (0.88, -16.34), (1.77, -14.79), (2.61, -13.33), (3.36, -11.94), (3.96, -10.59), (4.36, -9.29), (4.5, -8.0), (4.27, -6.57), (3.64, -5.34), (2.66, -4.36), (1.43, -3.73), (0.0, -3.5), (-1.43, -3.73), (-2.66, -4.36), (-3.64, -5.34), (-4.27, -6.57), (-4.5, -8.0), (-4.36, -9.29), (-3.96, -10.59), (-3.36, -11.94), (-2.61, -13.33), (-1.77, -14.79), (-0.88, -16.34)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(7.5, 1.5), (7.23, 2.16), (6.48, 2.76), (5.3, 3.27), (3.79, 3.66), (1.99, 3.91), (0.0, 4.0), (-1.99, 3.91), (-3.79, 3.66), (-5.3, 3.27), (-6.48, 2.76), (-7.23, 2.16), (-7.5, 1.5), (-7.23, 0.84), (-6.48, 0.24), (-5.3, -0.27), (-3.79, -0.66), (-1.99, -0.91), (0.0, -1.0), (1.99, -0.91), (3.79, -0.66), (5.3, -0.27), (6.48, 0.24), (7.23, 0.84)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(7.5, 1.5), (7.23, 2.16), (6.48, 2.76), (5.3, 3.27), (3.79, 3.66), (1.99, 3.91), (0.0, 4.0), (-1.99, 3.91), (-3.79, 3.66), (-5.3, 3.27), (-6.48, 2.76), (-7.23, 2.16), (-7.5, 1.5), (-7.23, 0.84), (-6.48, 0.24), (-5.3, -0.27), (-3.79, -0.66), (-1.99, -0.91), (0.0, -1.0), (1.99, -0.91), (3.79, -0.66), (5.3, -0.27), (6.48, 0.24), (7.23, 0.84)], closed: true, width: 1.0, white: false },
    ],
};

/// The `SymbolScreener` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SYMBOL_SCREENER: Glyph = Glyph {
    id: "SymbolScreener",
    parts: &[
        GlyphPart::Fill { points: &[(-10.0, -10.0), (2.0, -10.0), (2.0, 2.0), (-10.0, 2.0)], tris: &[], white: false },
        GlyphPart::Fill { points: &[(-4.0, -4.0), (10.0, -4.0), (10.0, 10.0), (-4.0, 10.0)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-4.0, -4.0), (10.0, -4.0), (10.0, 10.0), (-4.0, 10.0)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-2.28, -2.23), (-1.13, -2.23), (-1.13, -1.07), (-2.28, -1.07)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-2.28, -2.23), (-1.13, -2.23), (-1.13, -1.07), (-2.28, -1.07)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-2.28, 2.41), (-1.13, 2.41), (-1.13, 3.57), (-2.28, 3.57)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-2.28, 2.41), (-1.13, 2.41), (-1.13, 3.57), (-2.28, 3.57)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-2.28, 7.05), (-1.13, 7.05), (-1.13, 8.21), (-2.28, 8.21)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-2.28, 7.05), (-1.13, 7.05), (-1.13, 8.21), (-2.28, 8.21)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(0.02, 0.09), (1.17, 0.09), (1.17, 1.25), (0.02, 1.25)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(0.02, 0.09), (1.17, 0.09), (1.17, 1.25), (0.02, 1.25)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(0.02, 4.73), (1.17, 4.73), (1.17, 5.89), (0.02, 5.89)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(0.02, 4.73), (1.17, 4.73), (1.17, 5.89), (0.02, 5.89)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(2.36, -2.23), (3.51, -2.23), (3.51, -1.07), (2.36, -1.07)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(2.36, -2.23), (3.51, -2.23), (3.51, -1.07), (2.36, -1.07)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(7.0, -2.23), (8.14, -2.23), (8.14, -1.07), (7.0, -1.07)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(7.0, -2.23), (8.14, -2.23), (8.14, -1.07), (7.0, -1.07)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(2.36, 2.41), (3.51, 2.41), (3.51, 3.57), (2.36, 3.57)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(2.36, 2.41), (3.51, 2.41), (3.51, 3.57), (2.36, 3.57)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(7.0, 2.41), (8.14, 2.41), (8.14, 3.57), (7.0, 3.57)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(7.0, 2.41), (8.14, 2.41), (8.14, 3.57), (7.0, 3.57)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(2.36, 7.05), (3.51, 7.05), (3.51, 8.21), (2.36, 8.21)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(2.36, 7.05), (3.51, 7.05), (3.51, 8.21), (2.36, 8.21)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(7.0, 7.05), (8.14, 7.05), (8.14, 8.21), (7.0, 8.21)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(7.0, 7.05), (8.14, 7.05), (8.14, 8.21), (7.0, 8.21)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(4.61, 0.09), (5.76, 0.09), (5.76, 1.25), (4.61, 1.25)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(4.61, 0.09), (5.76, 0.09), (5.76, 1.25), (4.61, 1.25)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(4.61, 4.73), (5.76, 4.73), (5.76, 5.89), (4.61, 5.89)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(4.61, 4.73), (5.76, 4.73), (5.76, 5.89), (4.61, 5.89)], closed: true, width: 1.0, white: false },
    ],
};

/// The `SymbolStyler` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SYMBOL_STYLER: Glyph = Glyph {
    id: "SymbolStyler",
    parts: &[
        GlyphPart::Fill { points: &[(-8.2, -8.2), (-1.8, -8.2), (-1.8, -1.8), (-8.2, -1.8)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-8.2, -8.2), (-1.8, -8.2), (-1.8, -1.8), (-8.2, -1.8)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(1.0, -9.0), (9.0, -9.0), (9.0, -1.0), (1.0, -1.0)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(1.0, -9.0), (9.0, -9.0), (9.0, -1.0), (1.0, -1.0)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-9.0, 1.0), (-1.0, 1.0), (-1.0, 9.0), (-9.0, 9.0)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-9.0, 1.0), (-1.0, 1.0), (-1.0, 9.0), (-9.0, 9.0)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(1.8, 1.8), (8.2, 1.8), (8.2, 8.2), (1.8, 8.2)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(1.8, 1.8), (8.2, 1.8), (8.2, 8.2), (1.8, 8.2)], closed: true, width: 1.0, white: false },
    ],
};

/// The `TypeVertical` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (16.0, 16.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const TYPE_VERTICAL: Glyph = Glyph {
    id: "TypeVertical",
    parts: &[
        GlyphPart::Fill { points: &[(-0.05, 1.44), (-0.05, -1.44), (0.05, -1.44), (0.05, 1.44)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-0.05, 1.44), (-0.05, -1.44), (0.05, -1.44), (0.05, 1.44)], closed: true, width: 1.1, white: false },
        GlyphPart::Fill { points: &[(-10.42, -0.05), (4.92, -0.05), (4.92, 0.05), (-10.42, 0.05)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-10.42, -0.05), (4.92, -0.05), (4.92, 0.05), (-10.42, 0.05)], closed: true, width: 1.1, white: false },
        GlyphPart::Fill { points: &[(-11.05, 2.46), (-11.05, -2.46), (-10.95, -2.46), (-10.95, 2.46)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(-11.05, 2.46), (-11.05, -2.46), (-10.95, -2.46), (-10.95, 2.46)], closed: true, width: 1.1, white: false },
        GlyphPart::Fill { points: &[(5.0, 2.46), (5.0, -2.46), (5.1, -2.46), (5.1, 2.46)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(5.0, 2.46), (5.0, -2.46), (5.1, -2.46), (5.1, 2.46)], closed: true, width: 1.1, white: false },
    ],
};

/// The `Scissors` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (8.0, 8.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SCISSORS: Glyph = Glyph {
    id: "Scissors",
    parts: &[
        GlyphPart::Fill { points: &[(-0.12, 3.57), (-0.12, -3.57), (0.12, -3.57), (0.12, 3.57)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-0.12, 3.57), (-0.12, -3.57), (0.12, -3.57), (0.12, 3.57)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-3.57, 0.12), (-3.57, -0.12), (3.57, -0.12), (3.57, 0.12)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-3.57, 0.12), (-3.57, -0.12), (3.57, -0.12), (3.57, 0.12)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(4.87, 8.48), (4.24, 7.44), (4.52, 6.25), (5.56, 5.61), (6.74, 5.9), (7.38, 6.94), (7.1, 8.12), (6.06, 8.76)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(5.06, 11.96), (4.43, 13.01), (4.74, 14.19), (5.78, 14.81), (6.97, 14.51), (7.59, 13.47), (7.29, 12.28), (6.24, 11.66)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(16.75, 14.74), (6.98, 8.42), (7.35, 7.91), (16.79, 14.74)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(16.75, 14.74), (6.98, 8.42), (7.35, 7.91), (16.79, 14.74)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(16.84, 5.52), (7.16, 11.98), (7.54, 12.49), (16.88, 5.52)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(16.84, 5.52), (7.16, 11.98), (7.54, 12.49), (16.88, 5.52)], closed: true, width: 1.0, white: false },
    ],
};

/// The `ScissorsNotAllowed` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (8.0, 8.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const SCISSORS_NOT_ALLOWED: Glyph = Glyph {
    id: "ScissorsNotAllowed",
    parts: &[
        GlyphPart::Fill { points: &[(-0.12, 3.57), (-0.12, -3.57), (0.12, -3.57), (0.12, 3.57)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-0.12, 3.57), (-0.12, -3.57), (0.12, -3.57), (0.12, 3.57)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(-3.57, 0.12), (-3.57, -0.12), (3.57, -0.12), (3.57, 0.12)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(-3.57, 0.12), (-3.57, -0.12), (3.57, -0.12), (3.57, 0.12)], closed: true, width: 1.0, white: false },
        GlyphPart::Stroke { points: &[(6.55, 9.99), (6.77, 8.6), (7.4, 7.39), (8.35, 6.44), (9.55, 5.82), (10.94, 5.59), (12.33, 5.82), (13.53, 6.44), (14.49, 7.39), (15.11, 8.6), (15.33, 9.99), (15.11, 11.37), (14.49, 12.58), (13.53, 13.53), (12.33, 14.15), (10.94, 14.38), (9.55, 14.15), (8.35, 13.53), (7.4, 12.58), (6.77, 11.37)], closed: true, width: 1.2, white: false },
        GlyphPart::Stroke { points: &[(13.96, 7.05), (7.91, 13.1)], closed: false, width: 1.2, white: false },
    ],
};

/// The `MeshAdd` cursor, drawn by Michael Gieson.
/// Points relative to the hotspot, at (4.0, 4.0) in the drawing.
#[rustfmt::skip]
#[allow(clippy::approx_constant, reason = "generated coordinates, such as 6.28")]
const MESH_ADD: Glyph = Glyph {
    id: "MeshAdd",
    parts: &[
        GlyphPart::Fill { points: &[(16.06, 14.93), (24.06, 14.93), (24.06, 22.93), (16.06, 22.93)], tris: &[], white: true },
        GlyphPart::Stroke { points: &[(16.06, 14.93), (24.06, 14.93), (24.06, 22.93), (16.06, 22.93)], closed: true, width: 0.9, white: false },
        GlyphPart::Stroke { points: &[(18.73, 14.93), (19.15, 16.27), (19.06, 17.6), (18.73, 18.93), (18.4, 20.27), (18.31, 21.6), (18.73, 22.93)], closed: false, width: 0.75, white: false },
        GlyphPart::Stroke { points: &[(16.06, 18.93), (17.4, 18.52), (18.73, 18.6), (20.06, 18.93), (21.4, 19.27), (22.73, 19.35), (24.06, 18.93)], closed: false, width: 0.75, white: false },
        GlyphPart::Fill { points: &[(19.52, 6.19), (19.52, -0.96), (19.76, -0.96), (19.76, 6.19)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(19.52, 6.19), (19.52, -0.96), (19.76, -0.96), (19.76, 6.19)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(16.06, 2.73), (16.06, 2.49), (23.21, 2.49), (23.21, 2.73)], tris: &[], white: false },
        GlyphPart::Stroke { points: &[(16.06, 2.73), (16.06, 2.49), (23.21, 2.49), (23.21, 2.73)], closed: true, width: 1.0, white: false },
        GlyphPart::Fill { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], tris: &[[0, 3, 2], [2, 1, 0]], white: false },
        GlyphPart::Stroke { points: &[(5.13, 12.75), (11.65, 12.75), (0.32, 0.83), (0.45, 17.28)], closed: true, width: 1.0, white: false },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    /// A glyph's id is the cursor it draws: the drawing and `GENERATED` agree by name.
    #[test]
    fn every_generated_glyph_draws_the_cursor_it_is_named_after() {
        for (c, g) in GENERATED {
            let name = format!("{c:?}");
            assert_eq!(name.split('(').next(), Some(g.id), "{c:?} is drawn by {}", g.id);
        }
    }

    #[test]
    fn no_cursor_has_two_generated_glyphs() {
        for (i, (c, _)) in GENERATED.iter().enumerate() {
            assert!(!GENERATED[..i].iter().any(|(k, _)| k == c), "{c:?} appears twice in GENERATED");
        }
    }

    #[test]
    fn generated_glyph_data_is_well_formed() {
        for (c, g) in GENERATED {
            assert!(!g.parts.is_empty(), "{c:?}");
            for part in g.parts {
                let (GlyphPart::Fill { points, .. } | GlyphPart::Stroke { points, .. }) = part;
                assert!(points.iter().all(|(x, y)| x.is_finite() && y.is_finite() && x.abs() < 64.0 && y.abs() < 64.0), "{c:?}");
                if let GlyphPart::Fill { tris, .. } = part {
                    assert!(tris.iter().flatten().all(|&i| usize::from(i) < points.len()), "{c:?}: triangle index out of range");
                }
            }
        }
    }
}
