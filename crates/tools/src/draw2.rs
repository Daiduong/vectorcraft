//! Drawing and path-editing tools (pencil, curvature, anchor tools, scissors, knife, line family…).
//!
//! Like every tool these only emit [`crate::Action`]s; the geometry work happens in the engine's
//! `path.*` commands (`crates/engine/src/cmd/draw2.rs`), so gestures replay through the journal.

mod anchor;
mod curvature;
mod family;
mod gesture;

use serde_json::{Value, json};
use vectorcraft_doc::{NodeId, NodeKind};
use vectorcraft_geom::{Anchor, AnchorKind, BezPath, Point, SubPath};

use crate::{Action, Tool, ToolContext};

pub use anchor::AnchorTool;
pub use curvature::{CurvatureTool, EndCurve, curvature_close, curvature_extend, curvature_insert, curvature_move};
pub use family::FamilyTool;
pub use gesture::GestureTool;

/// Colour of tool feedback (Illustrator's selection blue).
pub const FEEDBACK: [u8; 3] = [0x4f, 0x9d, 0xff];

/// Create a drawing tool by id (None = not one of ours).
pub fn create(id: &str) -> Option<Box<dyn Tool>> {
    Some(match id {
        "pencil" | "paintbrush" | "blobBrush" | "eraser" | "knife" | "smooth" | "pathEraser" | "join" => Box::new(GestureTool::new(id)),
        "curvature" => Box::new(CurvatureTool::default()),
        "addAnchor" | "deleteAnchor" | "anchorPoint" | "scissors" => Box::new(AnchorTool::new(id)),
        "arc" | "spiral" | "rectangularGrid" | "polarGrid" => Box::new(FamilyTool::new(id)),
        _ => return None,
    })
}

/// A spline through `points` (Catmull-Rom converted to cubic Béziers). Points flagged `true` are
/// corners (no handles); open ends have no handles either. Used by the Curvature tool and by the
/// engine's freehand fitting.
pub fn catmull_rom(points: &[(Point, bool)], closed: bool) -> SubPath {
    let n = points.len();
    let mut anchors = Vec::with_capacity(n);
    for i in 0..n {
        let (p, corner) = points[i];
        let ends = !closed && (i == 0 || i + 1 == n);
        if corner || ends || n < 3 {
            anchors.push(Anchor::corner(p));
            continue;
        }
        let prev = points[(i + n - 1) % n].0;
        let next = points[(i + 1) % n].0;
        let t = (next - prev) / 6.0;
        if t.hypot() < 1e-9 {
            anchors.push(Anchor::corner(p));
            continue;
        }
        anchors.push(Anchor { p, h_in: p - t, h_out: p + t, kind: AnchorKind::Smooth });
    }
    SubPath::new(anchors, closed && n > 2)
}

/// `[[x, y], …]` for command params.
pub fn points_json(pts: &[Point]) -> Value {
    Value::Array(pts.iter().map(|p| json!([p.x, p.y])).collect())
}

/// A polyline preview path.
pub fn polyline(pts: &[Point]) -> BezPath {
    let mut bp = BezPath::new();
    if let Some(f) = pts.first() {
        bp.move_to(*f);
        for p in &pts[1..] {
            bp.line_to(*p);
        }
    }
    bp
}

/// Editable path nodes, selected ones first, then the rest top-most first.
fn candidate_paths(cx: &ToolContext) -> Vec<NodeId> {
    let selected = cx.doc.nodes_of(&cx.selection.objects);
    let mut v: Vec<NodeId> = selected.into_iter().filter(|n| n.path_data().is_some()).map(|n| n.id).collect();
    // The other paths, each with whether it and everything round it are visible and unlocked
    // (`Document::is_editable`), found in the same walk.
    fn walk(n: &vectorcraft_doc::Node, editable: bool, out: &mut Vec<(NodeId, bool)>) {
        let editable = editable && n.visible && !n.locked;
        if let NodeKind::Path { guide: false, .. } = &n.kind {
            out.push((n.id, editable));
        }
        for c in n.children().into_iter().flatten() {
            walk(c, editable, out);
        }
    }
    let mut rest = vec![];
    for l in &cx.doc.layers {
        walk(l, true, &mut rest);
    }
    // A selected path's editability from the walk (the first place its id is found, as a lookup
    // finds it); anything else (a guide, a compound path…) looked up.
    let mut walked: vectorcraft_doc::IdMap<bool> = vectorcraft_doc::IdMap::default();
    for (id, editable) in &rest {
        walked.entry(*id).or_insert(*editable);
    }
    v.retain(|id| walked.get(id).copied().unwrap_or_else(|| cx.doc.is_editable(*id)));
    let picked: std::collections::HashSet<NodeId> = cx.selection.objects.iter().copied().collect();
    v.extend(rest.into_iter().rev().filter(|(id, editable)| *editable && !picked.contains(id)).map(|(id, _)| id));
    v
}

