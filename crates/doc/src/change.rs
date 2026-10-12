//! What changed between two versions of a document's layer tree, found by comparing pointers.
//!
//! Versions share every node an edit didn't touch (`Arc<Node>`, copy-on-write), so a shared node is
//! an unchanged subtree and is never looked into: the comparison costs what changed times its
//! depth, plus a pointer comparison per member of each container that changed.
//!
//! Only the layer tree is compared: document-wide resources (symbols, swatches, artboards…) are
//! not.

use std::sync::Arc;

use vectorcraft_geom::Rect;

use crate::id_index::IdMap;
use crate::{Document, Node, NodeId, NodeKind};

/// A region [`ChangeSet::dirty`] measures: an object before and after (`None`: not there).
type Region = (Option<Arc<Node>>, Option<Arc<Node>>);

/// The changes between two versions of a document ([`ChangeSet::between`]).
#[derive(Clone, Debug, Default)]
pub struct ChangeSet {
    /// The nodes both versions have (by id, at the same parent) that differ: the containers whose
    /// members changed too. Parents come before their members, in the new version's tree order.
    pub changed: Vec<NodeId>,
    /// The nodes of the new version the old one doesn't have there: the roots of added subtrees.
    pub added: Vec<NodeId>,
    /// The nodes of the old version the new one doesn't have there: the roots of removed subtrees.
    pub removed: Vec<NodeId>,
    /// What [`ChangeSet::dirty`] measures: each changed object (old and new), and each added or
    /// removed subtree. Layers and groups whose own attributes are unchanged, and that draw their
    /// members one by one (no effects), are looked into instead of being measured whole.
    regions: Vec<Region>,
}

impl ChangeSet {
    /// The changes from `old` to `new`.
    pub fn between(old: &Document, new: &Document) -> Self {
        let mut out = ChangeSet::default();
        if !std::ptr::eq(old.layers.as_ptr(), new.layers.as_ptr()) || old.layers.len() != new.layers.len() {
            members(&old.layers, &new.layers, &mut out);
        }
        out
    }

    /// Nothing in the layer tree changed.
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty() && self.added.is_empty() && self.removed.is_empty()
    }

    /// The region the changes can repaint: the visual bounds, before and after, of what changed.
    /// `None` when nothing that paints changed.
    pub fn dirty(&self) -> Option<Rect> {
        self.regions.iter().fold(None, |acc, (o, n)| {
            let acc = vectorcraft_geom::union_opt(acc, o.as_ref().and_then(crate::bounds_cache::visual));
            vectorcraft_geom::union_opt(acc, n.as_ref().and_then(crate::bounds_cache::visual))
        })
    }

    /// The changed and added nodes of the new version, each subtree once: what a pass that derives
    /// something from the art needs to look at again.
    pub fn touched(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.changed.iter().chain(&self.added).copied()
    }
}

/// Does `pred` hold for a node of `new` that `old` doesn't have unchanged: a node that changed or
/// one of an added subtree (a node moved to another parent counts as added there)? What a pass
/// that derives something from the art can ask to skip its walk of the whole tree when nothing it
/// derives from changed. Costs what [`ChangeSet::between`] costs, without collecting anything.
pub fn any_new(old: &Document, new: &Document, pred: &dyn Fn(&Node) -> bool) -> bool {
    fn subtree(n: &Node, pred: &dyn Fn(&Node) -> bool) -> bool {
        pred(n) || n.children().is_some_and(|c| c.iter().any(|c| subtree(c, pred)))
    }
    fn changed(o: &Node, n: &Node, pred: &dyn Fn(&Node) -> bool) -> bool {
        pred(n)
            || match (o.children(), n.children()) {
                (Some(oc), Some(nc)) => (!std::ptr::eq(oc.as_ptr(), nc.as_ptr()) || oc.len() != nc.len()) && members_any(oc, nc, pred),
                (_, Some(nc)) => nc.iter().any(|c| subtree(c, pred)),
                _ => false,
            }
    }
    fn members_any(old: &[Arc<Node>], new: &[Arc<Node>], pred: &dyn Fn(&Node) -> bool) -> bool {
        let pre = old.iter().zip(new).take_while(|(o, n)| same(o, n)).count();
        let (old, new) = (&old[pre..], &new[pre..]);
        let suf = old.iter().rev().zip(new.iter().rev()).take_while(|(o, n)| same(o, n)).count();
        let (old, new) = (&old[..old.len() - suf], &new[..new.len() - suf]);
        if old.len() == new.len() && old.iter().zip(new).all(|(o, n)| o.id == n.id) {
            return old.iter().zip(new).any(|(o, n)| !same(o, n) && changed(o, n, pred));
        }
        let mut by_id: IdMap<Vec<usize>> = IdMap::default();
        for (i, o) in old.iter().enumerate().rev() {
            by_id.entry(o.id).or_default().push(i);
        }
        new.iter().any(|n| match by_id.get_mut(&n.id).and_then(Vec::pop).and_then(|i| old.get(i)) {
            Some(o) => !same(o, n) && changed(o, n, pred),
            None => subtree(n, pred),
        })
    }
    members_any(&old.layers, &new.layers, pred)
}

