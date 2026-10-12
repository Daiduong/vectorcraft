//! A spatial index of the objects of large containers (layers and groups with many members), so a
//! click, a hover or a marquee looks only at the objects near it instead of at every one.
//!
//! Each indexed container keeps an R-tree of its members' [`Node::pick_bounds`] (a box round
//! everything a click or a marquee can find in the member), by position among the members. The
//! index only *narrows* the search: the hit test and the marquee still run their exact tests on the
//! members it returns, in paint order, so their results are those of looking at every member.
//!
//! # Where it lives, and why it is never stale
//!
//! A document keeps one [`SpatialCache`], shared by its clones: the versions an edit makes, the
//! states the undo history keeps and the snapshot a drag previews from. The cache holds one index
//! per container id, for the version of the container it last saw, with a [`Weak`] to that version
//! and one to each member (so their addresses can't be reused, see [`crate::bounds_cache`]). Asked
//! about another version (an edit, an undo, a preview frame), it compares the members by pointer:
//! the shared prefix and suffix are kept, the members that differ are taken out and put in again,
//! and when many differ (a move of everything) the tree is built again in one go. Undo and redo
//! need nothing kept per history state: the index follows whichever version is asked about.
//!
//! Nothing here is saved, and only containers of at least [`MIN_MEMBERS`] members are indexed.

use std::sync::{Arc, Mutex, PoisonError, Weak};

use rstar::{AABB, RTree, RTreeObject};
use vectorcraft_geom::Rect;

use crate::Node;
use crate::id_index::IdMap;

/// Containers with fewer members are searched member by member.
pub const MIN_MEMBERS: usize = 64;
/// The most containers a document keeps an index for (the least recently used goes).
const MAX_CONTAINERS: usize = 256;
/// Boxes beyond this (in points) are not put in the tree (their members are always looked at):
/// it keeps the tree's arithmetic finite in `f32`.
const LIMIT: f64 = 1e15;

/// One member in the tree: its box (rounded outward to `f32`) and its position.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Entry {
    b: [f32; 4],
    i: u32,
}

impl RTreeObject for Entry {
    type Envelope = AABB<[f32; 2]>;
    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners([self.b[0], self.b[1]], [self.b[2], self.b[3]])
    }
}

/// `v` rounded down (`up`: up) to an `f32`.
fn round(v: f64, up: bool) -> f32 {
    let f = v as f32;
    match (up, f64::from(f).partial_cmp(&v)) {
        (false, Some(std::cmp::Ordering::Greater)) => f.next_down(),
        (true, Some(std::cmp::Ordering::Less)) => f.next_up(),
        _ => f,
    }
}

/// The box of `r` for the tree: rounded outward, or `None` (look at the member always) when it is
/// not finite, not normalized or too large.
fn tree_box(r: Option<Rect>) -> Option<[f32; 4]> {
    let r = r?;
    let ok = [r.x0, r.y0, r.x1, r.y1].iter().all(|v| v.is_finite() && v.abs() <= LIMIT) && r.x0 <= r.x1 && r.y0 <= r.y1;
    ok.then(|| [round(r.x0, false), round(r.y0, false), round(r.x1, true), round(r.y1, true)])
}

/// Is `c` looked at whatever the query (its box can't stand for it)? Sublayers (a marquee looks
/// through them, a click culls them by their own bounds), objects without reach bounds (a click
/// never culls them), and envelopes whose contents are edited or Shaper groups (a click looks
/// inside them whatever their bounds, depending on the isolation scope).
fn always(c: &Arc<Node>) -> bool {
    c.is_layer()
        || matches!(c.kind, crate::NodeKind::Envelope { editing: true, .. })
        || c.extras().shaper.is_some()
        || crate::bounds_cache::reach(c).is_none()
}

/// The index of one container's members.
struct Index {
    /// The version of the container indexed.
    version: Weak<Node>,
    /// Each member indexed, by position.
    kids: Vec<Weak<Node>>,
    /// Each member's box in the tree; `None`: in `always`.
    boxes: Vec<Option<[f32; 4]>>,
    tree: RTree<Entry>,
    /// The members looked at whatever the query, in order.
    always: Vec<u32>,
    /// When it was last used (for evicting the least recently used).
    used: u64,
}