/// [`editable_paths`] that may pass within `tol` of `p` (their anchors and handles do), in the
/// same order: per document state ([`crate::cache`]), so a hover looks only at those near it.
pub(crate) fn editable_paths_near(cx: &ToolContext, p: Point, tol: f64) -> Vec<NodeId> {
    struct Near {
        ids: Vec<NodeId>,
        ix: crate::boxindex::BoxIndex,
    }
    let near = crate::cache::ToolCache::get(cx, crate::cache::Slot::EditablePaths, || {
        let mut ids = vec![];
        let mut boxes = vec![];
        let editable = crate::cache::editable(cx);
        for n in cx.doc.nodes_of(&cx.selection.objects) {
            let NodeKind::Path { path, guide: false, .. } = &n.kind else { continue };
            let Some(b) = path.control_bounds() else { continue };
            if editable.get(&n.id).copied().unwrap_or(false) {
                boxes.push((b, u32::try_from(ids.len()).unwrap_or(u32::MAX)));
                ids.push(n.id);
            }
        }
        Near { ids, ix: crate::boxindex::BoxIndex::new(boxes) }
    });
    near.ix.near(p, tol).into_iter().filter_map(|i| near.ids.get(i as usize).copied()).collect()
}

/// The selected paths the Pen and the Curvature tool edit (not guides, nor locked or hidden ones).
pub(crate) fn editable_paths<'a>(cx: &'a ToolContext) -> impl Iterator<Item = NodeId> + 'a {
    cx.selection
        .objects
        .iter()
        .copied()
        .filter(|&id| cx.doc.is_editable(id) && cx.doc.node(id).is_some_and(|n| matches!(n.kind, NodeKind::Path { guide: false, .. })))
}

/// The paths whose anchors show with the Pen: the selected ones, and those inside selected groups
/// and compound paths (not guides, nor locked or hidden ones). Pen + Alt converts their anchors.
/// The tools ask [`shown_paths_near`], which looks only at those near the pointer; this is its
/// oracle.
#[cfg(test)]
pub(crate) fn shown_paths(cx: &ToolContext) -> Vec<NodeId> {
    fn inside(n: &vectorcraft_doc::Node, v: &mut Vec<NodeId>) {
        match &n.kind {
            NodeKind::Path { guide: false, .. } => v.push(n.id),
            // A Shape Builder group's first child is its shaper, not art.
            NodeKind::Group { children, .. } | NodeKind::Compound { children, .. } => {
                for c in children.iter().skip(usize::from(n.extras().shaper.is_some())) {
                    inside(c, v);
                }
            }
            _ => {}
        }
    }
    let mut v = vec![];
    for n in cx.selection.objects.iter().filter_map(|id| cx.doc.node(*id)) {
        inside(n, &mut v);
    }
    v.retain(|id| cx.doc.is_editable(*id));
    v
}

/// [`shown_paths`] that may pass within `tol` of `p` (their anchors and handles do), in the same
/// order: per document state ([`crate::cache`]), so a hover with Alt looks only at those near it.
pub(crate) fn shown_paths_near(cx: &ToolContext, p: Point, tol: f64) -> Vec<NodeId> {
    struct Near {
        ids: Vec<NodeId>,
        ix: crate::boxindex::BoxIndex,
    }
    let near = crate::cache::ToolCache::get(cx, crate::cache::Slot::ShownPaths, || {
        fn inside<'a>(n: &'a vectorcraft_doc::Node, v: &mut Vec<&'a vectorcraft_doc::Node>) {
            match &n.kind {
                NodeKind::Path { guide: false, .. } => v.push(n),
                // A Shape Builder group's first child is its shaper, not art.
                NodeKind::Group { children, .. } | NodeKind::Compound { children, .. } => {
                    for c in children.iter().skip(usize::from(n.extras().shaper.is_some())) {
                        inside(c, v);
                    }
                }
                _ => {}
            }
        }
        let mut paths = vec![];
        for n in cx.doc.nodes_of(&cx.selection.objects) {
            inside(n, &mut paths);
        }
        let editable = crate::cache::editable(cx);
        let (mut ids, mut boxes) = (vec![], vec![]);
        for n in paths {
            let Some(b) = n.path_data().and_then(|pd| pd.control_bounds()) else { continue };
            if editable.get(&n.id).copied().unwrap_or(false) {
                boxes.push((b, u32::try_from(ids.len()).unwrap_or(u32::MAX)));
                ids.push(n.id);
            }
        }
        Near { ids, ix: crate::boxindex::BoxIndex::new(boxes) }
    });
    near.ix.near(p, tol).into_iter().filter_map(|i| near.ids.get(i as usize).copied()).collect()
}

/// Nearest anchor within `tol`: (path, subpath, anchor).
pub(crate) fn hit_anchor(cx: &ToolContext, p: Point, tol: f64) -> Option<(NodeId, usize, usize)> {
    anchor_in(cx, candidate_paths(cx), p, tol)
}

