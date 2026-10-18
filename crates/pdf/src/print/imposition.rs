//! N-up imposition: several artboards on one physical sheet without copying document objects.

use kurbo::{Affine, Rect};

use super::layout::Layout;
use super::{ImpositionMode, PrintSettings};
use crate::PdfError;

/// Lay out `pages` on shared physical sheets when N-up is on; otherwise one layout per sheet.
pub(crate) fn apply(pages: &mut [Layout], set: &PrintSettings) -> Result<(), PdfError> {
    if set.imposition.mode == ImpositionMode::None {
        for (i, p) in pages.iter_mut().enumerate() {
            p.physical = i;
        }
        return Ok(());
    }
    if set.imposition.mode != ImpositionMode::NUp {
        return Ok(());
    }
    let cols = set.imposition.columns.max(1) as usize;
    let rows = set.imposition.rows.max(1) as usize;
    let per_sheet = cols * rows;
    let gap = set.imposition.gap;
    let m = set.margin;
    for (physical, chunk_start) in (0usize..).zip((0..pages.len()).step_by(per_sheet)) {
        let chunk_end = (chunk_start + per_sheet).min(pages.len());
        let page_w = pages[chunk_start].size.0;
        let page_h = pages[chunk_start].size.1;
        let imageable = Rect::new(m, m, page_w - m, page_h - m);
        if imageable.width() < 1.0 || imageable.height() < 1.0 {
            return Err(PdfError::BadSetting("imposition: the margin leaves no imageable area".into()));
        }
        let cell_w = (imageable.width() - gap * (cols.saturating_sub(1) as f64)) / cols as f64;
        let cell_h = (imageable.height() - gap * (rows.saturating_sub(1) as f64)) / rows as f64;
        if cell_w < 1.0 || cell_h < 1.0 {
            return Err(PdfError::BadSetting("imposition: the grid leaves no room for art on the sheet".into()));
        }
        for (j, layout) in pages[chunk_start..chunk_end].iter_mut().enumerate() {
            layout.physical = physical;
            let col = j % cols;
            let row = j / cols;
            let x0 = imageable.x0 + col as f64 * (cell_w + gap);
            let y0 = imageable.y0 + row as f64 * (cell_h + gap);
            let cell = Rect::new(x0, y0, x0 + cell_w, y0 + cell_h);
            fit_in_cell(layout, cell);
        }
    }
    Ok(())
}

/// Scale and move one laid-out page so its bleed box fits in `cell` (page space).
fn fit_in_cell(layout: &mut Layout, cell: Rect) {
    let content = layout.page_bleed;
    if content.width() <= 0.0 || content.height() <= 0.0 {
        return;
    }
    let s = (cell.width() / content.width()).min(cell.height() / content.height());
    if !(s.is_finite() && s > 0.0) {
        return;
    }
    let scaled = Affine::scale(s).transform_rect_bbox(content);
    let dx = cell.x0 - scaled.x0;
    let dy = cell.y0 - scaled.y0;
    let t = Affine::translate((dx, dy)) * Affine::scale(s);
    layout.view = t * layout.view;
    layout.page_trim = t.transform_rect_bbox(layout.page_trim);
    layout.page_bleed = t.transform_rect_bbox(layout.page_bleed);
    layout.scale = (layout.scale.0 * s, layout.scale.1 * s);
}
