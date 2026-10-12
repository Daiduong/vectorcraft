//! A cache of object bounds, keyed by the object's `Arc` (its address), so that a group's bounds
//! come from its members' cached bounds and only what an edit changed is measured again.
//!
//! # Why the cache can't go stale
//!
//! Every entry holds a [`Weak`] to the node it describes. While the entry lives:
//! - the allocation can't be freed, so no other node can ever get the same address (no ABA);
//! - the node can't change in place: `Arc::get_mut` refuses a node with weak references, and
//!   `Arc::make_mut` on it copies (shared) or moves (unique) the value into a new allocation, which
//!   has a new address and so no entry. Nodes have no interior mutability.
//!
//! So an entry found for a live `Arc<Node>` describes exactly that node's current contents, however
//! the tree is edited (through `Session::edit`, `Document::node_mut`, or by hand). Bounds depend on
//! nothing outside the node and its descendants, which an `Arc<Node>` owns.
//!
//! Only objects whose bounds cost more than a lookup are kept (containers, compound paths, live
//! art, type, long paths): a short path is measured again, allocation-free, each time.
//!
//! # Memory
//!
//! Entries of nodes that were let go (an undo step trimmed, a preview replaced) are swept now and
//! then; a shard that still fills up with live nodes (versions kept by the undo history) starts
//! over. The cache is never saved and holds at most [`SHARDS`] × [`SHARD_CAP`] entries.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::{Arc, LazyLock, Mutex, PoisonError, Weak};

use vectorcraft_geom::Rect;

use crate::{Node, NodeKind};

/// What is cached of a node: its geometric, visual, reach and pick bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Geometric = 0,
    Visual = 1,
    Reach = 2,
    Pick = 3,
}

/// One node's cached bounds.
struct Entry {
    /// The node: keeps its address from being reused, and tells a live entry from a dead one.
    node: Weak<Node>,
    /// Bit `k`: `rects[k]` holds the bounds of kind `k`; bit `4 + k`: those bounds are `None`.
    set: u8,
    rects: [Rect; 4],
}

/// Hasher for addresses: one multiply (they are not attacker-chosen keys).
#[derive(Default)]
struct AddrHasher(u64);

impl Hasher for AddrHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
        }
    }
    fn write_usize(&mut self, v: usize) {
        self.0 = (self.0 ^ v as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }
    fn finish(&self) -> u64 {
        self.0 ^ (self.0 >> 29)
    }
}

#[derive(Default)]
struct Shard {
    map: HashMap<usize, Entry, BuildHasherDefault<AddrHasher>>,
    /// The size at which the next sweep runs.
    next_sweep: usize,
}

/// Shards, so that threads measuring at once rarely wait on each other.
const SHARDS: usize = 16;
/// The most entries a shard keeps.
const SHARD_CAP: usize = 4096;
/// The smallest size a sweep waits for.
const MIN_SWEEP: usize = 256;

static CACHE: LazyLock<[Mutex<Shard>; SHARDS]> = LazyLock::new(|| std::array::from_fn(|_| Mutex::new(Shard::default())));

fn shard(addr: usize) -> &'static Mutex<Shard> {
    // Node allocations are at least 8-byte aligned: mix higher bits in.
    let i = ((addr >> 4) ^ (addr >> 11)) % SHARDS;
    &CACHE[i]
}

/// Is `n` worth an entry: do its bounds cost more than looking them up?
fn worth_caching(n: &Node) -> bool {
    match &n.kind {
        NodeKind::Path { path, .. } => path.subpaths.len() > 4 || path.anchor_count() > 32,
        NodeKind::Image(_) | NodeKind::SymbolInstance { .. } | NodeKind::PlacedDocument(_) => false,
        _ => true,
    }
}

fn bits(r: Option<Rect>) -> Option<[u64; 4]> {
    r.map(|r| [r.x0, r.y0, r.x1, r.y1].map(f64::to_bits))
}

/// The bounds of kind `kind` of `n`: cached, else `compute(n)` (then cached when worth it).
fn cached(n: &Arc<Node>, kind: Kind, compute: fn(&Node) -> Option<Rect>) -> Option<Rect> {
    if !worth_caching(n) {
        return compute(n);
    }
    let addr = Arc::as_ptr(n) as usize;
    let k = kind as u8;
    let found = {
        let s = shard(addr).lock().unwrap_or_else(PoisonError::into_inner);
        s.map.get(&addr).filter(|e| e.set & (1 << k) != 0).map(|e| (e.set & (1 << (4 + k)) == 0).then(|| e.rects[k as usize]))
    };
    if let Some(r) = found {
        // The invariant above, checked in debug builds now and then (measuring a container again
        // looks up its members, which check themselves in turn): a cached box is what measuring
        // again gives.
        #[cfg(debug_assertions)]
        {
            static HITS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            if HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed).is_multiple_of(16) {
                debug_assert_eq!(bits(r), bits(compute(n)), "cached {kind:?} bounds of {} went stale", n.id);
            }
        }
        return r;
    }
    // Measured without the lock held: a container's bounds look up its members'.
    let r = compute(n);
    let mut s = shard(addr).lock().unwrap_or_else(PoisonError::into_inner);
    if s.map.len() >= s.next_sweep.max(MIN_SWEEP) && !s.map.contains_key(&addr) {
        s.map.retain(|_, e| e.node.strong_count() > 0);
        if s.map.len() >= SHARD_CAP {
            s.map.clear();
        }
        s.next_sweep = (s.map.len() * 2).clamp(MIN_SWEEP, SHARD_CAP);
    }
    let e = s.map.entry(addr).or_insert_with(|| Entry { node: Arc::downgrade(n), set: 0, rects: [Rect::ZERO; 4] });
    e.set |= 1 << k;
    match r {
        Some(b) => e.rects[k as usize] = b,
        None => e.set |= 1 << (4 + k),
    }
    r
}

