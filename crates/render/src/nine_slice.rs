//! Place symbol art with Symbol Options › Enable Guides for 9-Slice Scaling.

use std::sync::Arc;

use vectorcraft_color::Paint;
use vectorcraft_doc::{Appearance, Node, NodeId, NodeKind, SymbolNineSlice, decompose_trs, map_rect, scale_cells};
use vectorcraft_geom::{Affine, shapes};

use crate::brush_fx;

/// Place `art` with instance transform `xf`, applying 9-slice scaling when guides are enabled.
///
/// `base` is the symbol's natural-size scale ([`vectorcraft_doc::Document::symbol_natural_xf`]):
/// relative scale is `xf`'s linear part divided by `base`. Falls back to a uniform place when
/// guides are off, the relative scale is 1×1, or the transform cannot be decomposed (shear).
pub fn place_nine_slice(art: &Node, xf: Affine, ns: SymbolNineSlice, base: Affine, inst: Option<&Node>) -> Arc<Node> {
    let Some((tx, ty, angle, sx, sy)) = decompose_trs(xf) else {
        return place_uniform(art, xf, inst);
    };
    let [ba, bb, bc, bd, _, _] = base.as_coeffs();
    let base_sx = (ba * ba + bb * bb).sqrt();
    let base_sy = (bc * bc + bd * bd).sqrt();
    if !(base_sx.is_finite() && base_sy.is_finite()) || base_sx < 1e-12 || base_sy < 1e-12 {
        return place_uniform(art, xf, inst);
    }
    let rel_sx = sx / base_sx;
    let rel_sy = sy / base_sy;
    let Some(cells) = scale_cells(ns, rel_sx, rel_sy) else {
        return place_uniform(art, xf, inst);
    };
    let translate = Affine::translate((tx, ty));
    let rot = if angle.abs() < 1e-12 { Affine::IDENTITY } else { Affine::rotate(angle) };
    let base_scale = Affine::scale_non_uniform(base_sx, base_sy);
    let stained = match inst {
        Some(i) => brush_fx::instance_art(art, i),
        None => art.clone(),
    };
    let mut children = Vec::with_capacity(9);
    for cell in cells {
        if cell.src.width() < 1e-12 || cell.src.height() < 1e-12 || cell.dst.width() < 1e-12 || cell.dst.height() < 1e-12 {
            continue;
        }
        let Some(map) = map_rect(cell.src, cell.dst) else { continue };
        // Local 9-slice map in art space, then natural-size scale, rotation and translation.
        let cell_xf = translate * rot * base_scale * map;
        let mut piece = stained.clone();
        piece.transform(cell_xf, false);
        let clip_local = cell.dst;
        let clip = Node::path(
            NodeId(0),
            shapes::rectangle(clip_local).transformed(translate * rot * base_scale),
            Appearance::basic(Paint::None, Paint::None, 0.0),
        );
        children.push(Arc::new(Node::new(
            NodeId(0),
            NodeKind::Group {
                children: vec![Arc::new(clip), Arc::new(piece)],
                clip: true,
            },
        )));
    }
    if children.is_empty() {
        return place_uniform(art, xf, inst);
    }
    Arc::new(Node::group(NodeId(0), children))
}

fn place_uniform(art: &Node, xf: Affine, inst: Option<&Node>) -> Arc<Node> {
    let mut placed = match inst {
        Some(i) => brush_fx::instance_art(art, i),
        None => art.clone(),
    };
    placed.transform(xf, false);
    Arc::new(placed)
}