fn same(a: &Arc<Node>, b: &Arc<Node>) -> bool {
    Arc::ptr_eq(a, b)
}

/// Compare the members `old` and `new` of one container (or the layer lists).
fn members(old: &[Arc<Node>], new: &[Arc<Node>], out: &mut ChangeSet) {
    let pre = old.iter().zip(new).take_while(|(o, n)| same(o, n)).count();
    let (old, new) = (&old[pre..], &new[pre..]);
    let suf = old.iter().rev().zip(new.iter().rev()).take_while(|(o, n)| same(o, n)).count();
    let (old, new) = (&old[..old.len() - suf], &new[..new.len() - suf]);
    if old.len() == new.len() && old.iter().zip(new).all(|(o, n)| o.id == n.id) {
        // Changed in place (the common case): pair by position.
        for (o, n) in old.iter().zip(new) {
            if !same(o, n) {
                node(o, n, out);
            }
        }
        return;
    }
    // Added, removed or reordered: pair by id (the first unpaired old node with the id).
    let mut by_id: IdMap<Vec<usize>> = IdMap::default();
    for (i, o) in old.iter().enumerate().rev() {
        by_id.entry(o.id).or_default().push(i);
    }
    let mut paired = vec![false; old.len()];
    for n in new {
        match by_id.get_mut(&n.id).and_then(Vec::pop) {
            Some(i) => {
                if let (Some(o), Some(p)) = (old.get(i), paired.get_mut(i)) {
                    *p = true;
                    if !same(o, n) {
                        node(o, n, out);
                    }
                }
            }
            None => {
                out.added.push(n.id);
                out.regions.push((None, Some(n.clone())));
            }
        }
    }
    for (o, p) in old.iter().zip(&paired) {
        if !p {
            out.removed.push(o.id);
            out.regions.push((Some(o.clone()), None));
        }
    }
}

/// `o` became `n` (same id, different node).
fn node(o: &Arc<Node>, n: &Arc<Node>, out: &mut ChangeSet) {
    out.changed.push(n.id);
    let (Some(oc), Some(nc)) = (o.children(), n.children()) else {
        out.regions.push((Some(o.clone()), Some(n.clone())));
        return;
    };
    // Live art (blends, envelopes, repeats, compound paths), a container whose own attributes
    // changed and one with effects repaint as a whole.
    let whole = !(same_own(o, n) && draws_members_alone(n));
    if whole {
        out.regions.push((Some(o.clone()), Some(n.clone())));
    }
    if !std::ptr::eq(oc.as_ptr(), nc.as_ptr()) || oc.len() != nc.len() {
        let mark = out.regions.len();
        members(oc, nc, out);
        if whole {
            out.regions.truncate(mark);
        }
    }
}

/// Does a change of one member of `n` repaint only where that member is? A layer or a group
/// without effects of its own (an effect, a blur say, spreads its members' paint).
fn draws_members_alone(n: &Node) -> bool {
    matches!(n.kind, NodeKind::Layer { .. } | NodeKind::Group { .. })
        && n.appearance.effects.is_empty()
        && n.appearance.items.iter().all(|i| i.effects().is_empty())
}