impl Index {
    fn build(container: &Arc<Node>, kids: &[Arc<Node>]) -> Self {
        let mut boxes = Vec::with_capacity(kids.len());
        let mut entries = Vec::with_capacity(kids.len());
        let mut alw = vec![];
        for (i, c) in kids.iter().enumerate() {
            let b = Self::box_of(c);
            match b {
                Some(b) => entries.push(Entry { b, i: i as u32 }),
                None => alw.push(i as u32),
            }
            boxes.push(b);
        }
        Index {
            version: Arc::downgrade(container),
            kids: kids.iter().map(Arc::downgrade).collect(),
            boxes,
            tree: RTree::bulk_load(entries),
            always: alw,
            used: 0,
        }
    }

    fn box_of(c: &Arc<Node>) -> Option<[f32; 4]> {
        if always(c) { None } else { tree_box(crate::bounds_cache::pick(c)) }
    }

    /// Follow the container to version `container` (members `kids`).
    fn update(&mut self, container: &Arc<Node>, kids: &[Arc<Node>]) {
        // The weak reference keeps the address: the same address is the same version.
        if std::ptr::eq(self.version.as_ptr(), Arc::as_ptr(container)) {
            return;
        }
        let same = |old: &Weak<Node>, new: &Arc<Node>| std::ptr::eq(old.as_ptr(), Arc::as_ptr(new));
        let (n_old, n_new) = (self.kids.len(), kids.len());
        let pre = self.kids.iter().zip(kids).take_while(|(o, n)| same(o, n)).count();
        let suf = self.kids[pre..].iter().rev().zip(kids[pre..].iter().rev()).take_while(|(o, n)| same(o, n)).count();
        let budget = n_new.max(MIN_MEMBERS) / 4;
        if n_old == n_new {
            // Members changed in place: replace those that differ.
            let changed: Vec<usize> = (pre..n_new - suf).filter(|&i| !self.kids.get(i).zip(kids.get(i)).is_some_and(|(o, n)| same(o, n))).collect();
            if changed.len() > budget {
                *self = Self::build(container, kids);
                return;
            }
            for i in changed {
                self.take_out(i);
                if let Some(c) = kids.get(i) {
                    self.put_in(i, c);
                }
            }
        } else if suf == 0 && (n_old - pre) + (n_new - pre) <= budget {
            // Members added or removed at the top (the common case: new art goes on top).
            for i in (pre..n_old).rev() {
                self.take_out(i);
            }
            self.kids.truncate(pre);
            self.boxes.truncate(pre);
            for (i, c) in kids.iter().enumerate().skip(pre) {
                self.kids.push(Weak::new());
                self.boxes.push(None);
                self.put_in(i, c);
            }
        } else {
            // Members moved, added or removed below others: positions shift, build afresh.
            *self = Self::build(container, kids);
            return;
        }
        self.version = Arc::downgrade(container);
    }

    fn take_out(&mut self, i: usize) {
        match self.boxes.get(i).copied().flatten() {
            Some(b) => {
                self.tree.remove(&Entry { b, i: i as u32 });
            }
            None => {
                if let Ok(k) = self.always.binary_search(&(i as u32)) {
                    self.always.remove(k);
                }
            }
        }
    }

    fn put_in(&mut self, i: usize, c: &Arc<Node>) {
        let b = Self::box_of(c);
        match b {
            Some(b) => self.tree.insert(Entry { b, i: i as u32 }),
            None => {
                if let Err(k) = self.always.binary_search(&(i as u32)) {
                    self.always.insert(k, i as u32);
                }
            }
        }
        if let (Some(k), Some(s)) = (self.kids.get_mut(i), self.boxes.get_mut(i)) {
            *k = Arc::downgrade(c);
            *s = b;
        }
    }