/// Nearest anchor within `tol` on the first of `ids` that has one: (path, subpath, anchor).
pub(crate) fn anchor_in(cx: &ToolContext, ids: impl IntoIterator<Item = NodeId>, p: Point, tol: f64) -> Option<(NodeId, usize, usize)> {
    let mut best: Option<((NodeId, usize, usize), f64)> = None;
    for id in ids {
        let Some(pd) = cx.doc.node(id).and_then(|n| n.path_data()) else { continue };
        for (si, ai, a) in pd.anchors() {
            let d = a.p.distance(p);
            if d <= tol && best.is_none_or(|b| d < b.1 - 1e-9) {
                best = Some(((id, si, ai), d));
            }
        }
        if best.is_some() {
            break;
        }
    }
    best.map(|b| b.0)
}

/// Nearest point on a path segment within `tol`: (path, subpath, segment, t).
pub(crate) fn hit_segment(cx: &ToolContext, p: Point, tol: f64) -> Option<(NodeId, usize, usize, f64)> {
    segment_in(cx, candidate_paths(cx), p, tol)
}

/// Nearest point within `tol` on a segment of the first of `ids` that passes there: (path,
/// subpath, segment, t).
pub(crate) fn segment_in(cx: &ToolContext, ids: impl IntoIterator<Item = NodeId>, p: Point, tol: f64) -> Option<(NodeId, usize, usize, f64)> {
    for id in ids {
        let Some(pd) = cx.doc.node(id).and_then(|n| n.path_data()) else { continue };
        if let Some((si, seg, t, _, d)) = pd.nearest(p)
            && d <= tol
        {
            return Some((id, si, seg, t));
        }
    }
    None
}

/// Add an anchor where [`hit_segment`] or [`segment_in`] found a segment.
pub(crate) fn insert_anchor((id, si, seg, t): (NodeId, usize, usize, f64)) -> Action {
    Action::Exec("path.insertAnchor".into(), json!({"id": id.0, "subpath": si, "segment": seg, "t": t}))
}

/// Delete the anchor [`hit_anchor`] or [`anchor_in`] found.
pub(crate) fn remove_anchor((id, si, ai): (NodeId, usize, usize)) -> Action {
    Action::Exec("path.removeAnchor".into(), json!({"id": id.0, "subpath": si, "anchor": ai}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catmull_rom_passes_through_points() {
        let pts = [(Point::new(0.0, 0.0), false), (Point::new(50.0, 50.0), false), (Point::new(100.0, 0.0), false)];
        let sp = catmull_rom(&pts, false);
        assert_eq!(sp.anchors.len(), 3);
        assert_eq!(sp.anchors[1].p, Point::new(50.0, 50.0));
        assert_eq!(sp.anchors[1].kind, AnchorKind::Smooth);
        // The middle tangent is horizontal (parallel to first → last).
        assert!((sp.anchors[1].h_out.y - 50.0).abs() < 1e-9);
        assert!(!sp.anchors[0].has_out());
    }

    #[test]
    fn all_tools_exist() {
        for id in [
            "pencil",
            "paintbrush",
            "blobBrush",
            "eraser",
            "knife",
            "smooth",
            "pathEraser",
            "join",
            "curvature",
            "addAnchor",
            "deleteAnchor",
            "anchorPoint",
            "scissors",
            "arc",
            "spiral",
            "rectangularGrid",
            "polarGrid",
        ] {
            assert_eq!(create(id).unwrap().id(), id);
        }
        assert!(create("selection").is_none());
    }

    /// The paths the Pen, the Curvature tool and the anchor tools pick from, found per state and
    /// near the pointer, are those walking everything finds, in the same order.
    #[test]
    fn path_candidates_match_walking_everything() {
        use crate::testutil::*;
        fn old_candidates(cx: &ToolContext) -> Vec<NodeId> {
            let mut v: Vec<NodeId> =
                cx.selection.objects.iter().copied().filter(|id| cx.doc.node(*id).is_some_and(|n| n.path_data().is_some())).collect();
            let mut rest = vec![];
            cx.doc.walk(|n| {
                if let NodeKind::Path { guide: false, .. } = &n.kind {
                    rest.push(n.id);
                }
            });
            rest.reverse();
            for id in rest {
                if !v.contains(&id) {
                    v.push(id);
                }
            }
            v.retain(|id| cx.doc.is_editable(*id));
            v
        }
        let p = paint();
        for seed in 1..=4 {
            let d = random_doc(seed, 300);
            let s = random_selection(&d, seed, 0.4);
            let c = cx(&d, &s, &p);
            assert_eq!(candidate_paths(&c), old_candidates(&c), "seed {seed}");
            for q in probes(&d, seed, 400) {
                let (tol, point) = (c.pick_tol(), c.point_tol());
                assert_eq!(anchor_in(&c, editable_paths_near(&c, q, point), q, point), anchor_in(&c, editable_paths(&c), q, point));
                assert_eq!(segment_in(&c, editable_paths_near(&c, q, tol), q, tol), segment_in(&c, editable_paths(&c), q, tol));
                assert_eq!(anchor_in(&c, shown_paths_near(&c, q, point), q, point), anchor_in(&c, shown_paths(&c), q, point));
                assert_eq!(segment_in(&c, shown_paths_near(&c, q, tol), q, tol), segment_in(&c, shown_paths(&c), q, tol));
            }
        }
    }
}
