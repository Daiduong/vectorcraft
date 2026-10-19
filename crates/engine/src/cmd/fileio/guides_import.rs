//! Straight path guides from Illustrator editing data become ruler guides on open.

use super::super::pathops::Beside;
use vectorcraft_doc::{Document, Guide, NodeId, NodeKind};
use vectorcraft_geom::PathData;

const TOL: f64 = 1e-6;

/// Axis-aligned single-segment path guides from an import become ruler guides (the rest stay paths).
pub(crate) fn promote_straight_path_guides(doc: &mut Document) {
    let mut promote: Vec<(NodeId, bool, f64, Option<NodeId>)> = vec![];
    doc.walk(|n| {
        if let NodeKind::Path { path, guide: true, clipping: false, .. } = &n.kind
            && let Some((vertical, pos)) = ruler_guide_of(path)
        {
            promote.push((n.id, vertical, pos, doc.layer_containing(n.id)));
        }
    });
    if promote.is_empty() {
        return;
    }
    for (_, vertical, pos, layer) in &promote {
        doc.guides.push(Guide { vertical: *vertical, pos: *pos, artboard: None, layer: *layer });
    }
    let ids: Vec<NodeId> = promote.iter().map(|(id, ..)| *id).collect();
    let mut gone = Beside::replacing(doc, &ids);
    for id in &ids {
        if doc.node(*id).is_some() {
            let _ = gone.put(doc, *id, vec![]);
        }
    }
    let _ = gone.finish(doc);
}

fn ruler_guide_of(path: &PathData) -> Option<(bool, f64)> {
    let [sp] = path.subpaths.as_slice() else { return None };
    if sp.closed || sp.segment_count() != 1 || !sp.segment_is_line(0) {
        return None;
    }
    let a = sp.anchors[0].p;
    let b = sp.anchors[1].p;
    if (a.x - b.x).abs() < TOL {
        if (a.y - b.y).abs() < TOL {
            return None;
        }
        return Some((true, a.x));
    }
    if (a.y - b.y).abs() < TOL {
        return Some((false, a.y));
    }
    None
}