    /// The members whose box meets `q` (and those looked at always), in order.
    fn query(&self, q: [f32; 4]) -> Vec<usize> {
        let env = AABB::from_corners([q[0], q[1]], [q[2], q[3]]);
        let mut out: Vec<usize> = self.tree.locate_in_envelope_intersecting(&env).map(|e| e.i as usize).collect();
        out.extend(self.always.iter().map(|&i| i as usize));
        out.sort_unstable();
        out.dedup();
        out
    }
}

#[derive(Default)]
struct State {
    containers: IdMap<Index>,
    clock: u64,
}

/// A document's spatial index (see the module docs): shared by the document's clones, never saved,
/// equal to any other (it is a cache, not content).
#[derive(Clone, Default)]
pub struct SpatialCache(Arc<Mutex<State>>);

impl PartialEq for SpatialCache {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl std::fmt::Debug for SpatialCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SpatialCache")
    }
}

impl SpatialCache {
    /// The members of `container` whose [`Node::pick_bounds`] may meet `q`, in paint order (bottom
    /// first): every member that can be clicked within `q` or that a marquee of `q` can touch, and
    /// maybe a few more. `None` when the container is small (search all its members) or `q` isn't
    /// finite.
    pub fn candidates(&self, container: &Arc<Node>, q: Rect) -> Option<Vec<usize>> {
        let kids = container.children().filter(|k| k.len() >= MIN_MEMBERS)?;
        if ![q.x0, q.y0, q.x1, q.y1].iter().all(|v| v.is_finite()) {
            return None;
        }
        // A little slack: the exact tests decide, the box only rules out what is surely away.
        let slack = 1e-6 * (1.0 + q.x0.abs().max(q.y0.abs()).max(q.x1.abs()).max(q.y1.abs()));
        let clamp = |v: f64| v.clamp(-LIMIT, LIMIT);
        let q = [
            round(clamp(q.x0.min(q.x1) - slack), false),
            round(clamp(q.y0.min(q.y1) - slack), false),
            round(clamp(q.x0.max(q.x1) + slack), true),
            round(clamp(q.y0.max(q.y1) + slack), true),
        ];
        let mut st = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        st.clock += 1;
        let clock = st.clock;
        // Now and then, let go of indexes of containers no version of which is left (deleted
        // layers and groups): they hold weak references to all their members.
        if clock.is_multiple_of(64) {
            st.containers.retain(|_, ix| ix.version.strong_count() > 0 || clock - ix.used < 64);
        }
        if !st.containers.contains_key(&container.id) && st.containers.len() >= MAX_CONTAINERS {
            let oldest = st.containers.iter().min_by_key(|(_, ix)| ix.used).map(|(id, _)| *id);
            if let Some(id) = oldest {
                st.containers.remove(&id);
            }
        }
        let ix = st.containers.entry(container.id).or_insert_with(|| Index::build(container, kids));
        ix.update(container, kids);
        ix.used = clock;
        Some(ix.query(q))
    }

    /// Members of `container` within `tol` of `p` ([`SpatialCache::candidates`]).
    pub fn candidates_near(&self, container: &Arc<Node>, p: vectorcraft_geom::Point, tol: f64) -> Option<Vec<usize>> {
        let t = if tol.is_finite() { tol.max(0.0) } else { return None };
        self.candidates(container, Rect::new(p.x - t, p.y - t, p.x + t, p.y + t))
    }

    /// Forget every index (they are built again when next asked for).
    pub fn clear(&self) {
        let mut st = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        st.containers.clear();
    }

    /// How many containers are indexed, and how many members those indexes hold: for tests and
    /// measurements.
    pub fn stats(&self) -> (usize, usize) {
        let st = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        (st.containers.len(), st.containers.values().map(|ix| ix.kids.len()).sum())
    }

