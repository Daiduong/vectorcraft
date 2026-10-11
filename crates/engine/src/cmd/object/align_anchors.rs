//! Align direct-selected points without translating their entire paths (#1180).

use super::*;

pub(super) struct Anchor {
    id: NodeId,
    subpath: usize,
    index: usize,
    pub(super) point: Point,
}

/// A whole selected path contributes all its points when another path has selected anchors.
/// With only whole objects selected, leave alignment to the object implementation.
pub(super) fn selected(s: &Session) -> Result<Option<Vec<Anchor>>> {
    let st = s.doc()?;
    if st.selection.anchors.values().all(BTreeSet::is_empty) {
        return Ok(None);
    }
    let mut anchors = vec![];
    for id in &st.selection.objects {
        let Some(path) = st.doc.node(*id).and_then(|n| n.path_data()) else { continue };
        for (subpath, index, a) in path.anchors() {
            if st.selection.anchors.get(id).is_none_or(|set| set.contains(&(subpath, index))) {
                anchors.push(Anchor { id: *id, subpath, index, point: a.p });
            }
        }
    }
    Ok(Some(anchors))
}

pub(super) fn apply(s: &mut Session, anchors: &[Anchor], reference: Rect, horizontal: Option<&str>, vertical: Option<&str>) -> Result<Value> {
    let x = match horizontal {
        Some("left") => Some(reference.x0),
        Some("center") => Some(reference.center().x),
        Some("right") => Some(reference.x1),
        _ => None,
    };
    let y = match vertical {
        Some("top") => Some(reference.y0),
        Some("center") => Some(reference.center().y),
        Some("bottom") => Some(reference.y1),
        _ => None,
    };
    let moves: Vec<_> = anchors
        .iter()
        .map(|a| (a, Point::new(x.unwrap_or(a.point.x), y.unwrap_or(a.point.y)) - a.point))
        .filter(|(_, dv)| dv.x.abs() > 1e-9 || dv.y.abs() > 1e-9)
        .collect();
    if moves.is_empty() {
        return ok();
    }
    s.edit("Align", |doc, _| {
        for (a, delta) in &moves {
            let node = doc.node_mut(a.id).ok_or(EngineError::NoNode(a.id))?;
            if let NodeKind::Path { path, live, .. } = &mut node.kind
                && let Some(anchor) = path.anchor_mut(a.subpath, a.index)
            {
                *live = None;
                anchor.translate(*delta);
            }
        }
        Ok(())
    })?;
    ok()
}