/// [`Node::geometric_bounds`] of `n`, cached.
pub fn geometric(n: &Arc<Node>) -> Option<Rect> {
    cached(n, Kind::Geometric, Node::geometric_bounds)
}

/// [`Node::visual_bounds`] of `n`, cached.
pub fn visual(n: &Arc<Node>) -> Option<Rect> {
    cached(n, Kind::Visual, Node::visual_bounds)
}

/// [`Node::reach_bounds`] of `n`, cached.
pub fn reach(n: &Arc<Node>) -> Option<Rect> {
    cached(n, Kind::Reach, Node::reach_bounds)
}

/// [`Node::pick_bounds`] of `n`, cached.
pub fn pick(n: &Arc<Node>) -> Option<Rect> {
    cached(n, Kind::Pick, Node::pick_bounds)
}

/// How many entries the cache holds (live or not yet swept): for tests and measurements.
pub fn entries() -> usize {
    CACHE.iter().map(|s| s.lock().unwrap_or_else(PoisonError::into_inner).map.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Appearance, Document};
    use vectorcraft_geom::{Affine, Point, shapes};

    fn group_of(d: &mut Document, n: usize) -> Arc<Node> {
        let kids = (0..n)
            .map(|i| {
                let x = i as f64 * 3.0;
                Arc::new(Node::path(d.alloc_id(), shapes::rectangle(Rect::new(x, 0.0, x + 2.0, 2.0)), Appearance::default_art()))
            })
            .collect();
        Arc::new(Node::group(d.alloc_id(), kids))
    }

    #[test]
    fn a_group_is_measured_once_and_a_copy_on_write_edit_measures_anew() {
        let mut d = Document::new(100.0, 100.0);
        let g = group_of(&mut d, 10);
        let before = geometric(&g);
        assert_eq!(before, g.geometric_bounds());
        assert_eq!(geometric(&g), before, "cached");
        // An edit of a shared node copies it: the copy has no entry and is measured.
        let mut edited = g.clone();
        Arc::make_mut(&mut edited).transform(Affine::translate((5.0, 0.0)), false);
        assert!(!Arc::ptr_eq(&g, &edited));
        assert_eq!(geometric(&edited), before.map(|b| b + vectorcraft_geom::Vec2::new(5.0, 0.0)));
        assert_eq!(geometric(&g), before, "the original keeps its entry");
    }

    /// The node an entry describes can't change in place: `make_mut` on a node the cache knows
    /// moves it to a new allocation even when nothing else shares it, and `get_mut` refuses it.
    #[test]
    fn a_unique_node_with_an_entry_is_not_changed_in_place() {
        let mut d = Document::new(100.0, 100.0);
        let mut g = group_of(&mut d, 3);
        let at = Arc::as_ptr(&g);
        let first = visual(&g);
        assert!(Arc::get_mut(&mut g).is_none(), "the entry's weak reference blocks get_mut");
        let n = Arc::make_mut(&mut g);
        n.transform(Affine::translate((0.0, 7.0)), false);
        assert_ne!(Arc::as_ptr(&g), at, "moved out, not changed in place");
        assert_eq!(visual(&g), g.visual_bounds());
        assert_ne!(visual(&g), first);
    }

    #[test]
    fn none_is_cached_too_and_short_paths_are_not_kept() {
        let mut d = Document::new(100.0, 100.0);
        let empty = Arc::new(Node::group(d.alloc_id(), vec![]));
        assert_eq!(geometric(&empty), None);
        assert_eq!(geometric(&empty), None);
        let short = Arc::new(Node::path(d.alloc_id(), shapes::ellipse(Rect::new(0.0, 0.0, 4.0, 4.0)), Appearance::default_art()));
        assert!(!worth_caching(&short));
        assert_eq!(reach(&short), short.reach_bounds());
        let long = Arc::new(Node::path(
            d.alloc_id(),
            vectorcraft_geom::PathData::single(vectorcraft_geom::SubPath::polyline(
                &(0..100).map(|i| Point::new(i as f64, (i % 7) as f64)).collect::<Vec<_>>(),
                false,
            )),
            Appearance::default_art(),
        ));
        assert!(worth_caching(&long));
        assert_eq!(pick(&long), long.pick_bounds());
    }

    /// Entries of nodes let go are swept, and the cache stays bounded however many nodes pass
    /// through it.
    #[test]
    fn the_cache_stays_bounded() {
        let mut d = Document::new(100.0, 100.0);
        for _ in 0..200 {
            let g = group_of(&mut d, 1);
            let _ = geometric(&g);
        }
        assert!(entries() <= SHARDS * SHARD_CAP);
        let live: Vec<Arc<Node>> = (0..(SHARDS * SHARD_CAP / 8)).map(|_| Arc::new(Node::group(d.alloc_id(), vec![]))).collect();
        for g in &live {
            let _ = geometric(g);
        }
        assert!(entries() <= SHARDS * SHARD_CAP);
    }
}