    /// Check every index against its container version, as a fresh build would make it (tests).
    #[cfg(test)]
    pub(crate) fn check(&self, container: &Arc<Node>) {
        let st = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(ix) = st.containers.get(&container.id) else { return };
        if !std::ptr::eq(ix.version.as_ptr(), Arc::as_ptr(container)) {
            return;
        }
        let kids = container.children().map(Vec::as_slice).unwrap_or_default();
        let fresh = Index::build(container, kids);
        assert_eq!(ix.boxes, fresh.boxes, "boxes of {}", container.id);
        assert_eq!(ix.always, fresh.always, "always of {}", container.id);
        assert_eq!(ix.tree.size(), fresh.tree.size());
        assert!(ix.kids.iter().zip(kids).all(|(o, n)| std::ptr::eq(o.as_ptr(), Arc::as_ptr(n))));
    }
}

/// The id of a container indexed by `cache` (tests).
#[cfg(test)]
pub(crate) fn indexed(cache: &SpatialCache, id: crate::NodeId) -> bool {
    cache.0.lock().unwrap_or_else(PoisonError::into_inner).containers.contains_key(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Appearance, Document, NodeId};
    use vectorcraft_geom::{Point, shapes};

    #[test]
    fn rounding_is_outward() {
        for v in [0.1, -0.1, 1e10 + 0.3, -3.3333333333, 7.0, 0.0] {
            assert!(f64::from(round(v, false)) <= v && f64::from(round(v, true)) >= v, "{v}");
        }
        assert_eq!(tree_box(Some(Rect::new(0.0, 0.0, f64::NAN, 1.0))), None);
        assert_eq!(tree_box(Some(Rect::new(2.0, 0.0, 1.0, 1.0))), None, "not normalized");
        assert_eq!(tree_box(Some(Rect::new(0.0, 0.0, 1e16, 1.0))), None, "too large");
    }

    #[test]
    fn follows_edits_and_undo_by_pointer() {
        let mut d = Document::new(1000.0, 1000.0);
        let l = d.layers[0].id;
        let mut ids = vec![];
        for i in 0..200 {
            let x = (i % 20) as f64 * 40.0;
            let y = (i / 20) as f64 * 40.0;
            let id = d.alloc_id();
            ids.push(id);
            d.insert(Some(l), usize::MAX, Node::path(id, shapes::rectangle(Rect::new(x, y, x + 10.0, y + 10.0)), Appearance::default_art())).unwrap();
        }
        let near = |d: &Document, p: Point| d.spatial.candidates_near(&d.layers[0], p, 1.0).unwrap();
        assert_eq!(near(&d, Point::new(5.0, 5.0)), vec![0]);
        assert!(near(&d, Point::new(25.0, 25.0)).is_empty());
        let before = d.clone();
        // Move one object onto the empty spot: one member replaced.
        d.node_mut(ids[0]).unwrap().transform(vectorcraft_geom::Affine::translate((20.0, 20.0)), false);
        assert_eq!(near(&d, Point::new(25.0, 25.0)), vec![0]);
        d.spatial.check(&d.layers[0]);
        // Undo: the old version, asked about, is followed back.
        assert_eq!(near(&before, Point::new(25.0, 25.0)), Vec::<usize>::new());
        assert_eq!(near(&before, Point::new(5.0, 5.0)), vec![0]);
        before.spatial.check(&before.layers[0]);
        // New art on top, then a deletion below: positions shift.
        let id = d.alloc_id();
        d.insert(Some(l), usize::MAX, Node::path(id, shapes::rectangle(Rect::new(0.0, 0.0, 10.0, 10.0)), Appearance::default_art())).unwrap();
        assert_eq!(near(&d, Point::new(5.0, 5.0)), vec![200]);
        d.spatial.check(&d.layers[0]);
        d.remove(ids[5]).unwrap();
        assert_eq!(near(&d, Point::new(5.0, 5.0)), vec![199]);
        d.spatial.check(&d.layers[0]);
        assert!(indexed(&d.spatial, l));
        // A small container isn't indexed.
        let g = Arc::new(Node::group(NodeId(9999), vec![]));
        assert!(d.spatial.candidates_near(&g, Point::ZERO, 1.0).is_none());
    }
}