/// Are the attributes of containers `o` and `n` other than their members the same?
fn same_own(o: &Node, n: &Node) -> bool {
    let kind = match (&o.kind, &n.kind) {
        (
            NodeKind::Layer { color: c1, template: t1, printable: p1, clip: k1, preview: v1, dim_images: d1, .. },
            NodeKind::Layer { color: c2, template: t2, printable: p2, clip: k2, preview: v2, dim_images: d2, .. },
        ) => c1 == c2 && t1 == t2 && p1 == p2 && k1 == k2 && v1 == v2 && d1 == d2,
        (NodeKind::Group { clip: a, .. }, NodeKind::Group { clip: b, .. }) => a == b,
        _ => false,
    };
    kind && o.id == n.id
        && o.name == n.name
        && o.visible == n.visible
        && o.locked == n.locked
        && o.opacity == n.opacity
        && o.blend == n.blend
        && o.isolate == n.isolate
        && o.knockout == n.knockout
        && o.knockout_shape == n.knockout_shape
        && (crate::Shared::ptr_eq(&o.appearance, &n.appearance) || *o.appearance == *n.appearance)
        && o.bbox_angle == n.bbox_angle
        && o.extras == n.extras
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Appearance;
    use vectorcraft_geom::{Affine, shapes};

    fn doc(n: usize) -> (Document, Vec<NodeId>) {
        let mut d = Document::new(1000.0, 1000.0);
        let l = d.layers[0].id;
        let mut ids = vec![];
        for i in 0..n {
            let x = i as f64 * 20.0;
            let id = d.alloc_id();
            ids.push(id);
            d.insert(Some(l), usize::MAX, Node::path(id, shapes::rectangle(Rect::new(x, 0.0, x + 10.0, 10.0)), Appearance::default_art())).unwrap();
        }
        (d, ids)
    }

    #[test]
    fn nothing_changed() {
        let (d, _) = doc(5);
        let c = ChangeSet::between(&d, &d.clone());
        assert!(c.is_empty());
        assert_eq!(c.dirty(), None);
    }

    #[test]
    fn a_moved_object_dirties_where_it_was_and_is() {
        let (d, ids) = doc(5);
        let mut e = d.clone();
        e.node_mut(ids[2]).unwrap().transform(Affine::translate((0.0, 100.0)), false);
        let c = ChangeSet::between(&d, &e);
        assert_eq!(c.changed, vec![d.layers[0].id, ids[2]]);
        assert!(c.added.is_empty() && c.removed.is_empty());
        let old = d.node(ids[2]).unwrap().visual_bounds().unwrap();
        let new = e.node(ids[2]).unwrap().visual_bounds().unwrap();
        assert_eq!(c.dirty(), Some(old.union(new)));
    }

    #[test]
    fn added_removed_and_reordered() {
        let (d, ids) = doc(6);
        let l = d.layers[0].id;
        let mut e = d.clone();
        e.remove(ids[1]).unwrap();
        let id = e.alloc_id();
        e.insert(Some(l), 2, Node::path(id, shapes::rectangle(Rect::new(500.0, 500.0, 510.0, 510.0)), Appearance::default_art())).unwrap();
        e.move_node(ids[4], Some(l), 0).unwrap();
        let c = ChangeSet::between(&d, &e);
        assert_eq!(c.added, vec![id]);
        assert_eq!(c.removed, vec![ids[1]]);
        assert_eq!(c.changed, vec![l], "the layer changed; the moved object didn't");
        let gone = d.node(ids[1]).unwrap().visual_bounds().unwrap();
        let new = e.node(id).unwrap().visual_bounds().unwrap();
        assert_eq!(c.dirty(), Some(gone.union(new)));
    }

    #[test]
    fn any_new_looks_at_changed_and_added_nodes_only() {
        let (d, ids) = doc(4);
        let l = d.layers[0].id;
        let mut e = d.clone();
        e.node_mut(ids[1]).unwrap().opacity = 0.5;
        let is = |id: NodeId| move |n: &Node| n.id == id;
        assert!(any_new(&d, &e, &is(ids[1])));
        assert!(!any_new(&d, &e, &is(ids[2])), "unchanged");
        assert!(any_new(&d, &e, &is(l)), "its layer changed");
        let id = e.alloc_id();
        let inner = Node::path(e.alloc_id(), shapes::rectangle(Rect::new(0.0, 0.0, 1.0, 1.0)), Appearance::default_art());
        let inner_id = inner.id;
        e.insert(Some(l), 0, Node::group(id, vec![Arc::new(inner)])).unwrap();
        assert!(any_new(&d, &e, &is(inner_id)), "inside an added group");
        assert!(!any_new(&d, &d.clone(), &|_| true));
    }

    /// A group's own change (its opacity) dirties all of it; a member's, only the member.
    #[test]
    fn a_containers_own_change_dirties_all_of_it() {
        let mut d = Document::new(1000.0, 1000.0);
        let l = d.layers[0].id;
        let (a, b, g) = (d.alloc_id(), d.alloc_id(), d.alloc_id());
        let pa = Node::path(a, shapes::rectangle(Rect::new(0.0, 0.0, 10.0, 10.0)), Appearance::default_art());
        let pb = Node::path(b, shapes::rectangle(Rect::new(100.0, 0.0, 110.0, 10.0)), Appearance::default_art());
        d.insert(Some(l), 0, Node::group(g, vec![Arc::new(pa), Arc::new(pb)])).unwrap();
        let mut e = d.clone();
        e.node_mut(a).unwrap().opacity = 0.5;
        let c = ChangeSet::between(&d, &e);
        assert_eq!(c.changed, vec![l, g, a]);
        assert_eq!(c.dirty(), d.node(a).unwrap().visual_bounds());
        let mut f = d.clone();
        f.node_mut(g).unwrap().opacity = 0.5;
        assert_eq!(ChangeSet::between(&d, &f).dirty(), d.node(g).unwrap().visual_bounds());
    }
}
