//! An R-tree of numbered boxes and of numbered points, for the per-state lists the tools keep
//! ([`crate::cache`]) and for snap targets: which of many boxes (points) are near the pointer,
//! without looking at each. They only narrow the search: callers run their exact tests on what
//! comes back, in the order of the numbers, so results are those of looking at every entry.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use rstar::{AABB, RTree, RTreeObject};
use vectorcraft_geom::{Point, Rect};

#[derive(Clone, Copy, Debug)]
struct Entry {
    b: [f64; 4],
    i: u32,
}

impl RTreeObject for Entry {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners([self.b[0], self.b[1]], [self.b[2], self.b[3]])
    }
}

/// Boxes beyond this are always looked at, keeping the tree's arithmetic sane.
const LIMIT: f64 = 1e15;

/// Numbered boxes (see the [module docs](self)). The first few lookups scan the boxes (no tree to
/// build when the state changes again before many come); later ones use an R-tree, built once.
#[derive(Debug, Default)]
pub struct BoxIndex {
    boxes: Vec<Entry>,
    tree: OnceLock<RTree<Entry>>,
    /// Entries whose box can't go in the tree (not finite, inverted, huge): always candidates.
    always: Vec<u32>,
    /// Lookups so far, up to [`SCANS`].
    asked: AtomicUsize,
}

/// How many lookups scan the boxes before the tree is built.
const SCANS: usize = 8;

impl BoxIndex {
    /// The index of `boxes`, each with its number.
    pub fn new(boxes: impl IntoIterator<Item = (Rect, u32)>) -> Self {
        let (mut entries, mut always) = (vec![], vec![]);
        for (r, i) in boxes {
            let b = [r.x0, r.y0, r.x1, r.y1];
            if b.iter().all(|v| v.is_finite() && v.abs() <= LIMIT) && r.x0 <= r.x1 && r.y0 <= r.y1 {
                entries.push(Entry { b, i });
            } else {
                always.push(i);
            }
        }
        Self { boxes: entries, tree: OnceLock::new(), always, asked: AtomicUsize::new(0) }
    }

    /// The numbers of the boxes that may meet `r` grown by a little slack (every box that does,
    /// maybe a few more), ascending. Everything when `r` isn't finite.
    pub fn meeting(&self, r: Rect) -> Vec<u32> {
        let q = [r.x0.min(r.x1), r.y0.min(r.y1), r.x0.max(r.x1), r.y0.max(r.y1)];
        let mut out: Vec<u32> = if q.iter().all(|v| v.is_finite()) {
            let slack = 1e-6 * (1.0 + q.iter().fold(0.0_f64, |m, v| m.max(v.abs())));
            let q = [q[0] - slack, q[1] - slack, q[2] + slack, q[3] + slack];
            let found: Vec<u32> = if self.asked.fetch_add(1, Ordering::Relaxed) < SCANS && self.tree.get().is_none() {
                self.boxes.iter().filter(|e| e.b[0] <= q[2] && q[0] <= e.b[2] && e.b[1] <= q[3] && q[1] <= e.b[3]).map(|e| e.i).collect()
            } else {
                let tree = self.tree.get_or_init(|| RTree::bulk_load(self.boxes.clone()));
                tree.locate_in_envelope_intersecting(&AABB::from_corners([q[0], q[1]], [q[2], q[3]])).map(|e| e.i).collect()
            };
            found.into_iter().chain(self.always.iter().copied()).collect()
        } else {
            self.boxes.iter().map(|e| e.i).chain(self.always.iter().copied()).collect()
        };
        out.sort_unstable();
        out
    }

    /// [`Self::meeting`] the square of half-side `tol` round `p`.
    pub fn near(&self, p: Point, tol: f64) -> Vec<u32> {
        let t = tol.max(0.0);
        self.meeting(Rect::new(p.x - t, p.y - t, p.x + t, p.y + t))
    }

    pub fn len(&self) -> usize {
        self.boxes.len() + self.always.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_boxes_near_a_point_and_keeps_odd_ones() {
        let ix = BoxIndex::new([
            (Rect::new(0.0, 0.0, 10.0, 10.0), 0),
            (Rect::new(100.0, 100.0, 110.0, 110.0), 1),
            (Rect::new(f64::NAN, 0.0, 1.0, 1.0), 2),
            (Rect::new(5.0, 5.0, 6.0, 6.0), 3),
        ]);
        assert_eq!(ix.near(Point::new(12.0, 5.0), 2.0), vec![0, 2]);
        assert_eq!(ix.near(Point::new(5.5, 5.5), 0.0), vec![0, 2, 3]);
        assert_eq!(ix.near(Point::new(f64::NAN, 0.0), 1.0), vec![0, 1, 2, 3]);
        assert_eq!(ix.len(), 4);
        // Past the first scans, the tree answers alike.
        for _ in 0..SCANS {
            ix.near(Point::new(0.0, 0.0), 1.0);
        }
        assert_eq!(ix.near(Point::new(12.0, 5.0), 2.0), vec![0, 2]);
        assert_eq!(ix.near(Point::new(5.5, 5.5), 0.0), vec![0, 2, 3]);
        assert!(ix.tree.get().is_some());
    }
}
